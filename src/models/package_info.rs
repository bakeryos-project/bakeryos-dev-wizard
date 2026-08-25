use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub group: PackageGroup,
    pub install_source: PackageInstallSource,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PackageGroup {
    CodeEditor,
    ProgrammingLanguage,
    Framework,
    Tooling,
    #[serde(rename = "ai_agent")]
    AIAgent,
    Browser,
    #[default]
    Other,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PackageInstallSource {
    Flathub(FlathubPackageMetadata),
    Pacman(PacmanPackageMetadata),

    CustomScript(CustomScript),
}

impl Default for PackageInstallSource {
    fn default() -> Self {
        Self::CustomScript(CustomScript {
            script: String::new(),
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct FlathubPackageMetadata {
    pub id: String,
    pub repository: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct PacmanPackageMetadata {
    pub id: String,
    pub repository: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CustomScript {
    pub script: String,
}
