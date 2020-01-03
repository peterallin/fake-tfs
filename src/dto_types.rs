use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CollectionResult<T> {
    pub count: usize,
    pub value: Vec<T>,   
}

#[derive(Serialize, Clone)]
pub struct PullRequest {
    pub repository: Repository,
    #[serde(rename = "pullRequestId")]
    pub pull_request_id: i32,
    pub status: String,
    #[serde(rename = "creationDate")]
    pub creation_date: String,
    pub title: String,
    pub description: String,
    pub reviewers: Vec<Reviewer>,
    pub url: String,
}

#[derive(Serialize, Clone)]
pub struct Reviewer {
    pub vote: i32,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "uniqueName")]
    pub unique_name: String,
}

#[derive(Serialize, Clone)]
pub struct Repository {
    pub name: String,
    pub project: Project,
}

impl Repository {
    pub fn new(name: &str, project: &Project) -> Self {
        Self {
            name: name.into(),
            project: project.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Project {
    pub name: String,
}

impl Project {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }
}
