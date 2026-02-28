use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Write, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use bytes::Bytes;

pub async fn download_file(url: &str, dest: PathBuf, name: &str) -> Result<String, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let total_size = response
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .map_err(|e| e.to_string())?
        .progress_chars("#>-"));
    pb.set_message(name.to_string());

    let mut file = fs::File::create(&dest).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut hasher = Sha256::new();

    while let Some(item) = stream.next().await {
        let chunk: Bytes = item.map_err(|e| format!("Error while downloading: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Failed to write: {}", e))?;
        hasher.update(&chunk);
        
        let new = downloaded + (chunk.len() as u64);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("{} downloaded", name));

    // chmod +x
    set_executable(&dest)?;

    let hash = format!("{:x}", hasher.finalize());
    Ok(hash)
}

pub fn calculate_hash(path: &PathBuf) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn set_executable(path: &PathBuf) -> Result<(), String> {
    let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    Ok(())
}
