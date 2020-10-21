# Renom

Renom is a simple program that allows you to rename your Unreal Engine 4 projects. It handles both Blueprint-only and C++ projects, in accordance with the guidelines set forth [here](https://unrealistic.dev/posts/rename-your-project-including-code).

![Screenshot](https://i.imgur.com/efEzpaX.png)

Among other things, Renom:

- Automatically detects original project name
- Updates target, build, config, and source files
- Creates backups of all affected files to prevent data loss
- Accommodates consecutive renames
- Optionally cleans up outdated directories (e.g. _/Intermediate_)

## Usage

You can use Renom either via the binary release or by building from source.

### Binary

Simply download the latest [release](https://github.com/UnrealisticDev/Renom/releases) and start the program (`.exe`).

### Building from Source

Renom is written in Rust. To build it from source, you will first need the [Rust toolchain](https://www.rust-lang.org/tools/install). Don't be scared, there is copious documentation every step of the way.

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

Alternatively, you can use the `install` subcommand, which will pull and build Renom directly from [crates.io](https://crates.io/crates/renom). If the build is successful, you should be able to find the installed executable at *C:/Users/{User}/.cargo/bin/renom.exe*.

```shell
cargo install renom
```
