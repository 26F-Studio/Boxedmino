pub mod paths {
    // #[cfg(target_os = "windows")]
    // pub const GAME_SAVE_PATH: &str = "%APPDATA%\\LOVE\\_tmp_boxedmino";
    // #[cfg(target_os = "windows")]
    // pub const CONFIG_PATH: &str = "%APPDATA%\\Boxedmino";

    // #[cfg(target_os = "macos")]
    // pub const GAME_SAVE_PATH: &str = "~/Library/Application Support/LOVE/_tmp_boxedmino";
    // #[cfg(target_os = "macos")]
    // pub const CONFIG_PATH: &str = "~/Library/Application Support/Boxedmino";
    
    // #[cfg(target_os = "linux")]
    // pub const GAME_SAVE_PATH: &str = "~/.local/share/love/_tmp_boxedmino";
    // #[cfg(target_os = "linux")]
    // pub const CONFIG_PATH: &str = "~/.config/Boxedmino";
    
    // #[cfg(target_os = "android")]
    // pub const GAME_SAVE_PATH: &str = "/data/data/org.love2d.android/_tmp_boxedmino";
    // #[cfg(target_os = "android")]
    // pub const CONFIG_PATH: &str = "/data/data/org.f26_studio.Boxedmino";

    use std::path::PathBuf;
    use home::home_dir;

    pub fn get_game_save_path() -> PathBuf {
        #[cfg(target_os = "windows")] {
            let appdata = std::env::var("APPDATA").expect("AppData directory not found");
            let path = (appdata + "LOVE\\_tmp_boxedmino").as_str();
            return PathBuf::from(path);
        }

        #[cfg(target_os = "macos")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join("Library/Application Support/LOVE/_tmp_boxedmino");
        }

        #[cfg(target_os = "linux")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join(".local/share/love/_tmp_boxedmino");
        }

        #[cfg(target_os = "android")]
        {
            return PathBuf::from("/data/data/org.love2d.android/_tmp_boxedmino");
        }
    }

    pub fn get_config_path() -> PathBuf {
        #[cfg(target_os = "windows")] {
            let appdata = std::env::var("APPDATA").expect("AppData directory not found");
            return PathBuf::from(path)
                .join("Boxedmino.json");
        }

        #[cfg(target_os = "macos")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join("Library/Application Support/Boxedmino.json");
        }

        #[cfg(target_os = "linux")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join(".local/share/Boxedmino.json");
        }

        #[cfg(target_os = "android")]
        {
            return PathBuf::from("/data/data/org.f26_studio.Boxedmino/config.json");
        }
    }
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    compile_error!("Unsupported operating system: {}", std::env::consts::OS);
}
