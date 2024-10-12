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

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    println!("╔═════╗");
    println!("║ ▄▄  ║  Boxedmino");
    println!("║  ▀▀ ║  Sandboxed Techmino runner");
    println!("╚═════╝");
    println!("2024 - 26F-Studio | https://github.com/26F-Studio/Boxedmino\n\n");

    // TODO: check for command line arguments

    open_window()?;

    return Ok(());
}

fn open_window() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.run()?;

    return Ok(());
}

/*
Plan:
- Autodetect platform and get temp directory
    - Windows: %TEMP%
    - *nix: /tmp
- UI using Slint
    - Some sorta dropdown for git tags?
    - Some sorta "Browse..." button for temp directory?
    - A button to run the game
- Run `git` to switch versions (don't forget to check that git exists)
- RUn `love` to run the game (don't forget to check that love exists)
 */