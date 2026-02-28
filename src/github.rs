use serde::Deserialize;
use reqwest::header::USER_AGENT;

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
    prerelease: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub struct RepoMetadata {
    pub asset: GithubAsset,
    pub version: String,
}

pub async fn find_github_asset(
    owner: &str,
    repo: &str,
    include_prerelease: bool,
    preferred_arch: &str,
) -> Result<RepoMetadata, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);

    let response = client
        .get(&url)
        .header(USER_AGENT, "axe-package-manager")
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!("Repository {}/{} not found", owner, repo));
    }

    let releases: Vec<GithubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if releases.is_empty() {
        return Err(format!("No releases found for {}/{}", owner, repo));
    }

    // Common arch names for x86_64
    let arch_aliases = if preferred_arch == "x86_64" {
        vec!["x86_64", "amd64", "x64"]
    } else if preferred_arch == "aarch64" {
        vec!["aarch64", "arm64"]
    } else {
        vec![preferred_arch]
    };

    for release in releases {
        if !include_prerelease && release.prerelease {
            continue;
        }

        let appimage_assets: Vec<&GithubAsset> = release.assets.iter()
            .filter(|a| a.name.to_lowercase().ends_with(".appimage"))
            .collect();

        if appimage_assets.is_empty() {
            continue;
        }

        // 1. Try to find the exact architecture match
        for arch in &arch_aliases {
            if let Some(asset) = appimage_assets.iter().find(|a| a.name.to_lowercase().contains(arch)) {
                return Ok(RepoMetadata {
                    asset: (*asset).clone(),
                    version: release.tag_name,
                });
            }
        }

        // 2. Fallback: if there's only one appimage asset, maybe it's the one (or we just take the first one found)
        // But the user complained about aarch64, so better to be strict or at least warn.
        // For now, let's take the first one if we can't find a match, but we already have a better way now.
    }

    Err(format!(
        "No valid AppImage for architecture '{}' found in releases for {}/{}",
        preferred_arch, owner, repo
    ))
}
