use rand::seq::SliceRandom;
use rand::thread_rng;
use std::error::Error;
use url::Url;

use fake_tfs::{CollectionResult, Project};

fn main() -> Result<(), Box<dyn Error>> {
    let prs_to_post_count = 10;
    let base_url = Url::parse("http://localhost:8000/")?;
    let projects_get_url = base_url.join("tfs/DefaultCollection/_apis/projects")?;
    let pr_post_url = base_url.join("control/pullrequest/")?;

    let client = reqwest::ClientBuilder::new().build()?;
    let projects: CollectionResult<Project> = client.
        get(projects_get_url.as_str()).
        send()?.
        json().
        map_err(|e| format!("Unable to decode JSON as projects: {}", e))?;
    let post_urls = projects
        .value
        .iter()
        .map(|p| pr_post_url.join(&p.name))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    println!("{:?}", post_urls);

    for _ in 0..prs_to_post_count {
        let url = post_urls.choose(&mut thread_rng()).unwrap().as_str();
        client.post(url).send()?;
    }

    println!("{:?}", projects);

    Ok(())
}
