use colored::Colorize;
use git2::{DiffFormat, DiffOptions, Repository};
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
pub struct File {
    pub filename: String,
    pub patch: String,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct PRInfo {
    pub title: String,
    pub body: String,
}

pub struct PR {
    pub info: PRInfo,
    pub files: Vec<File>,
}

pub fn get_git_diff_patch() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;

    let mut opts = git2::StatusOptions::new();
    opts.include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut warned = false;
    for status in statuses
        .iter()
        .filter(|s| s.status() != git2::Status::CURRENT)
    {
        let message = format!(
            "Warning: uncommitted changes detected in file: {}",
            status.path().unwrap_or("")
        );
        println!("{}", message.red());
        warned = true;
    }

    if warned {
        let message = "These changes will be ignored.\n";
        println!("{}", message.red());
    }

    let main = repo.revparse_single("main")?;
    let main_commit = main.peel_to_commit()?;
    let main_tree = main_commit.tree()?;

    let head = repo.head()?.peel_to_commit()?;
    let head_tree = head.tree()?;

    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&main_tree), Some(&head_tree), Some(&mut opts))?;

    let mut diff_str = String::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => "+",
            '-' => "-",
            ' ' => " ",
            _ => "",
        };
        diff_str.push_str(prefix);
        let content = std::str::from_utf8(line.content()).unwrap_or("");
        diff_str.push_str(content);
        true
    })?;
    Ok(diff_str)
}

pub async fn get_pr(
    owner: &str,
    repo: &str,
    pr_number: u32,
) -> Result<PR, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let pr_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );
    let files_url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/files",
        owner, repo, pr_number
    );

    // Try to get the Bearer token from the environment variable
    let token = env::var("GH_PR_TOKEN");
    let pr_request = client.get(&pr_url).header("User-Agent", "request");
    let files_request = client.get(&files_url).header("User-Agent", "request");

    // If the token exists, add the Authorization header
    let pr_request = match &token {
        Ok(token) => pr_request.header(AUTHORIZATION, format!("Bearer {}", token)),
        Err(_) => pr_request,
    };
    let files_request = match token {
        Ok(token) => files_request.header(AUTHORIZATION, format!("Bearer {}", token)),
        Err(_) => files_request,
    };

    let pr_response = pr_request.send().await?;
    let response = files_request.send().await?;
    if response.status() != StatusCode::OK || pr_response.status() != StatusCode::OK {
        let error_message = if response.status() == StatusCode::NOT_FOUND
            || response.status() == StatusCode::UNAUTHORIZED
        {
            format!("GitHub API request failed with status: {}. \nIf this is a private repo, the 'GH_PR_TOKEN' environment variable must be set.", response.status())
        } else {
            format!(
                "GitHub API request failed with status: {}.",
                response.status()
            )
        };

        return Err(error_message.into());
    }
    let pr_info: PRInfo = pr_response.json().await?;
    let files: Vec<File> = response.json().await?;

    Ok(PR {
        info: pr_info,
        files,
    })
}
