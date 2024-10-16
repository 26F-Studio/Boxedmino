pub mod paths {
    use std::path::PathBuf;

    #[cfg(not(target_os = "windows"))]
    use home::home_dir;

    pub fn get_game_save_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").expect("AppData directory not found");
            let path = appdata + "LOVE\\_tmp_boxedmino";
            let path = path.as_str();
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
            return PathBuf::from(appdata)
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
