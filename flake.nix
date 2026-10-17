{
  description = "pcloudcc-lneely dev environment";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];
      perSystem = { config, self', inputs', pkgs, system, ... }: {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "pcloudcc-lneely";
          version = "unstable-2026-10-16";

          src = ./.;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            zlib
            sqlite
            libudev-zero
            readline
            fuse
            mbedtls
          ];

          makeFlags = [
            "BUILD=release"
            "FORCE_FUSE=2"
          ];

          installPhase = ''
            runHook preInstall
            install -Dm755 pcloudcc $out/bin/pcloudcc
            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Linux console client for pCloud cloud storage";
            homepage = "https://github.com/mathew-d/pcloudcc-lneely";
            license = licenses.bsd3;
            mainProgram = "pcloudcc";
            platforms = platforms.linux;
          };
        };

        apps.default = {
          type = "app";
          program = "${self'.packages.default}/bin/pcloudcc";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bear
            clang-tools
            pkg-config
            zlib
            sqlite
            libudev-zero
            readline
            fuse
            mbedtls
            watchexec
          ];

          CFLAGS = [
            "-I${pkgs.zlib.dev}/include"
            "-I${pkgs.sqlite.dev}/include"
            "-I${pkgs.readline.dev}/include"
            "-I${pkgs.fuse.dev}/include"
            "-I${pkgs.mbedtls}/include"
          ];

          CXXFLAGS = [
            "-I${pkgs.zlib.dev}/include"
            "-I${pkgs.sqlite.dev}/include"
            "-I${pkgs.readline.dev}/include"
            "-I${pkgs.fuse.dev}/include"
            "-I${pkgs.mbedtls}/include"
          ];

        };
      };
    };
}
