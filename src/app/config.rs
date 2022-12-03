use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default = "default_app_environment")]
    pub app_environment: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_app_environment() -> String {
    String::from("development")
}

fn default_log_level() -> String {
    String::from("debug")
}
