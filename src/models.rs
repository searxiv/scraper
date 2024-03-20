#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct NewTask {
    pub submission_date: chrono::NaiveDate,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NewAuthor {
    pub name: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NewSubject {
    pub name: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NewPaperFull {
    pub arxiv_id: String,
    pub title: String,
    pub description: String,
    pub submission_date: chrono::NaiveDate,
    pub body: String,
    pub authors: Vec<NewAuthor>,
    pub subjects: Vec<NewSubject>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct TaskSubmission {
    pub submission_date: chrono::NaiveDate,
    pub papers: Vec<NewPaperFull>,
}
