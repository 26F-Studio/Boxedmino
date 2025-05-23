use std::fs;

pub fn clear_temp_dir() {
    let path = crate::dirs::paths::get_sandboxed_save_path();

    println!("Dangerous operation: Clearing temporary directory at {}", path.to_string_lossy());

    if !path.exists() {
        return;
    }

    let entries = fs::read_dir(path);

    if let Err(_) = entries {
        return;
    }

    let entries = entries.unwrap();

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .expect(
                    format!(
                        "Failed to remove directory {}",
                        path.to_string_lossy()
                    ).as_str()
                )
        } else {
            fs::remove_file(&path)
                .expect(
                    format!(
                        "Failed to remove file {}",
                        path.to_string_lossy()
                    ).as_str()
                )
        }
    }

    println!("Cleared temporary directory");
}


pub fn is_dir_empty(path: &str) -> bool {
    let files = fs::read_dir(path);
    if let Err(_) = files {
        return false;
    }

    let files = files.unwrap();
    return files.count() == 0;
}

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

    pub fn get_cold_clear_download_path(version: &str) -> PathBuf {
        return get_conf_dir_path()
            .join("cold_clear")
            .join(version.to_string() + ".zip");
    }

    pub fn get_cold_clear_download_url(version: &str) -> String {
        let file_name = match std::env::consts::OS {
            "windows" => "Windows.zip",
            "macos" => "macOS.zip",
            "linux" => "Linux.zip",
            "android" => "Android.zip",
            "ios" => "iOS.zip",
            _ => unreachable!()
        };

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "android",
            target_os = "ios"
        )))]
        compile_error!("Unsupported operating system: {}", std::env::consts::OS);

        return format!(
            "https://github.com/26F-Studio/cold_clear_ai_love2d_wrapper/releases/download/{version}/{file_name}"
        );
    }

    pub const COLD_CLEAR_RELEASES_API_URL: &str =
        "https://api.github.com/repos/26F-Studio/cold_clear_ai_love2d_wrapper/releases";
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    compile_error!("Unsupported operating system: {}", std::env::consts::OS);
}
