#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct NewTask {
    pub submission_date: chrono::NaiveDate,
}
