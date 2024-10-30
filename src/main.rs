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
use clap::{Parser, Subcommand};

mod cold_clear;
mod dirs;
mod conf;
mod game;
mod git;
mod main_window;
mod error_window;
mod setup;
mod slint_types;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli { 
    #[command(subcommand)]
    command: Option<CliInstruction>,
}

#[derive(Subcommand, Clone)]
pub enum CliInstruction {
    #[clap(about = "Lists available versions of the game")]
    ListVersions {
        #[arg(short, long)]
        /// Path to the game repository
        repo_path: Option<String>,
    },

    #[clap(about = "Runs the game")]
    Run {
        #[arg(short, long)]
        /// The version of the game to run. Accepts git tags and commit hashes.
        version: Option<String>,

        #[arg(short, long)]
        /// Path to the game's Git repository. It must contain a main.lua file and a .git folder.
        repo_path: Option<String>,

        #[arg(short, long)]
        /// Set configuration flags. Each flag is one character.
        /// A capital letter denotes "on", a lowercase letter denotes "off".
        /// The flags are:
        /// - `S` [Sandbox]: If on, the game will be tricked to save to a temporary directory.
        /// - `C` [Clear]  : If on, the temporary directory will be cleared before running the game.
        /// - `I` [Import] : If on, Boxedmino will try to import your main save to the temporary save directory.
        /// - `A` [AI]     : If on, Techmino's AI (ColdClear) will be enabled.
        flags: Option<String>,
    },
}

fn main() -> Result<(), slint::PlatformError> {
    print_intro();

    let instruction = Cli::parse().command; // TODO: Parse command

    if let Err(missing_dependencies) = check_dependencies() {
        let mut message = "The following dependencies are missing:".to_string();
        for dependency in missing_dependencies {
            message.push_str(&format!("\n- {}", dependency));
        }
        message.push_str("You can find download links in the console output, if you opened this program from the terminal.");
        error_window::open_safe(
            Some("Boxedmino - Startup Error".to_string()),
            Some("Missing Dependencies".to_string()),
            Some(message),
        );
        return Ok(());
    }

    let mut config = conf::Config::load();

    let no_repo = !config.repo_initialized ||
        !git::is_repo_valid(&config.game_repo_path);

    if no_repo {
        setup::run_setup()?;
        config = conf::Config::load();
    }

    if config.use_gui {
        main_window::open(&config)?;
    } else {
        game::run(&config);
    }

    return Ok(());
}

fn print_intro() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("╔═════╗");
    eprintln!("║ ▄▄  ║  Boxedmino v{version}");
    eprintln!("║  ▀▀ ║  Sandboxed Techmino runner");
    eprintln!("╚═════╝");
    eprintln!("2024 - 26F-Studio | https://github.com/26F-Studio/Boxedmino\n\n");
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



#[deprecated(
    note = "safe_todo used - do not forget to implement the feature"
)]
#[allow(dead_code)]
fn safe_todo(feature: Option<&str>) {
    let message =
        if let Some(feature) = feature {
            format!("{} is not implemented yet.", feature)
        } else {
            "This feature is not implemented yet.".to_string()
        };
    
    eprintln!("{}", message);

    error_window::open_safe(
        Some("Boxedmino - Unimplemented".to_string()),
        Some("Unimplemented Feature".to_string()),
        Some(message)
    );
}