use crate::cli::{AddArgs, AddSource, Commands, GithubArgs, parse_args};

mod cli;

fn main() {
    let cli = parse_args();

    match cli.command {
        Commands::Add(a) => handle_add(a),
    }
}

pub fn handle_add(add_args: AddArgs) {
    match add_args.source {
        AddSource::Github(a) => handle_github(a),
    }
}

pub fn handle_github(args: GithubArgs) {
    let _source = format!("{}/{}", args.repo.owner, args.repo.repo);
    println!("TODO");
}
