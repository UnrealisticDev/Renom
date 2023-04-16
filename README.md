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

## Usage

You can use Renom either via the binary release or by building from source.

### Binary

Simply download the latest
[release](https://github.com/UnrealisticDev/Renom/releases) and start the
program (`.exe`).

### Building from Source

Renom is written in Rust. To build it from source, you will first need the [Rust
toolchain](https://www.rust-lang.org/tools/install). Don't be scared, there is
copious documentation every step of the way.

Once you have Rust installed...

1. Clone the repo

```shell
git clone https://github.com/UnrealisticDev/Renom.git
```

2. Navigate into the repo

```shell
cd Renom
```

3. Build and run

```shell
cargo run
```

Alternatively, you can use the `install` subcommand, which will pull and build
Renom directly from [crates.io](https://crates.io/crates/renom). If the build is
successful, you should be able to find the installed executable at
_C:/Users/{User}/.cargo/bin/renom.exe_.

```shell
cargo install renom
```
