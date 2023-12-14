mod ai_funcs;
mod git_funcs;
mod utils;

use clap::Parser;
use colored::Colorize;
use std::process;
use utils::Args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.local {
        println!("Analysing local changes...\n");
        let diff = git_funcs::get_git_diff_patch()?;
        let prompt = format!("\n{}", diff);

        if args.details {
            match ai_funcs::implementation_details(prompt.clone()).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to analyse code by implementation details: {}", e);
                    process::exit(1);
                }
            }
        } else {
            match ai_funcs::code_review(prompt).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to analyse code by code review: {}", e);
                    process::exit(1);
                }
            }
        }
    }

    let owner = args.owner.unwrap_or_else(|| {
        eprintln!("Owner argument is missing. Run with --help for usage.");
        process::exit(1);
    });

    let repo = args.repo.unwrap_or_else(|| {
        eprintln!("Repo argument is missing. Run with --help for usage.");
        process::exit(1);
    });

    let pr_number = args.pr.unwrap_or_else(|| {
        eprintln!("PR number argument is missing. Run with --help for usage.");
        process::exit(1);
    });

    println!("Analysing PR changes: {}/{} #{}\n", owner, repo, pr_number);
    match git_funcs::get_pr_info(&owner, &repo, pr_number).await {
        Ok(pr_info) => {
            let mut output = String::new();
            for file in pr_info.files {
                utils::append_with_newline(
                    &format!("{} -- {}", &file.filename, &file.status),
                    &mut output,
                );
                utils::append_with_newline(&file.patch, &mut output);
            }

            if args.details {
                match ai_funcs::implementation_details(output).await {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Failed to analyze code: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                match ai_funcs::code_review(output).await {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Failed to analyze code: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}", &format!("Failed to get PR info: {}", e).red());
            process::exit(1);
        }
    }
}
