mod paths {
    #[cfg(target_os = "windows")]
    const GAME_SAVE_PATH: &str = "%APPDATA%\\LOVE\\_tmp_boxedmino";
    #[cfg(target_os = "windows")]
    const CONFIG_PATH: &str = "%APPDATA%\\Boxedmino";

    #[cfg(target_os = "macos")]
    const GAME_SAVE_PATH: &str = "~/Library/Application Support/LOVE/_tmp_boxedmino";
    #[cfg(target_os = "macos")]
    const CONFIG_PATH: &str = "~/Library/Application Support/Boxedmino";
    
    #[cfg(target_os = "linux")]
    const GAME_SAVE_PATH: &str = "~/.local/share/love/_tmp_boxedmino";
    #[cfg(target_os = "linux")]
    const CONFIG_PATH: &str = "~/.config/Boxedmino";
    
    #[cfg(target_os = "android")]
    const GAME_SAVE_PATH: &str = "/data/data/org.love2d.android/_tmp_boxedmino";
    #[cfg(target_os = "android")]
    const CONFIG_PATH: &str = "/data/data/org.f26_studio.Boxedmino";
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    compile_error!("Unsupported operating system: {}", std::env::consts::OS);

}
