#![feature(proc_macro_hygiene, decl_macro)]

use rand::distributions::{Alphanumeric, Uniform};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rocket::response::status;
use rocket::{get, post, routes, State};
use rocket_contrib::json::Json;
use serde::Serialize;
use std::error::Error;
use std::sync::{Arc, Mutex};

use fake_tfs::{CollectionResult, Project, PullRequest, Repository, Reviewer};

fn main() {
    let persons = vec![
        Person::new("Bob Bobson", "MEGACROPT\\bb"),
        Person::new("Glod Glodson", "MEGACROPT\\glod"),
        Person::new("Steinur Steinurson", "MEGACROPT\\ss"),
        Person::new("Halgrim Helgrimson", "MEGACROPT\\hh"),
    ];
    let secret_project = Project::new("SecretProject");
    let top_secret_project = Project::new("TopSecretProject");
    let meget_hemmligt_project = Project::new("MegetHemmeligtProjekt");
    let yderst_hemmeligt_projekt = Project::new("YderstHemmeligtProjekt");
    let repos = vec![
        Repository::new("secret-project", &secret_project),
        Repository::new("secret-statemachine-compiler", &secret_project),
        Repository::new("meget-hemmeligt-projekt", &meget_hemmligt_project),
        Repository::new("top-secret-project", &top_secret_project),
        Repository::new("top-secret-service-tool", &top_secret_project),
        Repository::new("yderst-hemmeligt-projekt", &yderst_hemmeligt_projekt),
    ];

    let projects = vec![secret_project, top_secret_project, meget_hemmligt_project, yderst_hemmeligt_projekt];
    let test_data = TestData::new(persons, repos);

    rocket::ignite()
        .mount("/control/", routes![persons, pullrequest])
        .mount("/tfs/DefaultCollection/", routes![projects, pullrequests])
        .manage(Arc::new(Mutex::new(TfsState::new(projects))))
        .manage(Arc::new(Mutex::new(test_data)))
        .launch();
}

#[derive(Serialize, Clone)]
struct Person {
    pub display_name: String,
    pub unique_name: String,
}

impl Person {
    fn new(display_name: &str, unique_name: &str) -> Person {
        Person {
            display_name: display_name.into(),
            unique_name: unique_name.into(),
        }
    }
}

#[derive(Clone)]
struct TestData {
    persons: Vec<Person>,
    repos: Vec<Repository>,
    pullrequest_count: i32,
}

impl TestData {
    fn new(persons: Vec<Person>, repos: Vec<Repository>) -> Self {
        Self {
            persons,
            repos,
            pullrequest_count: 0,
        }
    }
}

fn make_random_pullrequest(
    test_data: &mut TestData,
    repository: &Repository,
) -> Result<PullRequest, Box<dyn Error>> {
    test_data.pullrequest_count += 1;
    let pull_request_id = test_data.pullrequest_count;
    let status = "active".into();
    let creation_date = "8/3 1917".into();
    let title = reqwest::get("http://whatthecommit.com/index.txt")?.text()?;
    let description_length = thread_rng().sample(Uniform::new(20, 500));
    let description = thread_rng()
        .sample_iter(Alphanumeric)
        .take(description_length)
        .collect();
    let reviewer_count = thread_rng().sample(Uniform::new(1, test_data.persons.len()));
    let reviewers = test_data
        .persons
        .choose_multiple(&mut thread_rng(), reviewer_count)
        .cloned()
        .map(|person| Reviewer {
            vote: 0,
            display_name: person.display_name,
            unique_name: person.unique_name,
        })
        .collect();

    let url = "http://fake.fake".into();
    let repository = repository.clone();
    Ok(PullRequest {
        pull_request_id,
        status,
        creation_date,
        title,
        description,
        reviewers,
        url,
        repository,
    })
}

#[get("/persons")]
fn persons(
    test_data: State<Arc<Mutex<TestData>>>,
) -> Result<Json<Vec<Person>>, status::Custom<String>> {
    Ok(Json(test_data.lock().unwrap().persons.clone()))
}

#[post("/pullrequest/<project>")]
fn pullrequest(
    test_data_mutex: State<Arc<Mutex<TestData>>>,
    tfs_state_mutex: State<Arc<Mutex<TfsState>>>,
    project: String,
) -> Result<(), Box<dyn Error>> {
    let mut test_data = test_data_mutex.lock().unwrap();
    let repos: Vec<Repository> = test_data
        .repos
        .iter()
        .filter(|repo| repo.project.name == project)
        .cloned()
        .collect();
    if let Some(repo) = repos.choose(&mut thread_rng()) {
        let pr = make_random_pullrequest(&mut test_data, repo)?;
        let mut tfs_data = tfs_state_mutex.lock().unwrap();
        tfs_data.pullrequests.push(pr);
    }
    Ok(())
}

struct TfsState {
    projects: Vec<Project>,
    pullrequests: Vec<PullRequest>,
}

impl TfsState {
    fn new(projects: Vec<Project>) -> Self {
        Self {
            projects,
            pullrequests: vec![],
        }
    }
}

#[get("/_apis/projects")]
fn projects(tfs_state: State<Arc<Mutex<TfsState>>>) -> Json<CollectionResult<Project>> {
    let value = tfs_state.lock().unwrap().projects.clone();
    let count = value.len();
    Json(CollectionResult{count, value})
}

#[get("/<project>/_apis/git/pullrequests")]
fn pullrequests(project: String, tfs_state: State<Arc<Mutex<TfsState>>>) -> Json<CollectionResult<PullRequest>> {
    let value : Vec<_> = tfs_state
        .lock()
        .unwrap()
        .pullrequests
        .iter()
        .filter(|pr| pr.repository.project.name == project)
        .cloned()
        .collect();
    let count = value.len();
    Json(CollectionResult{count, value})
}
