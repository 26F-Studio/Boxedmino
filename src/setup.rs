use crate::error_window;
use crate::git;
use crate::conf;
use rfd::FileDialog;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

slint::include_modules!();

pub fn run_setup() -> Result<(), slint::PlatformError> {
    // Wrap `setup_finished` and `setup_window` in Rc<RefCell> for shared access.
    let setup_finished = Rc::new(RefCell::new(false));
    let setup_window = Rc::new(SetupWindow::new()?);

    // Clone Rc pointers for use in closures
    let window_clone = setup_window.clone();

    setup_window.on_browse_for_repo(|| {
        let path = FileDialog::new()
            .pick_folder()
            .map(|path| path.to_string_lossy().to_string());

        return path.unwrap_or_default().into();
    });

    setup_window.on_change_path(move |path| {
        let valid = git::is_repo_valid(path.as_str());
        window_clone.set_repo_valid(valid);

        let empty = is_dir_empty(path.as_str());
        window_clone.set_dir_empty(empty);
    });

    setup_window.on_clone_repo(|path| {
        if let Err(e) = git::clone(path) {
            error_window::open_safe(
                None,
                Some("Failed to clone repository".to_string()),
                Some(format!("Error: {}", e))
            );
        }
    });

    let window_clone = setup_window.clone();
    let finished_clone = setup_finished.clone();
    setup_window.on_finish(move || {
        *finished_clone.borrow_mut() = true;
        window_clone.as_weak().unwrap().hide().expect(
            "Failed to close setup window"
        );
    });

    setup_window.run()?;

    // Check if setup finished properly
    if *setup_finished.borrow() {
        let mut config = conf::Config::load();
        config.repo_initialized = true;
        config.game_repo_path = setup_window.get_game_repo_path().to_string();
        config.save();
        Ok(())
    } else {
        panic!("Setup closed prematurely");
    }
}

// TODO: Deduplicate
fn is_dir_empty(path: &str) -> bool {
    let files = fs::read_dir(path);
    if let Err(_) = files {
        return false;
    }

    let files = files.unwrap();
    return files.count() == 0;
}