use crate::cli::{Commands, parse_args};
use crate::config::AxePaths;

mod cli;
mod config;
mod github;
mod download;
mod commands;

#[tokio::main]
async fn main() {
    let paths = AxePaths::new().expect("Failed to initialize paths");
    paths.ensure_dirs().expect("Failed to create necessary directories");
    
    // Load or create config (with architecture detection)
    let config = paths.load_config().expect("Failed to load or create config");

    let cli = parse_args();

    match cli.command {
        Commands::Add(a) => commands::handle_add(a, &paths, &config).await,
        Commands::List => commands::handle_list(&paths),
        Commands::Install => commands::handle_install(&paths).await,
        Commands::Run(a) => commands::handle_run(a, &paths).await,
    }
}
