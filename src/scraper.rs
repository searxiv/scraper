use crate::config;
use crate::models;
use crate::models::NewTask;

use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("network error")]
    Network(#[from] reqwest::Error),

    #[error("file error")]
    File(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

type Url = String;

pub struct Scraper {
    pub config: config::Config,
    client: reqwest::Client,
    last_request: Arc<Mutex<std::time::Instant>>,
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
            last_request,
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

    pub async fn scrape(&self, task: NewTask) -> anyhow::Result<()> {
        let start_date = task.submission_date.format("%Y-%m-%d").to_string();
        let end_date = task
            .submission_date
            .succ_opt()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();

        let start_url = format!(
            "https://arxiv.org/search/advanced?\
             advanced=&terms-0-operator=AND&\
             terms-0-term=&\
             terms-0-field=title&\
             classification-computer_science=y&\
             classification-economics=y&\
             classification-eess=y&\
             classification-mathematics=y&\
             classification-physics=y&\
             classification-physics_archives=all&\
             classification-q_biology=y&\
             classification-q_finance=y&\
             classification-statistics=y&\
             classification-include_cross_list=include&\
             date-year=&\
             date-filter_by=date_range&\
             date-from_date={}&\
             date-to_date={}&\
             date-date_type=submitted_date_first&\
             abstracts=hide&\
             size=200&\
             order=-announced_date_first",
            start_date, end_date
        );

        let mut paper_urls = Vec::new();
        let mut current_url = start_url;
        loop {
            let (page_paper_urls, next_page_url) = self.scrape_page(current_url.to_string()).await;
            paper_urls.extend(page_paper_urls.into_iter());

            if let Some(next_page_url) = next_page_url {
                current_url = next_page_url;
            } else {
                break;
            }
        }

        let paper_urls_to_download = paper_urls
            .iter()
            .map(|url| url.replace("arxiv.org", "export.arxiv.org"))
            .collect::<Vec<_>>();

        log::info!("Starting to scrape {} papers", paper_urls_to_download.len());

        let total_progress = indicatif::ProgressBar::new(paper_urls_to_download.len() as u64)
            .with_style(
                indicatif::ProgressStyle::with_template(
                    "{elapsed_precise:.dim} {bar:50.cyan/blue} {pos}/{len}",
                )
                .unwrap(),
            );
        total_progress.enable_steady_tick(std::time::Duration::from_millis(100));
        let amtp = Arc::new(Mutex::new(total_progress));

        let paper_futures = paper_urls_to_download
            .iter()
            .map(|url| self.scrape_paper(url, task.submission_date, &amtp));
        let stream = futures::stream::iter(paper_futures)
            .buffer_unordered(10)
            .collect::<Vec<_>>();

        let papers = stream
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        let submission = models::TaskSubmission {
            submission_date: task.submission_date,
            papers,
        };
        let submission_json = serde_json::to_string(&submission)?;
        let json_size_mb = submission_json.len() as f64 / 1024. / 1024.;
        log::info!("Sent body size of {}", json_size_mb);

        let submit_path = format!(
            "{}{}",
            &self.config.archivist_url, &self.config.archivist_submit_task_path
        );

        self.client
            .post(&submit_path)
            .header("Content-Type", "application/json")
            .body(submission_json)
            .send()
            .await?;

        Ok(())
    }

    pub async fn scrape_page(&self, url: Url) -> (Vec<Url>, Option<String>) {
        let dom = match self.get_dom(&url).await {
            Ok(dom) => dom,
            Err(e) => {
                log::error!("Could not scrape {url:?}: {e}");
                return (Vec::new(), None);
            }
        };

        let paper_link_selector = scraper::Selector::parse(".list-title > a").unwrap();
        let paper_links = dom
            .select(&paper_link_selector)
            .map(|l| l.value().attr("href").unwrap().to_string())
            .collect::<Vec<Url>>();

        let next_page_selector = scraper::Selector::parse("a.pagination-next").unwrap();
        let mut next_page_url = None;
        if let Some(next_page_href) = dom.select(&next_page_selector).next() {
            let mut next_page = "https://arxiv.org".to_string();
            let next_page_href = next_page_href.value().attr("href").unwrap();
            next_page.push_str(next_page_href);

            next_page_url = Some(next_page);
        }

        (paper_links, next_page_url)
    }

    pub async fn scrape_paper(
        &self,
        url: &Url,
        submission_date: chrono::NaiveDate,
        sp: &Arc<Mutex<indicatif::ProgressBar>>,
    ) -> Result<models::NewPaperFull> {
        let dom = self.get_dom(url).await?;

        let title = select_title(&dom);
        let description = select_description(&dom);
        let arxiv_id = url.split_once("/abs/").unwrap().1.to_string();

        let pdf_url = url.replace("abs", "pdf");
        let pdf_bytes = self.download_pdf(pdf_url).await?;
        let body = body_from_pdf(&pdf_bytes);

        if body.is_empty() {
            log::warn!("PDF: empty body {url:?}")
        }

        let authors = select_authors(&dom)?;
        let subjects = select_subjects(&dom)?;
        let new_paper = models::NewPaperFull {
            arxiv_id,
            submission_date,
            title,
            body,
            description,
            authors,
            subjects,
        };

        sp.lock().await.inc(1);

        Ok(new_paper)
    }

    async fn get_dom(&self, url: &Url) -> Result<scraper::Html> {
        let page = self.get(url).await?;
        let body = page.text().await?;
        let dom = scraper::Html::parse_document(&body);

        Ok(dom)
    }

    async fn download_pdf(&self, url: Url) -> Result<glib::Bytes> {
        let response = self.get(&url).await?;
        Ok(glib::Bytes::from_owned(response.bytes().await?))
    }

    async fn get(&self, url: &Url) -> Result<reqwest::Response> {
        let mut last_request = self.last_request.lock().await;
        let now = std::time::Instant::now();

        let since_last_request = now - *last_request;
        if since_last_request < std::time::Duration::from_secs(1) {
            std::thread::sleep(std::time::Duration::from_secs(1) - since_last_request);
        }
        *last_request = now;
        drop(last_request);

        let mut backoff = std::time::Duration::from_secs(1);
        loop {
            log::trace!("Reqwest: GET {url:?}");

            let response = self.client.get(url).send().await;
            if response.is_ok() {
                return response.map_err(|e| e.into());
            }
            if backoff > std::time::Duration::from_secs(60) {
                log::error!("Too many backoff steps: {response:?}");
                return response.map_err(|e| e.into());
            }

            tokio::time::sleep(backoff).await;
            backoff *= 2;
        }
    }
}

fn select_title(dom: &scraper::Html) -> String {
    let title_selector = scraper::Selector::parse("h1.title").unwrap();
    dom.select(&title_selector)
        .next()
        .map(|el| {
            el.text()
                .collect::<String>()
                .trim()
                .trim_start_matches("Title:")
                .trim_start()
                .replace("  ", " ")
                .to_string()
        })
        .unwrap_or_default()
}

fn select_description(dom: &scraper::Html) -> String {
    let description_selector = scraper::Selector::parse("blockquote.abstract").unwrap();
    dom.select(&description_selector)
        .next()
        .map(|el| {
            el.text()
                .collect::<String>()
                .trim()
                .trim_start_matches("Abstract:")
                .trim_start()
                .replace('\n', " ")
                .to_string()
        })
        .unwrap_or_default()
}

fn select_authors(dom: &scraper::Html) -> Result<Vec<models::NewAuthor>> {
    let authors_selector = scraper::Selector::parse(".authors > a").unwrap();
    let authors_elements = dom.select(&authors_selector).collect::<Vec<_>>();
    let authors = authors_elements
        .iter()
        .map(|a| models::NewAuthor {
            name: a.text().collect::<String>(),
        })
        .collect::<Vec<_>>();

    Ok(authors)
}

fn select_subjects(dom: &scraper::Html) -> Result<Vec<models::NewSubject>> {
    let subjects_selector = scraper::Selector::parse("td.subjects").unwrap();
    let subjects = dom
        .select(&subjects_selector)
        .next()
        .map(|s| {
            s.text()
                .collect::<String>()
                .split(';')
                .map(|x| models::NewSubject {
                    name: x.trim().to_string(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(subjects)
}

// NOTE: take metrics
fn body_from_pdf(bytes: &glib::Bytes) -> String {
    let mut body = String::new();
    if let Ok(pdf) = poppler::Document::from_bytes(bytes, None) {
        let n = pdf.n_pages();
        for i in 0..n {
            if let Some(text) = pdf.page(i).and_then(|page| page.text()) {
                body.push_str(text.as_str());
                body.push(' ');
            }
        }
    }

    fix_line_breaks(body)
}

fn fix_line_breaks(text: String) -> String {
    let rg = regex::Regex::new(r"(\w)-\n(\w)").unwrap(); // TODO: handle spaces
    rg.replace_all(&text, "$1$2").to_string()
}
