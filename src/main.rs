use clap::Parser;
use git2::Repository;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::Deserialize;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    org: String,
}

#[derive(Deserialize)]
struct Repo {
    name: String,
    clone_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "rust-cli/0.1".parse().unwrap());

    let url = format!("https://api.github.com/orgs/{}/repos", args.org);
    let client = reqwest::Client::new();
    let repos: Vec<Repo> = client.get(&url).headers(headers).send().await?.json().await?;

    for repo in repos {
        println!("Cloning: {}", repo.name);
        let _ = Repository::clone(&repo.clone_url, &repo.name)?;
    }

    Ok(())
}
