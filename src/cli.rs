use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "axe",
    about = "A CLI appimage manager",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a package
    Add(AddArgs),
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Optional override for package name
    #[arg(long)]
    pub name: Option<String>,

    #[command(subcommand)]
    pub source: AddSource,
}

#[derive(Subcommand, Debug)]
pub enum AddSource {
    /// Add from github repository
    Github(GithubArgs),
}

#[derive(Args, Debug)]
pub struct GithubArgs {
    /// Repository in form: owner/repo
    pub repo: RepoRef,

    /// Include pre-releases
    #[arg(long)]
    pub prerelease: bool,
}

#[derive(Debug, Clone)]
pub struct RepoRef {
    pub owner: String,
    pub repo: String,
}

impl std::str::FromStr for RepoRef {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = input.split('/').collect();

        if parts.len() != 2 {
            return Err("Repository must be in form: owner/repo".into());
        }

        let owner = parts[0].trim();
        let repo = parts[1].trim();

        if owner.is_empty() || repo.is_empty() {
            return Err("Owner and repo cannot be empty".into());
        }

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
