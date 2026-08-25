use std::sync::Arc;

use crate::models::package_info::PackageInfo;

pub const EMBED: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/packages.json"));

pub struct PackageService;

impl PackageService {
    pub fn load_packages() -> Result<Arc<Vec<PackageInfo>>, String> {
        let result = serde_json::from_str(EMBED);
        let list: Vec<PackageInfo> =
            result.map_err(|e| format!("Failed to parse packages.json: {}", e))?;

        Ok(Arc::new(list))
    }

    pub fn authenticate() -> Result<(), String> {
        let status = std::process::Command::new("flatpak-spawn")
            .arg("--host")
            .arg("true")
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
    pub fn install_pkg(pkg: &PackageInfo) -> Result<(), String> {
        match &pkg.install_source {
            crate::models::package_info::PackageInstallSource::Flathub(
                flathub_package_metadata,
            ) => {
                let status = std::process::Command::new("flatpak-spawn")
                    .arg("--host")
                    .arg("flatpak")
                    .arg("install")
                    .arg("-y")
                    .arg("flathub")
                    .arg(&flathub_package_metadata.id)
                    .status()
                    .map_err(|e| format!("Failed to execute flatpak-spawn: {}", e))?;

                if status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "Flatpak installation failed with exit code: {:?}",
                        status.code()
                    ))
                }
            }

            crate::models::package_info::PackageInstallSource::Pacman(pacman_package_metadata) => {
                let status = std::process::Command::new("flatpak-spawn")
                    .arg("--host")
                    .arg("pkexec")
                    .arg("pacman")
                    .arg("-S")
                    .arg("--noconfirm")
                    .arg(&pacman_package_metadata.id)
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

            crate::models::package_info::PackageInstallSource::CustomScript(custom_script) => {
                let status = std::process::Command::new("flatpak-spawn")
                    .arg("--host")
                    .arg("sh")
                    .arg("-c")
                    .arg(&custom_script.script)
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
        }
    }
}
