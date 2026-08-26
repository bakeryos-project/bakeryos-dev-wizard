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
    pub install_steps: Vec<PackageInstallStep>,
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
pub enum PackageInstallStep {
    InstallFromFlathub(InstallFromFlathubConfig),
    InstallFromPacman(InstallFromPacmanConfig),
    RunCustomScript(RunCustomScriptConfig),
    RunInTerminal(RunInTerminalConfig),
}

impl Default for PackageInstallStep {
    fn default() -> Self {
        Self::RunCustomScript(RunCustomScriptConfig {
            script: String::new(),
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct InstallFromFlathubConfig {
    pub id: String,
    pub repository: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct InstallFromPacmanConfig {
    pub id: String,
    pub repository: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RunCustomScriptConfig {
    pub script: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RunInTerminalConfig {
    pub script: String,
}
