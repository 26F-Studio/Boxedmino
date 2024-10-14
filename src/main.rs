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

    let config = conf::Config::load();

    if(!config.repo_initialized) {
        safe_todo(Some("Initializing the game repository"));
    }

    // TODO: check for command line arguments


    open_window()?;

    return Ok(());
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
        Some("Unimplmented Feature".to_string()),
        Some(message)
    );
}

fn open_window() -> Result<MainWindow, slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.on_open_game(|_| {
        // TODO: open game
        safe_todo(Some("Opening the game"));
    });
    main_window.set_sandbox_path(
        consts::paths::GAME_SAVE_PATH.into()
    );
    main_window.on_open_link(open_link);
    main_window.on_copy_text(|string| {
        copy_text_handled(string.as_str());
    });
    main_window.on_open_save_dir(|| {
        open::that(consts::paths::GAME_SAVE_PATH).unwrap_or_else(|err| {
            open_error_window_safe(
                Some("Failed to open save directory".to_string()),
                Some(format!("Path: {}", consts::paths::GAME_SAVE_PATH)),
                Some(format!("Details: {}", err))
            );
        });
    });
    main_window.set_is_wayland_used(is_wayland_session());
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
    main_window.run()?;

    return Ok(main_window);
}

fn open_link(url: slint::SharedString) {
    println!("Opening link: {url}");
    // if webbrowser::open(&url).is_err() {
    //     eprintln!("Failed to open the link: {}", url);
    // }
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