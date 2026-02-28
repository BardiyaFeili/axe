use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use target_lexicon::{Architecture, Triple};

pub struct AxePaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub bin_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub packages: HashMap<String, PackageEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "source_type", rename_all = "lowercase")]
pub enum Source {
    Github {
        owner: String,
        repo: String,
        prerelease: bool,
    },
    Direct,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub url: String,
    pub hash: String,
    pub path: PathBuf,
    #[serde(flatten)]
    pub source: Source,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub arch: String,
    pub github_api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let triple = Triple::host();
        let arch = match triple.architecture {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64(_) => "aarch64",
            _ => "x86_64",
        }.to_string();

        Self {
            arch,
            github_api_key: None,
        }
    }
}

impl AxePaths {
    pub fn new() -> Result<Self, String> {
        let proj_dirs = ProjectDirs::from("", "", "axe")
            .ok_or("Could not determine project directories")?;

        let config_dir = proj_dirs.config_dir().to_path_buf();
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let bin_dir = data_dir.join("bin");

        Ok(Self {
            config_dir,
            data_dir,
            bin_dir,
        })
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        fs::create_dir_all(&self.bin_dir)
            .map_err(|e| format!("Failed to create bin dir: {}", e))?;
        Ok(())
    }

    pub fn lockfile_path(&self) -> PathBuf {
        self.config_dir.join("axe.lock")
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("axe.toml")
    }

    pub fn load_lockfile(&self) -> Result<Lockfile, String> {
        let path = self.lockfile_path();
        if !path.exists() {
            return Ok(Lockfile::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        toml::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save_lockfile(&self, lockfile: &Lockfile) -> Result<(), String> {
        let path = self.lockfile_path();
        let content = toml::to_string_pretty(lockfile).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn load_config(&self) -> Result<Config, String> {
        let path = self.config_path();
        if !path.exists() {
            let config = Config::default();
            self.save_config(&config)?;
            return Ok(config);
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        toml::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save_config(&self, config: &Config) -> Result<(), String> {
        let path = self.config_path();
        let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
}
