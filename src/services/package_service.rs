use std::process::{Command, Stdio};

use crate::models::package_info::{
    InstallFromFlathubConfig, InstallFromPacmanConfig, PackageInfo, PackageInstallStep,
    RunCustomScriptConfig, RunInTerminalConfig,
};

pub const EMBED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/packages.json"));

pub struct PackageService;

impl PackageService {
    pub fn load_packages() -> Result<Vec<PackageInfo>, String> {
        let result = serde_json::from_str(EMBED);
        let list: Vec<PackageInfo> =
            result.map_err(|e| format!("Failed to parse packages.json: {}", e))?;

        Ok(list)
    }

    pub fn authenticate() -> Result<(), String> {
        let status = std::process::Command::new("true")
            .status()
            .map_err(|e| format!("Failed to execute: {}", e))?;
        if !status.success() {
            return Err(format!(
                "Installation failed with exit code: {:?}",
                status.code()
            ));
        }

        Ok(())
    }

    pub fn install_from_flathub(config: &InstallFromFlathubConfig) -> Result<(), String> {
        let status = std::process::Command::new("flatpak")
            .arg("install")
            .arg("-y")
            .arg("flathub")
            .arg(&config.id)
            .status()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Flatpak installation failed with exit code: {:?}",
                status.code()
            ))
        }
    }

    pub fn install_from_pacman(config: &InstallFromPacmanConfig) -> Result<(), String> {
        let status = std::process::Command::new("pkexec")
            .arg("pacman")
            .arg("-S")
            .arg("--noconfirm")
            .arg(&config.id)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to execute pacman: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Pacman installation failed with exit code: {:?}",
                status.code()
            ))
        }
    }

    pub fn run_custom_script(config: &RunCustomScriptConfig) -> Result<(), String> {
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&config.script)
            .status()
            .map_err(|e| format!("Failed to execute custom script: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Custom script execution failed with exit code: {:?}",
                status.code()
            ))
        }
    }

    pub fn run_in_terminal(config: &RunInTerminalConfig) -> Result<(), String> {
        let status = Command::new("kgx")
            .arg("-e")
            .arg(&config.script)
            .status()
            .map_err(|e| format!("Failed to execute script: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "Script execution failed with exit code: {:?}",
                status.code()
            ))
        }
    }
    pub fn run_step(step: &PackageInstallStep) -> Result<(), String> {
        match step {
            PackageInstallStep::InstallFromFlathub(install_from_flathub_config) => {
                PackageService::install_from_flathub(install_from_flathub_config)
            }
            PackageInstallStep::InstallFromPacman(install_from_pacman_config) => {
                PackageService::install_from_pacman(install_from_pacman_config)
            }
            PackageInstallStep::RunCustomScript(run_custom_script_config) => {
                PackageService::run_custom_script(run_custom_script_config)
            }
            PackageInstallStep::RunInTerminal(run_in_termial_config) => {
                PackageService::run_in_terminal(run_in_termial_config)
            }
        }
    }

    pub fn install_pkg(pkg: &PackageInfo) -> Result<(), String> {
        for step in &pkg.install_steps {
            PackageService::run_step(step)?;
        }

        Ok(())
    }
}
