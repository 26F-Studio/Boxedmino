use crate::conf::Config;
use crate::consts;
use crate::error_window;
use crate::git;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn run(cfg: &Config) {
    let path = PathBuf::from(cfg.game_repo_path.clone());

    if cfg.sandboxed {
        let script = include_str!("injected.lua");
        let main_lua = path.join("conf.lua");
        let mut main_lua_contents = fs::read_to_string(&main_lua)
            .expect("Failed to read Techmino's conf.lua file");
        main_lua_contents = format!("{}\n{}", script, main_lua_contents);
        fs::write(main_lua, main_lua_contents)
            .expect("Failed to write to Techmino's conf.lua file");
    }

    if cfg.clear_temp_dir {
        clear_temp_dir();
    }

    if cfg.import_save_on_play {
        overwrite_temp_dir();
    }

    if cfg.use_cold_clear {
        crate::safe_todo(Some("Importing Cold Clear"));
    }

    let mut command = Command::new("love");
    command.arg(&path);
    
    let status = command.status();

    if let Err(e) = status {
        error_window::open_safe(
            Some("Failed to run game".to_string()),
            Some("An error was yielded from love2d.".to_string()),
            Some(e.to_string())
        );
    }

    git::restore(&cfg.game_repo_path)
        .expect("Failed to restore repository using git");
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

fn copy_dir_all(from: &str, to: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let entries = fs::read_dir(from);

    if let Err(e) = entries {
        return Err(Box::new(e));
    }

    let entries = entries.unwrap();

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let new_path = PathBuf::from(to).join(file_name);
        if path.is_dir() {
            fs::create_dir_all(&new_path)?;
            copy_dir_all(&path.to_string_lossy(), &new_path.to_string_lossy())?;
        } else {
            fs::copy(&path, &new_path)?;
        }
    }

    return Ok(());
}

fn overwrite_temp_dir() {
    let sandboxed_path = consts::paths::get_sandboxed_save_path();
    
    if !is_dir_empty(sandboxed_path.to_str().unwrap()) {
        clear_temp_dir();
    }

    let normal_path = consts::paths::get_normal_save_path();

    if !normal_path.exists() {
        eprintln!("Could not find normal save directory (inferred location: '{}')", normal_path.to_string_lossy());
        return;
    }

    if !sandboxed_path.exists() {
        fs::create_dir_all(&sandboxed_path)
            .expect("Failed to create sandboxed save directory");
    }

    if let Err(e) = copy_dir_all(
        normal_path.to_str().unwrap(),
        sandboxed_path.to_str().unwrap()
    ) {
        error_window::open_safe(
            None,
            Some("Failed to copy save directory".to_string()),
            Some(format!("Error: {}", e))
        );
    }

    println!("Overwritten temporary directory");
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