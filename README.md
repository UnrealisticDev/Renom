# Renom

Renom is a simple program that allows you to rename your Unreal Engine projects.
It handles both Blueprint-only and C++ projects, in accordance with the
guidelines set forth
[here](https://unrealistic.dev/posts/rename-your-project-including-code).

```shell
> renom wizard

[ Welcome to Renom ]
> Choose a workflow: Rename a project
> Project root directory path: LyraStarterGame
> Provide a new name for the project: SpyroStarterGame
( apply ) set [URL] GameName = SpyroStarterGame in config file LyraStarterGame\Config/DefaultEngine.ini
( apply ) set [/Script/EngineSettings.GeneralProjectSettings] ProjectName = SpyroStarterGame in config file LyraStarterGame\Config/DefaultGame.ini
( apply ) rename file LyraStarterGame\LyraStarterGame.uproject to LyraStarterGame\SpyroStarterGame.uproject
( apply ) rename file LyraStarterGame to SpyroStarterGame

        [ Success ]
        Successfully renamed project LyraStarterGame to SpyroStarterGame.
```

Among other things, Renom:

- Provides workflows to rename projects, plugins, targets, and modules
- Detects project name, targets, modules, and other metadata
- Updates target, build, config, and source files
- Creates backups of all affected files to prevent data loss
- Supports consecutive renames

## Installation

You can install Renom either by downloading the binary release or by using
the Cargo package manager.

### Binary

Simply download the latest release
[here](https://github.com/UnrealisticDev/Renom/releases) and put the executable
(_.exe_) on your system PATH.

### Cargo

Renom is written in Rust, and Cargo is the package manager for Rust. Install the
Rust toolchain, which includes Cargo, by following the instructions
[here](https://www.rust-lang.org/tools/install). Once Cargo is installed, run
the following command to install Renom:

```shell
cargo install renom
```

This will pull and build Renom directly from
[crates.io](https://crates.io/crates/renom). If the build is successful, you
should be able to find the installed executable at
_C:/Users/{User}/.cargo/bin/renom.exe_.

## Usage

Run the following command to see available options:

```shell
renom
```

To start an interactive session, run the following command instead:

```shell
renom wizard
```
