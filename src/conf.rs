use std::fs;
use crate::consts::paths;
use crate::git;
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
    pub use_gui: bool,
    pub use_cold_clear: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            sandboxed: true,
            clear_temp_dir: true,
            import_save_on_play: false,
            repo_initialized: false,
            game_repo_path: "".to_string(),
            use_gui: true,
            use_cold_clear: true,
        }
    }
    pub fn load_from_file() -> Self {
        let config_path = paths::get_config_path();
        let config = fs::read_to_string(config_path);
        match config {
            Ok(config) => {
                let mut config: Config = serde_json::from_str(&config)
                    .unwrap_or_default();
                config.use_gui = true;
                return config;
            }
            Err(_) => {
                let config = Config::new();
                config.save();
                return config;
            }
        }
    }
    pub fn load() -> Self {
        let mut cfg = Self::load_from_file();

        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            let arg = args[i].as_str();
            match arg {
                // TODO: simplify cli arg processing using the `clap` crate
                // TODO: --use-version <version> to specify version to run
                "--help" => {
                    println!("{}", include_str!("help.txt"));
                    std::process::exit(0);
                }
                "--run" => {
                    cfg.use_gui = false;
                }
                "--sandboxed" => {
                    cfg.sandboxed = true;
                    cfg.use_gui = false;
                }
                "--no-sandbox" => {
                    cfg.sandboxed = false;
                    cfg.use_gui = false;
                }
                "--clear-temp-dir" => {
                    cfg.clear_temp_dir = true;
                    cfg.use_gui = false;
                }
                "--no-clear-temp-dir" => {
                    cfg.clear_temp_dir = false;
                    cfg.use_gui = false;
                }
                "--import-save-on-play" => {
                    cfg.import_save_on_play = true;
                    cfg.use_gui = false;
                }
                "--no-import-save-on-play" => {
                    cfg.import_save_on_play = false;
                    cfg.use_gui = false;
                }
                "--repo-path" => {
                    if i + 1 < args.len() {
                        cfg.game_repo_path = args[i + 1].clone();
                        i += 1;
                    }
                    cfg.use_gui = false;
                }
                "--version" => {
                    println!("Boxedmino v{}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--list-versions" => {
                    let versions = git::tags(&cfg.game_repo_path);
                    println!("Available versions:");
                    for version in versions {
                        println!("- {}", version);
                    }
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                    std::process::exit(1);
                }
            }
            i += 1;
        }

        return cfg;
    }
    pub fn save(&self) {
        let mut config: Self = self.clone();
        config.use_gui = true;
        let config_path = paths::get_config_path();
        let config = serde_json::to_string(&config)
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