pub mod paths {
    use std::path::PathBuf;

    #[cfg(not(target_os = "windows"))]
    use home::home_dir;

    pub fn get_conf_dir_path() -> PathBuf {
        #[cfg(target_os = "windows")] {
            let appdata = std::env::var("APPDATA").expect("AppData directory not found");
            return PathBuf::from(appdata)
                .join("Boxedmino")
        }

        #[cfg(target_os = "macos")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join("Library/Application Support/Boxedmino");
        }

        #[cfg(target_os = "linux")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join(".local/share/Boxedmino");
        }

        #[cfg(target_os = "android")]
        {
            return PathBuf::from("/data/data/org.f26_studio.Boxedmino");
        }
    }

    pub fn get_sandboxed_save_path() -> PathBuf {
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

    pub fn get_normal_save_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").expect("AppData directory not found");

            // TODO: Check for fused directory

            let path = appdata + "LOVE\\Techmino";
            let path = path.as_str();
            return PathBuf::from(path);
        }

        #[cfg(target_os = "macos")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join("Library/Application Support/LOVE/Techmino");
        }

        #[cfg(target_os = "linux")]
        {
            return home_dir()
                .expect("Could not find home directory")
                .join(".local/share/love/Techmino");
        }

        #[cfg(target_os = "android")]
        {
            // TODO: Check for fused directory
            return PathBuf::from("/data/data/org.love2d.android/Techmino");
        }
    }

    pub fn get_config_path() -> PathBuf {
        return get_conf_dir_path().join("config.json");
    }

    pub fn get_cold_clear_download_path() -> PathBuf {
        return get_conf_dir_path().join("cold_clear.zip");
    }

    pub const COLD_CLEAR_DOWNLOAD_URL: &str =
        if cfg!(target_os = "windows") {
            "https://github.com/26F-Studio/cold_clear_ai_love2d_wrapper/releases/download/11.4.2/Windows.zip"
        } else if cfg!(target_os = "macos") {
            "https://github.com/26F-Studio/cold_clear_ai_love2d_wrapper/releases/download/11.4.2/macOS.zip"
        } else if cfg!(target_os = "linux") {
            "https://github.com/26F-Studio/cold_clear_ai_love2d_wrapper/releases/download/11.4.2/Linux.zip"
        } else if cfg!(target_os = "android") {
            "https://github.com/26F-Studio/cold_clear_ai_love2d_wrapper/releases/download/11.4.2/Android.zip"
        } else if cfg!(target_os = "ios") {
            "https://github.com/26F-Studio/cold_clear_ai_love2d_wrapper/releases/download/11.4.2/iOS.zip"
        } else {
            #[cfg(not(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "linux",
                target_os = "android",
                target_os = "ios"
            )))]
            compile_error!("Unsupported operating system: {}", std::env::consts::OS);

            unreachable!();
        };
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    compile_error!("Unsupported operating system: {}", std::env::consts::OS);
}
