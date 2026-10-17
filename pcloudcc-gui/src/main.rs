mod process_manager;

use eframe::egui;
use process_manager::{PCloudConfig, ProcessManager, ProcessStatus};
use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq)]
enum Tab {
    Status,
    Configuration,
    SyncFolders,
    Crypto,
}

struct PCloudGui {
    manager: Arc<Mutex<ProcessManager>>,
    current_tab: Tab,
    config: PCloudConfig,
    status_output: String,
    pending_output: String,
    sync_folders_output: String,
    pcloudcc_path: String,
    show_password: bool,
    crypto_password: String,
    error_message: Option<String>,
    success_message: Option<String>,
    runtime: tokio::runtime::Runtime,
}

impl PCloudGui {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Try to find pcloudcc in common locations
        let pcloudcc_path = find_pcloudcc_binary();

        // Create tokio runtime for async operations
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
            manager: Arc::new(Mutex::new(ProcessManager::new(pcloudcc_path.clone()))),
            current_tab: Tab::Status,
            config: PCloudConfig::default(),
            status_output: String::new(),
            pending_output: String::new(),
            sync_folders_output: String::new(),
            pcloudcc_path,
            show_password: false,
            crypto_password: String::new(),
            error_message: None,
            success_message: None,
            runtime,
        }
    }
}

fn find_pcloudcc_binary() -> String {
    // Try common locations
    let paths = vec![
        "./pcloudcc",
        "/usr/local/bin/pcloudcc",
        "/usr/bin/pcloudcc",
        "~/.local/bin/pcloudcc",
    ];

    for path in paths {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Default to current directory
    "./pcloudcc".to_string()
}

impl eframe::App for PCloudGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle window close - minimize to background instead
        ctx.input(|i| {
            if i.viewport().close_requested() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        });

        // Check for restore via global hotkey (Ctrl+Shift+P)
        ctx.input(|i| {
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::P) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            }
        });

        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Status, "Status");
                ui.selectable_value(&mut self.current_tab, Tab::Configuration, "Configuration");
                ui.selectable_value(&mut self.current_tab, Tab::SyncFolders, "Sync Folders");
                ui.selectable_value(&mut self.current_tab, Tab::Crypto, "Crypto");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Ctrl+Shift+P to restore").small().italics());
                    ui.label("|");
                    ui.label(egui::RichText::new("Close to minimize").small().italics());
                });
            });
        });

        // Show messages
        let error_to_show = self.error_message.clone();
        if let Some(error) = error_to_show {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(&error).color(egui::Color32::RED));
                    if ui.button("OK").clicked() {
                        self.error_message = None;
                    }
                });
        }

        let success_to_show = self.success_message.clone();
        if let Some(success) = success_to_show {
            egui::Window::new("Success")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(&success).color(egui::Color32::GREEN));
                    if ui.button("OK").clicked() {
                        self.success_message = None;
                    }
                });
        }

        // Main content based on selected tab
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Status => self.status_tab(ui, ctx),
                Tab::Configuration => self.configuration_tab(ui),
                Tab::SyncFolders => self.sync_folders_tab(ui, ctx),
                Tab::Crypto => self.crypto_tab(ui, ctx),
            }
        });

        // Bottom panel with minimize/quit buttons
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Minimize to Background").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    }
}

impl PCloudGui {
    fn status_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("pCloudCC Status");

        ui.add_space(10.0);

        // Display process status
        let manager = self.manager.lock().unwrap();
        let status = manager.get_process_status().clone();
        drop(manager);

        let status_text = match &status {
            ProcessStatus::NotRunning => "Not Running",
            ProcessStatus::Starting => "Starting...",
            ProcessStatus::Running => "Running",
            ProcessStatus::Stopping => "Stopping...",
            ProcessStatus::Error(msg) => &format!("Error: {}", msg),
        };

        let status_color = match &status {
            ProcessStatus::Running => egui::Color32::GREEN,
            ProcessStatus::Error(_) => egui::Color32::RED,
            ProcessStatus::NotRunning => egui::Color32::GRAY,
            _ => egui::Color32::YELLOW,
        };

        ui.label(egui::RichText::new(format!("Status: {}", status_text)).color(status_color));
        ui.label(format!("Binary: {}", self.pcloudcc_path));

        ui.add_space(20.0);

        // Start/Stop buttons
        ui.horizontal(|ui| {
            if matches!(status, ProcessStatus::NotRunning | ProcessStatus::Error(_)) {
                if ui.button("Start pCloudCC").clicked() {
                    let mut manager = self.manager.lock().unwrap();
                    manager.set_config(self.config.clone());
                    let result = self.runtime.block_on(manager.start());

                    match result {
                        Ok(_) => {
                            self.success_message = Some("pCloudCC started successfully".to_string());
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                        }
                    }
                }
            } else if matches!(status, ProcessStatus::Running) {
                if ui.button("Stop pCloudCC").clicked() {
                    let mut manager = self.manager.lock().unwrap();
                    let result = self.runtime.block_on(manager.stop());

                    match result {
                        Ok(_) => {
                            self.success_message = Some("pCloudCC stopped successfully".to_string());
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                        }
                    }
                }
            }
        });

        ui.add_space(20.0);

        // Refresh buttons
        ui.horizontal(|ui| {
            if ui.button("Refresh Status").clicked() {
                let manager = self.manager.lock().unwrap();
                let result = tokio::runtime::Handle::try_current()
                    .unwrap()
                    .block_on(manager.get_daemon_status());

                match result {
                    Ok(output) => {
                        self.status_output = output;
                    }
                    Err(e) => {
                        self.status_output = format!("Error: {}", e);
                    }
                }
            }

            if ui.button("Check Pending").clicked() {
                let manager = self.manager.lock().unwrap();
                let result = tokio::runtime::Handle::try_current()
                    .unwrap()
                    .block_on(manager.get_pending());

                match result {
                    Ok(output) => {
                        self.pending_output = output;
                    }
                    Err(e) => {
                        self.pending_output = format!("Error: {}", e);
                    }
                }
            }
        });

        ui.add_space(20.0);

        // Status output
        ui.separator();
        ui.label("Status Output:");
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                ui.label(&self.status_output);
            });

        ui.add_space(10.0);

        // Pending output
        ui.label("Pending Transfers:");
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                ui.label(&self.pending_output);
            });

        // Auto-refresh status
        ctx.request_repaint();
    }

    fn configuration_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("pcloudcc Binary:");
            if ui.text_edit_singleline(&mut self.pcloudcc_path).changed() {
                let mut manager = self.manager.lock().unwrap();
                *manager = ProcessManager::new(self.pcloudcc_path.clone());
            }
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.config.username);
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Mount Point:");
            ui.text_edit_singleline(&mut self.config.mount_point);
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Cache Size (GB):");
            ui.add(egui::Slider::new(&mut self.config.cache_size_gb, 1..=1024));
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Log Path:");
            ui.text_edit_singleline(&mut self.config.log_path);
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Log Level:");
            egui::ComboBox::from_label("")
                .selected_text(&self.config.log_level)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.log_level, "NONE".to_string(), "NONE");
                    ui.selectable_value(&mut self.config.log_level, "ERROR".to_string(), "ERROR");
                    ui.selectable_value(&mut self.config.log_level, "WARNING".to_string(), "WARNING");
                    ui.selectable_value(&mut self.config.log_level, "INFO".to_string(), "INFO");
                    ui.selectable_value(&mut self.config.log_level, "NOTICE".to_string(), "NOTICE");
                    ui.selectable_value(&mut self.config.log_level, "DEBUG".to_string(), "DEBUG");
                });
        });

        ui.add_space(10.0);

        ui.checkbox(&mut self.config.save_password, "Save Password");

        ui.add_space(20.0);

        if ui.button("Apply Configuration").clicked() {
            let mut manager = self.manager.lock().unwrap();
            manager.set_config(self.config.clone());
            self.success_message = Some("Configuration applied".to_string());
        }
    }

    fn sync_folders_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Sync Folders");

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("List Sync Folders").clicked() {
                let manager = self.manager.lock().unwrap();
                let result = tokio::runtime::Handle::try_current()
                    .unwrap()
                    .block_on(manager.list_sync_folders());

                match result {
                    Ok(output) => {
                        self.sync_folders_output = output;
                    }
                    Err(e) => {
                        self.sync_folders_output = format!("Error: {}", e);
                    }
                }
            }

            if ui.button("Pause Sync").clicked() {
                let manager = self.manager.lock().unwrap();
                let result = tokio::runtime::Handle::try_current()
                    .unwrap()
                    .block_on(manager.pause_sync());

                match result {
                    Ok(_) => {
                        self.success_message = Some("Sync paused".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }

            if ui.button("Resume Sync").clicked() {
                let manager = self.manager.lock().unwrap();
                let result = tokio::runtime::Handle::try_current()
                    .unwrap()
                    .block_on(manager.resume_sync());

                match result {
                    Ok(_) => {
                        self.success_message = Some("Sync resumed".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }
        });

        ui.add_space(20.0);

        ui.separator();
        ui.label("Sync Folders:");
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                ui.label(&self.sync_folders_output);
            });
    }

    fn crypto_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Crypto Folder");

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Crypto Password:");
            if self.show_password {
                ui.text_edit_singleline(&mut self.crypto_password);
            } else {
                ui.add(
                    egui::TextEdit::singleline(&mut self.crypto_password)
                        .password(true),
                );
            }
            ui.checkbox(&mut self.show_password, "Show");
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Start Crypto").clicked() {
                if self.crypto_password.is_empty() {
                    self.error_message = Some("Crypto password is required".to_string());
                } else {
                    let manager = self.manager.lock().unwrap();
                    let result = self.runtime.block_on(manager.start_crypto(&self.crypto_password));

                    match result {
                        Ok(_) => {
                            self.success_message = Some("Crypto folder started".to_string());
                            self.crypto_password.clear();
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                        }
                    }
                }
            }

            if ui.button("Stop Crypto").clicked() {
                let manager = self.manager.lock().unwrap();
                let result = tokio::runtime::Handle::try_current()
                    .unwrap()
                    .block_on(manager.stop_crypto());

                match result {
                    Ok(_) => {
                        self.success_message = Some("Crypto folder stopped".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }
        });

        ui.add_space(20.0);

        ui.label(
            "Note: Crypto folder must be unlocked before you can access encrypted files.",
        );
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "pCloudCC GUI",
        options,
        Box::new(|cc| Box::new(PCloudGui::new(cc))),
    )
}
