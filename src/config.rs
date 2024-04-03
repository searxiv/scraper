#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    pub archivist_url: String,
    pub archivist_new_task_path: String,
    pub archivist_submit_task_path: String,
    pub search_url_pattern: String,
    pub request_interval_millis: u64,
    pub concurrent_jobs: usize,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            archivist_url: "http://fire:9000/".to_string(),
            archivist_new_task_path: "/tasks".to_string(),
            archivist_submit_task_path: "/tasks".to_string(),
            request_interval_millis: 500,
            log_level: "info".to_string(),
            concurrent_jobs: 5,
            search_url_pattern: "
                https://arxiv.org/search/advanced?
                advanced=&terms-0-operator=AND&
                terms-0-term=&
                terms-0-field=title&
                classification-computer_science=y&
                classification-economics=y&
                classification-eess=y&
                classification-mathematics=y&
                classification-physics=y&
                classification-physics_archives=all&
                classification-q_biology=y&
                classification-q_finance=y&
                classification-statistics=y&
                classification-include_cross_list=include&
                date-year=&
                date-filter_by=date_range&
                date-from_date={}&
                date-to_date={}&
                date-date_type=submitted_date_first&
                abstracts=hide&
                size=200&
                order=-announced_date_first"
                .to_string(),
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
