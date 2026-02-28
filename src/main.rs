use crate::cli::{Commands, parse_args};
use crate::config::AxePaths;

mod cli;
mod commands;
mod config;
mod download;
mod github;

#[tokio::main]
async fn main() {
    let paths = AxePaths::new().expect("Failed to initialize paths");
    paths
        .ensure_dirs()
        .expect("Failed to create necessary directories");

    // Load or create config (with architecture detection)
    let config = paths
        .load_config()
        .expect("Failed to load or create config");

    let cli = parse_args();

    match cli.command {
        Commands::Add(a) => commands::handle_add(a, &paths, &config).await,
        Commands::List => commands::handle_list(&paths),
        Commands::Install => commands::handle_install(&paths).await,
        Commands::Run(a) => commands::handle_run(a, &paths).await,
        Commands::Rename(a) => commands::handle_rename(a, &paths),
        Commands::Update(a) => commands::handle_update(a, &paths, &config).await,
        Commands::Remove(a) => commands::handle_remove(a, &paths),
    }
}
