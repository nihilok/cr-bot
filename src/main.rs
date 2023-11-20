use std::process;
use reqwest;
use serde::Deserialize;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use clap::{command, arg, Parser};
use std::io::{stdout, Write};
use futures::StreamExt;
use git2::{DiffFormat, Repository, DiffOptions};
use colored::*;

#[derive(Debug, Parser)]
#[command(author, version, long_about = "Dynamic DNS Client")]
pub struct Args {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub pr: Option<u32>,
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub local: bool,
}

const COMPLETION_TOKENS: u16 = 1024;

#[derive(Deserialize, Debug)]
struct FileIn {
    filename: String,
    patch: String,
}

#[derive(Debug)]
struct File {
    filename: String,
    content: String,
    patch: String,
}

struct PRInfo {
    files: Vec<File>,
}


fn get_git_diff_patch() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;

    let mut opts = git2::StatusOptions::new();
    opts.include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut warned = false;
    for status in statuses.iter().filter(|s| s.status() != git2::Status::CURRENT) {
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

async fn code_review(output: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = async_openai::Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(COMPLETION_TOKENS)
        .model("gpt-4-1106-preview")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a code reviewer. You provide your response in markdown, using a heading (`## ...`) for each file reviewed, normal text for your comment, and, potentially, code blocks for code snippets relating to suggested changes (```language...```). You should use line numbers as points of reference.  You should focus only on the changes from the diff; if the rest of the file is included, it is for context only; you can comment on the rest of the file if there are serious implications to be considered; otherwise, this extra file content should be ignored.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("Can you review some changes from a PR?")
                .build()?
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content("Sure thing! What are the changes?")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(output.as_str())
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;

    println!("== START OF COMMENTS ==\n");
    let mut lock = stdout().lock();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        write!(lock, "{}", content).unwrap();
                    }
                });
            }
            Err(err) => {
                writeln!(lock, "error: {err}").unwrap();
            }
        }
        stdout().flush()?;
    }

    Ok(())
}

async fn fetch_file_content(owner: &str, repo: &str, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, file_path);

    let content: String = client.get(&url)
        .header("User-Agent", "request")
        .header("Accept", "application/vnd.github.v3.raw")
        .send().await?
        .text().await?;

    Ok(content)
}

async fn get_pr_info(owner: &str, repo: &str, pr_number: u32) -> Result<PRInfo, Box<dyn std::error::Error>> {

    let client = reqwest::Client::new();
    let pr_url = format!("https://api.github.com/repos/{}/{}/pulls/{}/files", owner, repo, pr_number);

    let files_info: Vec<FileIn> = client.get(&pr_url)
        .header("User-Agent", "request")
        .send().await?
        .json().await?;

    let mut files = Vec::new();

    for file_info in files_info {

        let file_content = fetch_file_content(owner, repo, &file_info.filename).await?;

        files.push(
            File {
                filename: file_info.filename,
                patch: file_info.patch,
                content: file_content
            }
        );
    }

    Ok(PRInfo {
        files,
    })
}

fn append_with_newline(new_str: &str, buffer: &mut String) {
    buffer.push_str("\n");
    buffer.push_str(new_str);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.local {
        println!("Analysing local changes...\n");
        let diff = get_git_diff_patch()?;
        let prompt = format!("\n{}", diff);
        let review_comments = code_review(prompt).await;
        match review_comments {
            Ok(_) => {
                println!("\n\n== END OF COMMENTS ==");
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to analyse code: {}", e);
                process::exit(1);
            }
        }
    }

    if args.owner.is_none() {
        eprintln!("Required positional args not provided. Run with --help for usage.");
        process::exit(1);
    }

    let owner = args.owner.unwrap();

    // Gets a value for repo if supplied by user
    let repo = args.repo.unwrap();

    // Gets a value for pr if supplied by user
    let pr_number = args.pr.unwrap();
    println!("Getting PR data: {}/{} #{}", owner, repo, pr_number);
    match get_pr_info(&owner, &repo, pr_number).await {
        Ok(pr_info) => {
            // print_review_comments(pr_info.number, comments);
            // Iterate over each file in the PR and fetch its content
            let mut output = String::new();
            for file in pr_info.files {
                append_with_newline(&format!("-- CHANGED FILE -- {}", &file.filename), &mut output);
                append_with_newline(&file.content, &mut output);
                append_with_newline(&file.patch, &mut output);
            }
            println!("Analysing changes...\n");
            let review_comments = code_review(output).await;
            match review_comments {
                Ok(_) => {
                    println!("\n\n== END OF COMMENTS ==");
                }
                Err(e) => {
                    eprintln!("Failed to analyse code: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get PR info: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}
