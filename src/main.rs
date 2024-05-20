mod ai_funcs;
mod git_funcs;
mod utils;

use clap::Parser;
use colored::Colorize;
use std::process;
use utils::Args;

fn exit_msg(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}

fn exit_err(message: &str, err: Box<dyn std::error::Error>) -> ! {
    exit_msg(&format!("{} {}", message, err));
}

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
                    exit_err("Failed to analyse code by implementation details", e);
                }
            }
        } else {
            match ai_funcs::code_review(prompt).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    exit_err("Failed to analyse code by code review", e);
                }
            }
        }
    }

    let owner = args
        .owner
        .unwrap_or_else(|| exit_msg("Owner argument is missing. Run with --help for usage."));

    let repo = args
        .repo
        .unwrap_or_else(|| exit_msg("Repo argument is missing. Run with --help for usage."));

    let pr_number = args
        .pr
        .unwrap_or_else(|| exit_msg("PR number argument is missing. Run with --help for usage."));

    println!("Analysing PR changes: {}/{} #{}\n", owner, repo, pr_number);
    match git_funcs::get_pr(&owner, &repo, pr_number).await {
        Ok(pr) => {
            let mut output = String::from(format!("# {}\n\n", pr.info.title));
            utils::append_with_newline(
                &format!("{}\n # Changed Files:\n", pr.info.body),
                &mut output,
            );
            for file in pr.files {
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
                    Err(e) => exit_err("Failed to analyze code", e),
                }
            } else {
                match ai_funcs::code_review(output).await {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => exit_err("Failed to analyze code", e),
                }
            }
        }
        Err(e) => exit_msg(&format!("Failed to get PR info: {}", e).red()),
    }
}
