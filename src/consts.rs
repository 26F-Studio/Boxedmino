pub mod paths {
    #[cfg(target_os = "windows")]
    pub const GAME_SAVE_PATH: &str = "%APPDATA%\\LOVE\\_tmp_boxedmino";
    #[cfg(target_os = "windows")]
    pub const CONFIG_PATH: &str = "%APPDATA%\\Boxedmino";

    #[cfg(target_os = "macos")]
    pub const GAME_SAVE_PATH: &str = "~/Library/Application Support/LOVE/_tmp_boxedmino";
    #[cfg(target_os = "macos")]
    pub const CONFIG_PATH: &str = "~/Library/Application Support/Boxedmino";
    
    #[cfg(target_os = "linux")]
    pub const GAME_SAVE_PATH: &str = "~/.local/share/love/_tmp_boxedmino";
    #[cfg(target_os = "linux")]
    pub const CONFIG_PATH: &str = "~/.config/Boxedmino";
    
    #[cfg(target_os = "android")]
    pub const GAME_SAVE_PATH: &str = "/data/data/org.love2d.android/_tmp_boxedmino";
    #[cfg(target_os = "android")]
    pub const CONFIG_PATH: &str = "/data/data/org.f26_studio.Boxedmino";
    
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_os = "android"
    )))]
    compile_error!("Unsupported operating system: {}", std::env::pub consts::OS);

}
