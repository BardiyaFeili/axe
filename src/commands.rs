use crate::cli::{AddArgs, RunArgs, Source as CliSource};
use crate::config::{AxePaths, PackageEntry, Config, Source};
use crate::github;
use crate::download;
use std::process::Command;
use std::io::{self, Write};

pub async fn handle_add(add_args: AddArgs, paths: &AxePaths, config: &Config) {
    let (name, meta_version, url, source) = match add_args.source {
        CliSource::Github { ref owner, ref repo } => {
            println!("Checking repository {}/{} for architecture '{}'...", owner, repo, config.arch);
            match github::find_github_asset(owner, repo, add_args.prerelease, &config.arch).await {
                Ok(meta) => {
                    let name = add_args.name.clone().unwrap_or_else(|| repo.clone());
                    (name, meta.version, meta.asset.browser_download_url, Source::Github {
                        owner: owner.clone(),
                        repo: repo.clone(),
                        prerelease: add_args.prerelease,
                    })
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        CliSource::Url(ref url) => {
            let suggested_name = url.split('/').last().unwrap_or("appimage")
                .trim_end_matches(".AppImage")
                .trim_end_matches(".appimage");
            
            let name = if let Some(n) = add_args.name {
                n
            } else if add_args.yes {
                suggested_name.to_string()
            } else {
                print!("Package name [default: {}]: ", suggested_name);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input.is_empty() {
                    suggested_name.to_string()
                } else {
                    input.to_string()
                }
            };
            (name, "unknown".to_string(), url.clone(), Source::Direct)
        }
    };

    let file_name = url.split('/').last().unwrap_or(&name);
    let dest = paths.bin_dir.join(file_name);

    let hash = if dest.exists() {
        let lockfile = paths.load_lockfile().unwrap_or_default();
        if let Some(existing) = lockfile.packages.get(&name) {
            if existing.version == meta_version {
                println!("{} version {} is already in bin at {:?}", name, meta_version, dest);
                return;
            }
        }
        
        println!("Binary already exists at {:?}. Calculating hash...", dest);
        match download::calculate_hash(&dest) {
            Ok(h) => {
                download::set_executable(&dest).expect("Failed to set executable permissions");
                h
            }
            Err(e) => {
                eprintln!("Failed to calculate hash: {}. Re-downloading...", e);
                download::download_file(&url, dest.clone(), &name).await.expect("Failed to download")
            }
        }
    } else {
        println!("Downloading {}...", name);
        download::download_file(&url, dest.clone(), &name).await.expect("Failed to download")
    };

    let mut lockfile = paths.load_lockfile().unwrap_or_default();
    lockfile.packages.insert(name.clone(), PackageEntry {
        name: name.clone(),
        version: meta_version,
        url,
        hash,
        path: dest,
        source,
    });
    paths.save_lockfile(&lockfile).expect("Failed to save lockfile");
    println!("Successfully installed {}!", name);
}

pub fn handle_list(paths: &AxePaths) {
    let lockfile = paths.load_lockfile().unwrap_or_default();
    if lockfile.packages.is_empty() {
        println!("No packages tracked in lockfile.");
        return;
    }

    println!("{:<30} {:<25} {:<15}", "NAME", "VERSION", "STATUS");
    println!("{}", "-".repeat(70));

    for (name, pkg) in lockfile.packages {
        let status = if pkg.path.exists() {
            "Installed"
        } else {
            "Missing"
        };
        println!("{:<30} {:<25} {:<15}", name, pkg.version, status);
    }
}

pub async fn handle_install(paths: &AxePaths) {
    let lockfile = paths.load_lockfile().unwrap_or_default();
    if lockfile.packages.is_empty() {
        println!("Nothing to install.");
        return;
    }

    for (name, pkg) in lockfile.packages {
        if !pkg.path.exists() {
            println!("Installing missing package: {}...", name);
            match download::download_file(&pkg.url, pkg.path.clone(), &name).await {
                Ok(_) => println!("Successfully installed {}!", name),
                Err(e) => eprintln!("Failed to install {}: {}", name, e),
            }
        } else {
            println!("Package {} is already installed.", name);
        }
    }
}

pub async fn handle_run(args: RunArgs, paths: &AxePaths) {
    let lockfile = paths.load_lockfile().unwrap_or_default();
    let pkg = match lockfile.packages.get(&args.name) {
        Some(p) => p,
        None => {
            eprintln!("Package '{}' not found in lockfile.", args.name);
            std::process::exit(1);
        }
    };

    if !pkg.path.exists() {
        let should_download = if args.yes {
            true
        } else {
            print!("Package '{}' is not installed. Download it now? [Y/n]: ", args.name);
            io::stdout().flush().unwrap();
            
            let mut response = String::new();
            io::stdin().read_line(&mut response).unwrap();
            let response = response.trim().to_lowercase();
            response.is_empty() || response == "y" || response == "yes"
        };

        if should_download {
            println!("Installing {}...", args.name);
            match download::download_file(&pkg.url, pkg.path.clone(), &args.name).await {
                Ok(_) => println!("Successfully installed {}!", args.name),
                Err(e) => {
                    eprintln!("Failed to install {}: {}", args.name, e);
                    std::process::exit(1);
                }
            }
        } else {
            println!("Aborted.");
            std::process::exit(1);
        }
    }

    let mut cmd = Command::new(&pkg.path);
    cmd.args(&args.args);

    match cmd.status() {
        Ok(status) => {
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Failed to run AppImage: {}", e);
            std::process::exit(1);
        }
    }
}
