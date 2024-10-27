use open as file_open;
use copypasta::ClipboardProvider;
use crate::consts;
use crate::conf::Config;
use crate::game;
use crate::git;
use crate::error_window;
use slint::{ModelRc, VecModel, SharedString};
use std::fs;

slint::include_modules!();

pub fn open(cfg: &Config) -> Result<MainWindow, slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.on_open_game(|version| {
        let cfg = Config::load();

        if !version.is_empty() {
            let reset_cmd = git::restore(&cfg.game_repo_path);

            if let Err(e) = reset_cmd {
                error_window::open_safe(
                    Some("Failed to reset repository".to_string()),
                    Some("Failed to reset repository before switching versions".to_string()),
                    Some(format!("Git: {e}"))
                );
                return;
            }

            let status = git::checkout(&cfg.game_repo_path, &version);

            if let Err(e) = status {
                error_window::open_safe(
                    Some("Failed to switch versions".to_string()),
                    Some(
                        format!("Failed to switch to version '{version}'")
                    ),
                    Some(format!("Git: {e}"))
                );
                return;
            }
        }

        game::run(&cfg);
    });
    main_window.set_selected_version("".into());
    main_window.set_sandbox_path(
        consts::paths::get_sandboxed_save_path()
            .to_string_lossy()
            .to_string()
            .into()
    );
    main_window.set_boxedmino_version(env!("CARGO_PKG_VERSION").into());
    main_window.on_open_link(open_link);
    main_window.on_copy_text(|string| {
        copy_text_handled(string.as_str());
    });
    main_window.on_open_save_dir(|| {
        let path = consts::paths::get_sandboxed_save_path();
        if let Err(err) = file_open::that(&path) {
            error_window::open_safe(
                Some("Failed to open save directory".to_string()),
                Some(format!("Path: {:#?}", path)),
                Some(format!("Details: {}", err))
            );
        }
    });
    main_window.set_settings(Settings {
        sandboxed: cfg.sandboxed,
        clear_temp_dir: cfg.clear_temp_dir,
        import_save_on_play: cfg.import_save_on_play,
        game_repo_path: cfg.game_repo_path.clone().into(),
        repo_initialized: cfg.repo_initialized,
        use_cold_clear: cfg.use_cold_clear,
    });
    main_window.set_is_wayland_used(is_wayland_session());
    main_window.set_versions(
        ModelRc::new(
            VecModel::from(
                git::tags(&cfg.game_repo_path)
                    .iter()
                    .map(|s| SharedString::from(s))
                    .collect::<Vec<SharedString>>()
            )
        )
    );
    main_window.on_clear_save_dir(clear_temp_dir);
    main_window.on_filter(|arr: ModelRc<SharedString>, search: SharedString| -> ModelRc<SharedString> {
        let search = search.as_str().to_lowercase();
        let filtered = arr.filter(
            move |item| item.as_str().to_lowercase().contains(&search)
        );

        return ModelRc::new(filtered);
    });
    main_window.on_apply_settings(|settings| {
        let config = Config {
            sandboxed: settings.sandboxed,
            clear_temp_dir: settings.clear_temp_dir,
            import_save_on_play: settings.import_save_on_play,
            repo_initialized: settings.repo_initialized,
            game_repo_path: settings.game_repo_path.as_str().to_string(),
            use_gui: true,
            use_cold_clear: settings.use_cold_clear,
        };
        config.save();
    });
    main_window.run()?;

    return Ok(main_window);
}


fn copy_text(text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut ctx = copypasta::ClipboardContext::new()?;
    ctx.set_contents(text.to_string())?;
    return Ok(());
}

fn copy_text_handled(text: &str) {
    if is_wayland_session() {
        println!("Copying text to clipboard is not supported on Wayland.");
    }
    if let Err(error) = copy_text(text) {
        error_window::open_safe(
            None,
            Some("Failed to copy text to clipboard.".to_string()),
            Some(format!("Error: {}", error))
        );
    }
}

fn is_wayland_session() -> bool {
    return std::env::var("XDG_SESSION_TYPE")
        .unwrap_or("".to_string()) == "wayland";
}

fn open_link(url: slint::SharedString) {
    println!("Opening link: {url}");
    open::that(url.as_str()).unwrap_or_else(|_| {
        error_window::open_safe(
            None,
            Some("Failed to open link".to_string()),
            Some(format!("URL: {}", url))
        );
    });
}

// TODO: Deduplicate
fn clear_temp_dir() {
    let path = consts::paths::get_sandboxed_save_path();

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