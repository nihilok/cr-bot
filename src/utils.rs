use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, long_about = "AI Code Review Tool")]
pub struct Args {
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub pr: Option<u32>,
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub local: bool,
}

pub fn append_with_newline(new_str: &str, buffer: &mut String) {
    buffer.push_str("\n");
    buffer.push_str(new_str);
}
