use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    NotRunning,
    Starting,
    Running,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PCloudConfig {
    pub username: String,
    pub mount_point: String,
    pub cache_size_gb: u64,
    pub log_path: String,
    pub log_level: String,
    pub save_password: bool,
}

impl Default for PCloudConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            mount_point: String::new(),
            cache_size_gb: 5,
            log_path: String::new(),
            log_level: "INFO".to_string(),
            save_password: false,
        }
    }
}

pub struct ProcessManager {
    child: Option<Child>,
    status: ProcessStatus,
    config: PCloudConfig,
    pcloudcc_path: String,
}

impl ProcessManager {
    pub fn new(pcloudcc_path: String) -> Self {
        Self {
            child: None,
            status: ProcessStatus::NotRunning,
            config: PCloudConfig::default(),
            pcloudcc_path,
        }
    }

    pub fn set_config(&mut self, config: PCloudConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &PCloudConfig {
        &self.config
    }

    pub fn get_process_status(&self) -> &ProcessStatus {
        &self.status
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, ProcessStatus::Running)
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if self.is_running() {
            return Err("pcloudcc is already running".to_string());
        }

        if self.config.username.is_empty() {
            return Err("Username is required".to_string());
        }

        if !Path::new(&self.pcloudcc_path).exists() {
            return Err(format!("pcloudcc binary not found at: {}", self.pcloudcc_path));
        }

        self.status = ProcessStatus::Starting;

        let mut cmd = Command::new(&self.pcloudcc_path);
        cmd.arg("-u").arg(&self.config.username);
        cmd.arg("-d"); // daemon mode

        if !self.config.mount_point.is_empty() {
            cmd.arg("-m").arg(&self.config.mount_point);
        }

        if self.config.cache_size_gb > 0 {
            cmd.arg("--cache-size").arg(self.config.cache_size_gb.to_string());
        }

        if !self.config.log_path.is_empty() {
            cmd.arg("--log-path").arg(&self.config.log_path);
        }

        if !self.config.log_level.is_empty() {
            cmd.arg("--log-level").arg(&self.config.log_level);
        }

        if self.config.save_password {
            cmd.arg("-s");
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(child) => {
                self.child = Some(child);
                self.status = ProcessStatus::Running;
                Ok(())
            }
            Err(e) => {
                self.status = ProcessStatus::Error(e.to_string());
                Err(format!("Failed to start pcloudcc: {}", e))
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if !self.is_running() {
            return Err("pcloudcc is not running".to_string());
        }

        self.status = ProcessStatus::Stopping;

        if let Some(mut child) = self.child.take() {
            // Try graceful shutdown first using SIGTERM
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;
                if let Err(e) = signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM) {
                    self.status = ProcessStatus::Error(e.to_string());
                    return Err(format!("Failed to send SIGTERM: {}", e));
                }
            }

            #[cfg(not(unix))]
            {
                child.kill().map_err(|e| format!("Failed to kill process: {}", e))?;
            }

            // Wait for process to exit
            let _ = sleep(Duration::from_secs(2)).await;
            let _ = child.try_wait();

            self.status = ProcessStatus::NotRunning;
            Ok(())
        } else {
            self.status = ProcessStatus::NotRunning;
            Ok(())
        }
    }

    pub async fn send_command(&self, command: &str) -> Result<String, String> {
        if !self.is_running() {
            return Err("pcloudcc is not running".to_string());
        }

        let mut child = Command::new(&self.pcloudcc_path)
            .arg("-k")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "{}", command)
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            writeln!(stdin, "quit")
                .map_err(|e| format!("Failed to write quit to stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub async fn get_daemon_status(&self) -> Result<String, String> {
        self.send_command("status").await
    }

    pub async fn get_pending(&self) -> Result<String, String> {
        self.send_command("pending").await
    }

    pub async fn list_sync_folders(&self) -> Result<String, String> {
        self.send_command("sync list").await
    }

    pub async fn pause_sync(&self) -> Result<String, String> {
        self.send_command("sync pause").await
    }

    pub async fn resume_sync(&self) -> Result<String, String> {
        self.send_command("sync resume").await
    }

    pub async fn start_crypto(&self, password: &str) -> Result<String, String> {
        let cmd = format!("crypto start {}", password);
        self.send_command(&cmd).await
    }

    pub async fn stop_crypto(&self) -> Result<String, String> {
        self.send_command("crypto stop").await
    }

    pub async fn finalize(&self) -> Result<String, String> {
        self.send_command("finalize").await
    }
}
