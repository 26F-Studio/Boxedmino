# Work in Progress
This project is not complete, and many key features are missing. Below are the planned features for the project.

# Boxedmino

A Rust program to select and run old versions of [Techmino](https://github.com/26F-Studio/Techmino), without worrying about savefile corruption. **When playing through this, your progress will not be saved.**

Use cases:
1. Explore the game's history without corrupting your current save files.
2. Verify replays from old versions of the game.
3. Revisit a deleted feature or bug from an old version of the game.
4. Play a specific version of the game for a challenge or speedrun.

This project uses Lua injection to run old versions of [Techmino](https://github.com/26F-Studio/Techmino) in a temporary environment.

## Installation

### External Dependencies

You *will* need the following dependencies to run this program:

- Git: https://git-scm.com/
- Love2D binary: https://love2d.org/

**Make sure that both Git and Love2D are in your PATH.**  
You can check this by trying to run `git` and `love` in your terminal.

### Main Installation

<!-- There are two ways to install this program: -->

<!-- #### Downloading the Binary -->

1. Download the latest release from the [releases page](https://github.com/26F-Studio/Boxedmino/releases).
2. Extract the contents of the archive.
3. Run the executable.

<!-- 
#### Installing through Cargo

If you have Cargo installed, it can be more convenient to install the program through Cargo.
```
cargo install boxedmino
``` -->

## Building

To build the project, you will need to have Rust installed. You can install Rust by following the instructions on [rustup.rs](https://rustup.rs/).

Then, run the following command to build the project:
```
cargo build --release
```

If you want to build and run it, you can use the following command:
```
cargo run --release
```

The `--release` flag is optional and may optimize the program for performance, at the cost of slower compilation times.