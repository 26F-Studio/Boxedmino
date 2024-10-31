use open as file_open;
use copypasta::ClipboardProvider;
use crate::dirs;
use crate::conf::Config;
use crate::game;
use crate::git;
use crate::error_window;
use crate::slint_types::MainWindow;
use slint::{ModelRc, VecModel, SharedString, ModelExt, ComponentHandle};

fn get_versions(repo_path: &str, include_commits: bool) -> ModelRc<SharedString> {
    let mut versions = git::tags(repo_path)
        .iter()
        .map(|s| SharedString::from(s))
        .collect::<Vec<SharedString>>();

    if include_commits {
        let commits = git::get_commits(repo_path)
            .iter()
            .map(|(hash, name)| format!("[Commit {hash}: {name}]"))
            .map(|s| SharedString::from(s))
            .collect::<Vec<SharedString>>();

        versions.extend(commits);
    }

    return ModelRc::new(
        VecModel::from(versions)
    );
}

/// Gets the commit hash of the formatted commit name.
/// If the name is not in a valid format, the original string will be returned.
fn try_unwrap_version_name(name: &str) -> String {
    if name.starts_with("[Commit ") {
        let hash_end = name.find(':').unwrap_or(name.len());
        return name[8..hash_end].to_string();
    } else {
        return name.to_string();
    }
}

pub fn open(cfg: &Config) -> Result<MainWindow, slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.on_open_game(|version| {
        let cfg = Config::load();

        let version = try_unwrap_version_name(&version);

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
    main_window.set_sandbox_path(
        dirs::paths::get_sandboxed_save_path()
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
        let path = dirs::paths::get_sandboxed_save_path();
        if let Err(err) = file_open::that(&path) {
            error_window::open_safe(
                Some("Failed to open save directory".to_string()),
                Some(format!("Path: {:#?}", path)),
                Some(format!("Details: {}", err))
            );
        }
    });
    main_window.set_settings(cfg.clone().into());
    main_window.set_is_wayland_used(is_wayland_session());
    main_window.set_versions(
        get_versions(&cfg.game_repo_path, false)
    );
    main_window.on_update_version_list(|include_commits| {
        get_versions(Config::load().game_repo_path.as_str(), include_commits)
    });
    main_window.on_clear_save_dir(dirs::clear_temp_dir);
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
