use crate::consts;
use std::fs;
use std::io;
use std::process::{Command, ExitStatus, Stdio};
use slint::{SharedString};

slint::include_modules!();

pub fn tags(repo_path: &str) -> Vec<String> {
    let mut cmd = Command::new("git");

    cmd.arg("tag")
        .current_dir(repo_path);

    let output = cmd.output()
        .expect("Failed to run command 'git tag' to retrieve version list");

    let output = String::from_utf8(output.stdout)
        .expect("Failed to convert 'git tag' UTF-8 output");

    let versions: Vec<String> = output
        .split("\n")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    return versions;
}

pub fn restore(repo_path: &str) -> io::Result<ExitStatus> {
    return Command::new("git")
        .args(["restore", "."])
        .current_dir(repo_path)
        .status();
}

pub fn checkout(repo_path: &str, version: &str) -> io::Result<ExitStatus> {
    return Command::new("git")
        .args(["checkout", version])
        .current_dir(repo_path)
        .status();
}


pub fn clone(path: SharedString) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

pub fn is_repo_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    if !path.is_dir() {
        return false;
    }

    let files = fs::read_dir(path);

    if let Err(_) = files {
        return false;
    }

    let files = files.unwrap();

    // Check for .git and conf.lua
    let mut has_git = false;
    let mut has_main_lua = false;

    for file in files {
        let file = file.expect("Failed to read file");
        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap_or("");
        if file_name == ".git" {
            has_git = true;
        } else if file_name == "conf.lua" {
            has_main_lua = true;
        }
    }

    return has_git && has_main_lua;
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