use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub assets_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("clipper");

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("clipper");

        let db_path = data_dir.join("clips.db");
        let assets_dir = data_dir.join("assets");

        // Ensure directories exist
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;
        fs::create_dir_all(&assets_dir)
            .with_context(|| format!("Failed to create assets directory: {:?}", assets_dir))?;
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;

        Ok(Self {
            data_dir,
            db_path,
            assets_dir,
            config_dir,
        })
    }

    /// Custom config for testing or isolated environments
    pub fn custom(base_dir: PathBuf) -> Result<Self> {
        let data_dir = base_dir.join("share").join("clipper");
        let config_dir = base_dir.join("config").join("clipper");
        let db_path = data_dir.join("clips.db");
        let assets_dir = data_dir.join("assets");

        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&assets_dir)?;
        fs::create_dir_all(&config_dir)?;

        Ok(Self {
            data_dir,
            db_path,
            assets_dir,
            config_dir,
        })
    }
}
