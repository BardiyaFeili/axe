use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

pub struct AxePaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub applications_dir: PathBuf,
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
    pub desktop_file: Option<PathBuf>,
    #[serde(flatten)]
    pub source: Source,
}

impl AxePaths {
    pub fn new() -> Result<Self, String> {
        let proj_dirs =
            ProjectDirs::from("", "", "axe").ok_or("Could not determine project directories")?;

        let config_dir = proj_dirs.config_dir().to_path_buf();
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let bin_dir = data_dir.join("bin");
        let applications_dir = proj_dirs
            .data_local_dir()
            .parent()
            .unwrap_or(proj_dirs.data_local_dir())
            .join("applications");

        Ok(Self {
            config_dir,
            data_dir,
            bin_dir,
            applications_dir,
        })
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        fs::create_dir_all(&self.bin_dir)
            .map_err(|e| format!("Failed to create bin dir: {}", e))?;
        fs::create_dir_all(&self.applications_dir)
            .map_err(|e| format!("Failed to create applications dir: {}", e))?;
        Ok(())
    }

    pub fn lockfile_path(&self) -> PathBuf {
        self.config_dir.join("axe.lock")
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
}
