use crate::config;
use crate::models;
use crate::models::NewTask;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network error")]
    Network(#[from] reqwest::Error),

    #[error("file error")]
    File(#[from] std::io::Error),
}

pub struct Scraper {
    pub client: reqwest::Client,
    config: config::Config,
    _last_request: Arc<Mutex<std::time::Instant>>,
}

impl Scraper {
    pub async fn new(config: config::Config) -> Scraper {
        let client = reqwest::Client::builder()
            .user_agent("Googlebot")
            .build()
            .unwrap();
        let last_request = Arc::new(Mutex::new(std::time::Instant::now()));

        Self {
            client,
            config,
            _last_request: last_request,
        }
    }

    pub async fn get_next_task(&self) -> Option<models::NewTask> {
        let new_task_url = format!(
            "{}{}",
            self.config.archivist_url, self.config.archivist_new_task_path
        );

        let res = self.client.get(&new_task_url).send().await;

        if let Err(e) = res {
            log::error!("Request for new task failed: {e}");
            return None;
        }
        let response = res.unwrap();

        if !response.status().is_success() {
            return None;
        }

        let body = response.text().await.unwrap();

        serde_json::from_str(&body).ok()
    }

    pub async fn scrape(&self, _task: NewTask) {}
}
