mod config;
mod models;
mod scraper;

use config::Config;
use figment::{providers::Env, Figment};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env
    dotenvy::dotenv()?;

    // Build config by merging environment variables with Config::default()
    let config: Config = Figment::from(Config::default())
        .merge(Env::prefixed("SCRAPER_"))
        .extract()?;

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or(&config.log_level));

    let scraper = scraper::Scraper::new(config).await;

    // Constantly wait for tasks
    loop {
        let task = scraper.get_next_task().await;
        if let Some(task) = task {
            scraper.scrape(task).await;
        }

        sleep(Duration::from_millis(500)).await;
    }
}
