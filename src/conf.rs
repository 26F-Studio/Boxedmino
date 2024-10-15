use std::fs;
use crate::consts::paths;
use serde::{Serialize, Deserialize};

slint::include_modules!();

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    // Editing this struct?
    // Don't forget to also update the Slint model!
    // See the `Settings` struct in /ui/main.slint

    pub sandboxed: bool,
    pub clear_temp_dir: bool,
    pub import_save_on_play: bool,
    pub repo_initialized: bool,
    pub game_repo_path: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            sandboxed: true,
            clear_temp_dir: true,
            import_save_on_play: false,
            repo_initialized: false,
            game_repo_path: "".to_string(),
        }
    }
    pub fn load() -> Self {
        let config_path = paths::get_config_path();
        let config = fs::read_to_string(config_path);
        match config {
            Ok(config) => {
                let config: Config = serde_json::from_str(&config).unwrap();
                return config;
            }
            Err(_) => {
                let config = Config::new();
                config.save();
                return config;
            }
        }
    }
    pub fn save(&self) {
        let config_path = paths::get_config_path();
        let config = serde_json::to_string(self)
            .expect("Failed to serialize config");

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .expect(format!(
                    "Failed to create directory {}",
                    parent.to_str().unwrap_or("(invalid path)")
                ).as_str());
        }

        fs::write(&config_path, config)
            .expect(format!(
                "Failed to write config to {}",
                config_path.to_str().unwrap_or("(invalid path)")
            ).as_str());
    }
}

impl Default for Config {
    fn default() -> Self {
        return Self::new();
    }
}