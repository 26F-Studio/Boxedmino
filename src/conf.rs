use std::fs;
use crate::{dirs::paths, CliInstruction};
use crate::INSTRUCTION;
use crate::slint_types::Settings;
use serde::{Serialize, Deserialize};

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

/// [See definition for flags](crate::CliInstruction::Run::flags)
fn get_cli_config_flags() -> Option<&'static str> {
    let instruction = INSTRUCTION.get()?;

    let instruction = instruction.as_ref()?;

    return match instruction {
        CliInstruction::Run { flags, .. } =>
            Some(flags.as_ref()?.as_str()),
        _ => None
    };
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

        if let Some(flags) = get_cli_config_flags() {
            for byte in flags.trim().chars() {
                match byte {
                    's' => cfg.sandboxed = false,
                    'S' => cfg.sandboxed = true,
                    'c' => cfg.clear_temp_dir = false,
                    'C' => cfg.clear_temp_dir = true,
                    'i' => cfg.import_save_on_play = false,
                    'I' => cfg.import_save_on_play = true,
                    'a' => cfg.use_cold_clear = false,
                    'A' => cfg.use_cold_clear = true,
                    _ => {
                        eprintln!("Invalid config flag: {:?}", byte);
                        std::process::exit(1);
                    }
                }
            }
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

impl From<Settings> for Config {
    fn from(settings: Settings) -> Self {
        Self {
            sandboxed: settings.sandboxed,
            clear_temp_dir: settings.clear_temp_dir,
            import_save_on_play: settings.import_save_on_play,
            repo_initialized: settings.repo_initialized,
            game_repo_path: settings.game_repo_path.as_str().to_string(),
            use_gui: true,
            use_cold_clear: settings.use_cold_clear,
        }
    }
}

impl From<Config> for Settings {
    fn from(cfg: Config) -> Self {
        Self {
            sandboxed: cfg.sandboxed,
            clear_temp_dir: cfg.clear_temp_dir,
            import_save_on_play: cfg.import_save_on_play,
            game_repo_path: cfg.game_repo_path.clone().into(),
            repo_initialized: cfg.repo_initialized,
            use_cold_clear: cfg.use_cold_clear,
        }
    }
}