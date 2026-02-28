use clap::{Args, Parser, Subcommand};
use std::str::FromStr;

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
    
    /// List all packages in the lockfile
    List,
    
    /// Install all packages defined in the lockfile
    Install,

    /// Run an installed AppImage by name
    Run(RunArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Name of the package to run
    pub name: String,
    
    /// Arguments to pass to the AppImage
    pub args: Vec<String>,

    /// Auto-agree to all prompts
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Source to add from (GitHub repo 'owner/repo' or a URL)
    pub source: Source,

    /// Optional override for package name
    #[arg(long)]
    pub name: Option<String>,

    /// Include pre-releases (for GitHub sources)
    #[arg(long)]
    pub prerelease: bool,

    /// Auto-agree to all prompts
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Debug, Clone)]
pub enum Source {
    Github { owner: String, repo: String },
    Url(String),
}

impl FromStr for Source {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.starts_with("http://") || input.starts_with("https://") {
            if input.contains("github.com") {
                let clean_input = input
                    .trim_end_matches('/')
                    .trim_start_matches("https://github.com/")
                    .trim_start_matches("http://github.com/");
                
                let parts: Vec<&str> = clean_input.split('/').collect();
                if parts.len() == 2 {
                    return Ok(Source::Github {
                        owner: parts[0].to_string(),
                        repo: parts[1].to_string(),
                    });
                }
            }
            return Ok(Source::Url(input.to_string()));
        }

        // Shorthand owner/repo
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() == 2 {
            let owner = parts[0].trim();
            let repo = parts[1].trim();

            if !owner.is_empty() && !repo.is_empty() {
                return Ok(Source::Github {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                });
            }
        }

        Err("Invalid source. Use 'owner/repo' for GitHub or a full URL.".into())
    }
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
