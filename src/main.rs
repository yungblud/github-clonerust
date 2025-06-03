use std::env;

use clap::Parser;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::Deserialize;
use dialoguer::Input;

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

fn clone_with_auth(url: &str, path: &str, token: &str) -> Result<Repository, git2::Error> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("oauth2", token)
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    builder.clone(url, std::path::Path::new(path))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut github_token: String;
    match env::var("GITHUB_TOKEN") {
        Ok(token) => {
            github_token = token;
        },
        Err(_) => {
            let token_input: String = Input::new()
            .with_prompt("Enter the token")
            .interact_text()?;
            // TODO: ~/.zshrc에 덮어쓰는 것으로 변경
            unsafe {
                env::set_var("GITHUB_TOKEN", &token_input);
            }
            github_token = token_input;
        }
    };

    let args = Args::parse();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "rust-cli/0.1".parse().unwrap());

    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("token {}", github_token).parse().unwrap(),
    );

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
        // TODO: token 관련 unwrap이 실제 Option 타입과 맞지 않음
        clone_with_auth(&repo.clone_url, &repo.name, &github_token)?;
    } else {
        println!("!Repo not found");
    }

    Ok(())
}
