use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, long_about = "AI Code Review Tool")]
pub struct Args {
    /// The owner of the repo; required if not running locally
    pub owner: Option<String>,
    /// The repo name; required if not running locally
    pub repo: Option<String>,
    /// The PR number as an integer; required if not running locally
    pub pr: Option<u32>,
    /// Run locally inside git repository, comparing HEAD against main (branch named `main` must exist)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub local: bool,
}

pub fn append_with_newline(new_str: &str, buffer: &mut String) {
    buffer.push_str("\n");
    buffer.push_str(new_str);
}
