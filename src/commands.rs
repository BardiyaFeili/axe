use crate::cli::{AddArgs, RemoveArgs, RenameArgs, RunArgs, Source as CliSource, UpdateArgs};
use crate::config::{AxePaths, Config, PackageEntry, Source};
use crate::download;
use crate::github;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

pub async fn handle_add(add_args: AddArgs, paths: &AxePaths, config: &Config) {
    let (suggested_name, meta_version, url, source) = match add_args.source {
        CliSource::Github {
            ref owner,
            ref repo,
        } => {
            println!(
                "Checking repository {}/{} for architecture '{}'...",
                owner, repo, config.arch
            );
            match github::find_github_asset(owner, repo, add_args.prerelease, &config.arch).await {
                Ok(meta) => (
                    repo.clone(),
                    meta.version,
                    meta.asset.browser_download_url,
                    Source::Github {
                        owner: owner.clone(),
                        repo: repo.clone(),
                        prerelease: add_args.prerelease,
                    },
                ),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        CliSource::Url(ref url) => {
            let suggested_name = url
                .split('/')
                .next_back()
                .unwrap_or("appimage")
                .trim_end_matches(".AppImage")
                .trim_end_matches(".appimage");

            (
                suggested_name.to_string(),
                "unknown".to_string(),
                url.clone(),
                Source::Direct,
            )
        }
    };

    let mut lockfile = paths.load_lockfile().unwrap_or_default();

    // Check if the source is already tracked under ANY name
    let existing_name = lockfile
        .packages
        .values()
        .find(|p| match (&p.source, &source) {
            (
                Source::Github {
                    owner: o1,
                    repo: r1,
                    ..
                },
                Source::Github {
                    owner: o2,
                    repo: r2,
                    ..
                },
            ) => o1.to_lowercase() == o2.to_lowercase() && r1.to_lowercase() == r2.to_lowercase(),
            (Source::Direct, Source::Direct) => p.url == url,
            _ => false,
        })
        .map(|p| p.name.clone());

    let name = if let Some(n) = add_args.name {
        n
    } else if let Some(ref ename) = existing_name {
        ename.clone()
    } else if add_args.yes {
        suggested_name
    } else {
        print!("Package name [default: {}]: ", suggested_name);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            suggested_name
        } else {
            input.to_string()
        }
    };

    if let Some(existing) = lockfile.packages.get(&name) {
        if existing.version == meta_version {
            println!("{} version {} is already installed.", name, meta_version);
            return;
        }
        println!("Updating {} to version {}...", name, meta_version);
    }

    let file_name = url.split('/').next_back().unwrap_or(&name);
    let dest = paths.bin_dir.join(file_name);

    let hash = if dest.exists() {
        println!("Binary already exists at {:?}. Calculating hash...", dest);
        match download::calculate_hash(&dest) {
            Ok(h) => {
                download::set_executable(&dest).expect("Failed to set executable permissions");
                h
            }
            Err(e) => {
                eprintln!("Failed to calculate hash: {}. Re-downloading...", e);
                download::download_file(&url, dest.clone(), &name)
                    .await
                    .expect("Failed to download")
            }
        }
    } else {
                println!("Downloading {}...", name);
                download::download_file(&url, dest.clone(), &name)
                    .await
                    .expect("Failed to download")
            };
        
            let should_create_desktop = if add_args.yes || add_args.desktop {
                true
            } else {
                print!("Create a desktop entry for {}? [Y/n]: ", name);
                io::stdout().flush().unwrap();
                let mut response = String::new();
                io::stdin().read_line(&mut response).unwrap();
                let response = response.trim().to_lowercase();
                response.is_empty() || response == "y" || response == "yes"
            };
        
            let desktop_file = if should_create_desktop {
                match create_desktop_file(&name, &dest, paths) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        eprintln!("Warning: Failed to create desktop file: {}", e);
                        None
                    }
                }
            } else {
                None
            };
        
            lockfile.packages.insert(
                name.clone(),
                PackageEntry {
                    name: name.clone(),
                    version: meta_version,
                    url,
                    hash,
                    path: dest,
                    desktop_file,
                    source,
                },
            );
            paths
                .save_lockfile(&lockfile)
                .expect("Failed to save lockfile");
            println!("Successfully installed {}!", name);
        }
        
        fn create_desktop_file(
            name: &str,
            exec_path: &std::path::Path,
            paths: &AxePaths,
        ) -> Result<std::path::PathBuf, String> {
            let desktop_path = paths.applications_dir.join(format!("{}.desktop", name));
        
            let content = format!(
                "[Desktop Entry]\nType=Application\nName={}\nExec={}\nIcon=utilities-terminal\nTerminal=false\nCategories=Utility;\n",
                name,
                exec_path.to_string_lossy(),
            );
        
            fs::write(&desktop_path, content).map_err(|e| format!("Failed to write desktop file: {}", e))?;
            Ok(desktop_path)
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
        // 1. Check/Install binary
        if !pkg.path.exists() {
            println!("Installing missing binary: {}...", name);
            match download::download_file(&pkg.url, pkg.path.clone(), &name).await {
                Ok(_) => println!("Successfully installed binary for {}!", name),
                Err(e) => eprintln!("Failed to install binary for {}: {}", name, e),
            }
        }

        // 2. Check/Restore desktop file
        if let Some(_desktop_path) = pkg.desktop_file.as_ref().filter(|p| !p.exists()) {
            println!("Restoring desktop entry for {}...", name);
            if let Err(e) = create_desktop_file(&name, &pkg.path, paths) {
                eprintln!(
                    "Warning: Failed to restore desktop file for {}: {}",
                    name, e
                );
            }
        }
    }
}

pub async fn handle_run(args: RunArgs, paths: &AxePaths) {
    let lockfile = paths.load_lockfile().unwrap_or_default();

    // Case-insensitive lookup
    let pkg = match lockfile
        .packages
        .values()
        .find(|p| p.name.to_lowercase() == args.name.to_lowercase())
    {
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
            print!(
                "Package '{}' is not installed. Download it now? [Y/n]: ",
                args.name
            );
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

pub fn handle_rename(args: RenameArgs, paths: &AxePaths) {
    let mut lockfile = paths.load_lockfile().expect("Failed to load lockfile");

    let old_name_internal = match lockfile
        .packages
        .values()
        .find(|p| p.name.to_lowercase() == args.old_name.to_lowercase())
    {
        Some(p) => p.name.clone(),
        None => {
            eprintln!("Package '{}' not found in lockfile.", args.old_name);
            std::process::exit(1);
        }
    };

    if lockfile.packages.contains_key(&args.new_name) {
        eprintln!("Package '{}' already exists in lockfile.", args.new_name);
        std::process::exit(1);
    }

    let mut pkg = lockfile.packages.remove(&old_name_internal).unwrap();

    // Update desktop file if it exists
    if let Some(old_desktop) = pkg.desktop_file {
        if let Err(e) = fs::remove_file(&old_desktop) {
            eprintln!("Warning: Failed to remove old desktop file: {}", e);
        }

        match create_desktop_file(&args.new_name, &pkg.path, paths) {
            Ok(new_desktop) => pkg.desktop_file = Some(new_desktop),
            Err(e) => {
                eprintln!("Warning: Failed to create new desktop file: {}", e);
                pkg.desktop_file = None;
            }
        }
    }

    pkg.name = args.new_name.clone();
    lockfile.packages.insert(args.new_name.clone(), pkg);

    paths
        .save_lockfile(&lockfile)
        .expect("Failed to save lockfile");
    println!(
        "Successfully renamed '{}' to '{}' in lockfile!",
        args.old_name, args.new_name
    );
}

pub async fn handle_update(args: UpdateArgs, paths: &AxePaths, config: &Config) {
    let mut lockfile = paths.load_lockfile().expect("Failed to load lockfile");
    let mut updated_packages = Vec::new();

    if lockfile.packages.is_empty() {
        println!("No packages tracked in lockfile.");
        return;
    }

    for (name, pkg) in &lockfile.packages {
        match &pkg.source {
            Source::Github {
                owner,
                repo,
                prerelease,
            } => {
                println!("Checking update for {} ({}/{})...", name, owner, repo);
                match github::find_github_asset(owner, repo, *prerelease, &config.arch).await {
                    Ok(meta) => {
                        if meta.version != pkg.version {
                            println!(
                                "New version found for {}: {} -> {}",
                                name, pkg.version, meta.version
                            );

                            let should_update = if args.yes {
                                true
                            } else {
                                print!("Update {} to {}? [Y/n]: ", name, meta.version);
                                io::stdout().flush().unwrap();
                                let mut input = String::new();
                                io::stdin().read_line(&mut input).unwrap();
                                let input = input.trim().to_lowercase();
                                input.is_empty() || input == "y" || input == "yes"
                            };

                            if should_update {
                                updated_packages.push((
                                    name.clone(),
                                    meta.version,
                                    meta.asset.browser_download_url,
                                ));
                            }
                        } else {
                            println!("{} is already up to date ({}).", name, pkg.version);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to check updates for {}: {}", name, e);
                    }
                }
            }
            Source::Direct => {
                println!("Skipping update check for {} (Direct URL source).", name);
            }
        }
    }

    for (name, new_version, new_url) in updated_packages {
        println!("Updating {} to {}...", name, new_version);

        let pkg = lockfile.packages.get(&name).unwrap();
        let old_path = pkg.path.clone();

        let file_name = new_url.split('/').next_back().unwrap_or(&name);
        let new_dest = paths.bin_dir.join(file_name);

        match download::download_file(&new_url, new_dest.clone(), &name).await {
            Ok(hash) => {
                // Remove old file if it's different from the new one
                if old_path.exists() && old_path != new_dest {
                    let _ = fs::remove_file(&old_path);
                }

                // Update lockfile entry
                let pkg_entry = lockfile.packages.get_mut(&name).unwrap();

                pkg_entry.version = new_version;
                pkg_entry.url = new_url;
                pkg_entry.hash = hash;
                pkg_entry.path = new_dest.clone();

                // Update desktop file if it exists
                if pkg_entry.desktop_file.is_some() {
                    let _ = create_desktop_file(&name, &new_dest, paths);
                }

                println!("Successfully updated {}!", name);
            }
            Err(e) => {
                eprintln!("Failed to update {}: {}", name, e);
            }
        }
    }

    paths
        .save_lockfile(&lockfile)
        .expect("Failed to save lockfile");
}

pub fn handle_remove(args: RemoveArgs, paths: &AxePaths) {
    let mut lockfile = paths.load_lockfile().expect("Failed to load lockfile");

    let internal_name = match lockfile
        .packages
        .values()
        .find(|p| p.name.to_lowercase() == args.name.to_lowercase())
    {
        Some(p) => p.name.clone(),
        None => {
            eprintln!("Package '{}' not found in lockfile.", args.name);
            return;
        }
    };

    let should_remove = if args.yes {
        true
    } else {
        print!(
            "Are you sure you want to remove '{}'? [y/N]: ",
            internal_name
        );
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        input == "y" || input == "yes"
    };

    if !should_remove {
        println!("Aborted.");
        return;
    }

    let pkg = lockfile.packages.remove(&internal_name).unwrap();

    // Remove binary
    if pkg.path.exists() {
        let _ = fs::remove_file(&pkg.path);
    }

    // Remove desktop file
    if let Some(desktop_path) = pkg.desktop_file.as_ref().filter(|p| p.exists()) {
        let _ = fs::remove_file(desktop_path);
    }

    paths
        .save_lockfile(&lockfile)
        .expect("Failed to save lockfile");
    println!("Successfully removed '{}'!", args.name);
}
