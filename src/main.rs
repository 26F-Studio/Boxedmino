/*
    Boxedmino - Sandboxed Techmino runner
    Copyright (C) 2024 - 26F-Studio

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use conf::Config;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::process::{Command, Stdio};
use slint::{ModelRc, SharedString, VecModel};
use copypasta::ClipboardProvider;
use open;
mod consts;
mod conf;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    println!("╔═════╗");
    println!("║ ▄▄  ║  Boxedmino");
    println!("║  ▀▀ ║  Sandboxed Techmino runner");
    println!("╚═════╝");
    println!("2024 - 26F-Studio | https://github.com/26F-Studio/Boxedmino\n\n");

    if let Err(missing_dependencies) = check_dependencies() {
        let mut message = "The following dependencies are missing:".to_string();
        for dependency in missing_dependencies {
            message.push_str(&format!("\n- {}", dependency));
        }
        message.push_str("You can find download links in the console output, if you opened this program from the terminal.");
        open_error_window(
            Some("Boxedmino - Startup Error".to_string()),
            Some("Missing Dependencies".to_string()),
            Some(message),
        )?;
        return Ok(());
    }

    let mut config = conf::Config::load();

    mutate_config_with_cli_args(&mut config);    
    
    if !config.repo_initialized ||
        !is_repo_valid(&config.game_repo_path)
    {
        run_setup()?;
        config = conf::Config::load();
    }

    if config.use_gui {
        open_window(&config)?;
    } else {
        run_game(&config);
    }

    return Ok(());
}

fn run_game(cfg: &Config) {
    let path = PathBuf::from(cfg.game_repo_path.clone());

    if cfg.sandboxed {
        let script = include_str!("injected.lua");
        let main_lua = path.join("main.lua");
        let mut main_lua_contents = fs::read_to_string(&main_lua)
            .expect("Failed to read Techmino's main.lua file");
        main_lua_contents = format!("{}\n{}", script, main_lua_contents);
        fs::write(main_lua, main_lua_contents)
            .expect("Failed to write to Techmino's main.lua file");
    }

    if cfg.clear_temp_dir {
        clear_temp_dir();
    }

    if cfg.import_save_on_play {
        safe_todo(Some("Importing save on play"));
    }

    let mut command = Command::new("love");
    command.arg(path);
    command.status()
        .expect("Running love2d yielded an error");

    // Restore main.lua
    let mut command = Command::new("git");
    command.arg("restore");
    command.arg("main.lua");
    command.status()
        .expect("Failed to restore main.lua using git");
}

fn mutate_config_with_cli_args(cfg: &mut Config) {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            // TODO: --version for versioning
            // TODO: --list-versions
            "--help" => {
                println!("Note: Running Boxedmino with commandline arguments will immediately");
                println!("      start the game without the Boxedmino UI.\n");
                println!("Arguments");
                println!("--help:                   Show this help message");
                println!("--sandboxed:              Trick the game into saving into a temporary folder");
                println!("--no-sandboxed:           Do not trick the game into saving elsewhere");
                println!("--clear-temp-dir:         Clear the temporary directory before");
                println!("                          running the game");
                println!("--no-clear-temp-dir:      Do not clear the temporary directory");
                println!("                          before running the game");
                println!("--import-save-on-play:    Try to transfer your main save into");
                println!("                          the temporary folder");
                println!("--no-import-save-on-play: Do not transfer your main save into");
                println!("                          the temporary folder");
                std::process::exit(0);
            }
            "--sandboxed" => {
                cfg.sandboxed = true;
                cfg.use_gui = false;
            }
            "--no-sandbox" => {
                cfg.sandboxed = false;
                cfg.use_gui = false;
            }
            "--clear-temp-dir" => {
                cfg.clear_temp_dir = true;
                cfg.use_gui = false;
            }
            "--no-clear-temp-dir" => {
                cfg.clear_temp_dir = false;
                cfg.use_gui = false;
            }
            "--import-save-on-play" => {
                cfg.import_save_on_play = true;
                cfg.use_gui = false;
            }
            "--no-import-save-on-play" => {
                cfg.import_save_on_play = false;
                cfg.use_gui = false;
            }
            "--repo-path" => {
                if i + 1 < args.len() {
                    cfg.game_repo_path = args[i + 1].clone();
                    i += 1;
                }
                cfg.use_gui = false;
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                std::process::exit(1);
            }
        }
        i += 1;
    }
}

fn check_dependencies() -> Result<(), Vec<String>> {
    let mut missing_dependencies: Vec<String> = Vec::new();

    if Command::new("git")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!("{}\n{}\n{}",
            "It seems that Git is not installed on your system.",
            "Install Git from: https://git-scm.com/downloads",
            "Make sure to add Git to your PATH, and that running `git --version` in the terminal works."
        );
        missing_dependencies.push("git".to_string());
    }

    if Command::new("love")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!("{}\n{}\n{}",
            "It seems that LÖVE is not installed on your system.",
            "Install LÖVE from: https://love2d.org/",
            "Make sure to add LÖVE to your PATH, and that running `love --version` in the terminal works."
        );
        missing_dependencies.push("love".to_string());
    }

    if !missing_dependencies.is_empty() {
        return Err(missing_dependencies);
    }

    return Ok(());
}

fn open_error_window(
    title: Option<String>,
    message: Option<String>,
    details: Option<String>,
) -> Result<ErrorWindow, slint::PlatformError> {
    let error_window = ErrorWindow::new()?;
    error_window.set_error_title(
        title.unwrap_or("Error".to_string()).into()
    );
    error_window.set_error_message(
        message.unwrap_or("An error occurred.".to_string()).into()
    );
    error_window.set_error_details(
        details.unwrap_or("No details provided.".to_string()).into()
    );
    let weak = error_window.as_weak();
    error_window.on_dismiss(move || {
        weak.unwrap().window().hide().unwrap();
    });
    error_window.run()?;

    return Ok(error_window);
}

fn open_error_window_safe(
    title: Option<String>,
    message: Option<String>,
    details: Option<String>,
) {
    let result = open_error_window(
        title.clone(),
        message.clone(),
        details.clone()
    );
    if let Err(error) = result {
        eprintln!("Failed to open error window: {}", error);
        eprintln!("Title: {}", &title.unwrap_or("unspecified".into()).to_string());
        eprintln!("Message: {}", &message.unwrap_or("unspecified".into()).to_string());
        eprintln!("Details: {}", &details.unwrap_or("unspecified".into()).to_string());
    }
}

fn safe_todo(feature: Option<&str>) {
    let message =
        if let Some(feature) = feature {
            format!("{} is not implemented yet.", feature)
        } else {
            "This feature is not implemented yet.".to_string()
        };
    
    eprintln!("{}", message);

    let _ = open_error_window(
        Some("Boxedmino - Unimplemented".to_string()),
        Some("Unimplemented Feature".to_string()),
        Some(message)
    );
}

fn open_window(cfg: &Config) -> Result<MainWindow, slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.on_open_game(|_| {
        run_game(&Config::load());
    });
    main_window.set_sandbox_path(
        consts::paths::get_game_save_path()
            .to_string_lossy()
            .to_string()
            .into()
    );
    main_window.on_open_link(open_link);
    main_window.on_copy_text(|string| {
        copy_text_handled(string.as_str());
    });
    main_window.on_open_save_dir(|| {
        let path = consts::paths::get_game_save_path();
        if let Err(err) = open::that(&path) {
            open_error_window_safe(
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
    });
    main_window.set_is_wayland_used(is_wayland_session());
    // TODO: set version list
    main_window.set_versions(
        ModelRc::new(
            VecModel::from(
                vec![
                    SharedString::from("Debug Version"),
                    SharedString::from("Unfinished Version"),
                    SharedString::from("This is unimplemented"),
                    SharedString::from("v1.0.0"),
                ]
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
        };
        config.save();
    });
    main_window.run()?;

    return Ok(main_window);
}

fn is_dir_empty(path: &str) -> bool {
    let files = fs::read_dir(path);
    if let Err(_) = files {
        return false;
    }

    let files = files.unwrap();
    return files.count() == 0;
}

fn clear_temp_dir() {
    let path = consts::paths::get_game_save_path();

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

fn is_repo_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    if !path.is_dir() {
        return false;
    }

    let files = fs::read_dir(path);

    if let Err(_) = files {
        return false;
    }

    let files = files.unwrap();

    // Check for .git and main.lua
    let mut has_git = false;
    let mut has_main_lua = false;

    for file in files {
        let file = file.expect("Failed to read file");
        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap_or("");
        if file_name == ".git" {
            has_git = true;
        } else if file_name == "main.lua" {
            has_main_lua = true;
        }
    }

    return has_git && has_main_lua;
}

fn run_setup() -> Result<(), slint::PlatformError> {
    // Wrap `setup_finished` and `setup_window` in Rc<RefCell> for shared access.
    let setup_finished = Rc::new(RefCell::new(false));
    let setup_window = Rc::new(SetupWindow::new()?);

    setup_window.on_open_link(open_link);
    
    // Clone Rc pointers for use in closures
    let window_clone = setup_window.clone();

    setup_window.on_browse_for_repo(|| {
        let path = FileDialog::new()
            .pick_folder()
            .map(|path| path.to_string_lossy().to_string());

        return path.unwrap_or_default().into();
    });

    setup_window.on_change_path(move |path| {
        let valid = is_repo_valid(path.as_str());
        window_clone.set_repo_valid(valid);

        let empty = is_dir_empty(path.as_str());
        window_clone.set_dir_empty(empty);
    });

    setup_window.on_clone_repo(|path| {
        if let Err(e) = clone_repo(path) {
            open_error_window_safe(
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

const REPO_LINK: &str = "https://github.com/26F-Studio/Techmino.git";

#[cfg(target_os = "windows")]
fn get_terminal_clone_command(path: String) -> Option<Command> {
    let mut cmd = Command::new("cmd");
    cmd.args([
        "/c",
        "git",
        "clone",
        REPO_LINK,
        path.as_str()
    ]);
    return Some(cmd);
}

#[cfg(target_os = "macos")]
fn get_terminal_clone_command(path: String) -> Option<Command> {
    let script_dir = "/tmp/_boxedmino_clone.sh";

    let script_result = fs::write(
        script_dir,
        format!(
            "git clone {} {}; rm {}",
            REPO_LINK,
            path,
            script_dir
        ),
    );

    if let Err(_) = script_result {
        return None;
    }

    let chmod_result =
        Command::new("chmod")
            .args([
                "+x",
                script_dir
            ])
            .status();
    
    if let Err(_) = chmod_result {
        return None;
    }

    let mut command = Command::new("open");
    command.args([
        "-a",
        "/Applications/Utilities/Terminal.app",
        script_dir
    ]);

    return Some(command);
}

#[cfg(target_os = "linux")]
fn get_terminal_clone_command(path: String) -> Option<Command> {
    let popular_term_emus = [
        "x-terminal-emulator",
        "xterm",
        "gnome-terminal",
        "konsole",
        "alacritty",
        "kitty",
        "tilix",
        "terminator",
        "urxvt",
        "rxvt",
        "lxterminal",
        "xfce4-terminal",
        "guake",
        "yakuake",
        "st",
        "Eterm",
        "hyper",
        "qterminal",
        "tilda",
        "wezterm",
        "termux",
        "foot",
        "sakura",
        "mlterm",
        "cool-retro-term",
        "extraterm",
        "termite"
    ];

    let terminal = popular_term_emus.iter().find(|term| {
        Command::new("which")
            .stdout(Stdio::null())
            .arg(term)
            .status()
            .is_ok()
    });

    if let Some(terminal) = terminal {
        let mut cmd = Command::new(terminal);

        cmd.args([
            "-e",
            "git",
            "clone",
            REPO_LINK,
            path.as_str()
        ]);

        return Some(cmd);
    }

    return None;
}

fn get_fallback_clone_command(path: String) -> Command {
    let mut cmd = Command::new("git");
    cmd.args([
        "clone",
        REPO_LINK,
        path.as_str()
    ]);

    return cmd;
}

fn clone_repo(path: SharedString) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut terminal_opened = true;
    let command =
        get_terminal_clone_command(path.to_string().clone());
    
    let mut command = match command {
        Some(c) => c,
        None => {
            terminal_opened = false;
            get_fallback_clone_command(path.to_string().clone())
        }
    };

    if terminal_opened {
        command.status()?;
    } else {
        let window = GitCloneWaitWindow::new();

        if let Err(e) = window {
            return Err(Box::new(e));
        }

        let window = window.unwrap();

        let weak = window.as_weak();

        window.on_dismiss(move || {
            weak.unwrap().window().hide().unwrap();
        });

        let child = command.spawn()
            .expect("Failed to run git clone command");

        // when the command is done, close the window
        let weak = window.as_weak();
        std::thread::spawn(move || {
            child.wait_with_output().expect("Failed to wait for git clone command");
            weak.unwrap().set_finished(true);
            weak.unwrap().window().hide().unwrap();
        });

        window.run()?;
    }

    return Ok(());
}


fn open_link(url: slint::SharedString) {
    println!("Opening link: {url}");
    open::that(url.as_str()).unwrap_or_else(|_| {
        open_error_window_safe(
            None,
            Some("Failed to open link".to_string()),
            Some(format!("URL: {}", url))
        );
    });
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
        open_error_window_safe(
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

/*
Plan:
- Run `git` to switch versions
- RUn `love` to run the game
- Inject Lua to the game to change the save directory
- Clear the save directory before running the game
 */