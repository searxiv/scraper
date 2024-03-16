#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    pub archivist_url: String,
    pub archivist_new_task_path: String,
    pub request_interval_millis: u32,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            archivist_url: "http://fire:9000/".to_string(),
            archivist_new_task_path: "/tasks".to_string(),
            request_interval_millis: 500,
            log_level: "info".to_string(),
        }
    }
}

use figment::{
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Scraper config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
