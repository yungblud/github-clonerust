use clap::Parser;
use git2::Repository;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::Deserialize;
use dialoguer::Input;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    org: String,

    #[arg(short, long)]
    token: Option<String>,
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

    if let Some(token) = &args.token {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("token {}", token).parse().unwrap(),
        );
    }

    let max_per_page = 100;

    let url = format!("https://api.github.com/orgs/{}/repos?per_page={}&type=all", args.org, max_per_page);
    let client = reqwest::Client::new();
    let repos: Vec<Repo> = client.get(&url).headers(headers).send().await?.json().await?;

    let repo_names: Vec<String> = repos.iter().map(|repo| repo.name.clone()).collect();

    println!("Available repos: {:?}", repo_names);

    let input: String = Input::new()
        .with_prompt("Enter the repo name to clone")
        .interact_text()?;

    if let Some(repo) = repos.iter().find(|repo| repo.name == input) {
        println!("Cloning: {}", repo.name);
        let _ = Repository::clone(&repo.clone_url, &repo.name)?;
    } else {
        println!("!Repo not found");
    }

    Ok(())
}
