# Stag Toolkit

All-purpose toolkit for Godot game creation.

Currently equipped for Godot **4.2.2**+

> [!WARNING]
> This addon is highly experimental and frequently subject to change based on personal needs.
> These updates will sometimes require usage to be updated accordingly.
> Use at your own risk!

*If this tool has helped you, feel free to [send a Kofi](https://ko-fi.com/stagmath) my way, or to anyone listed in the [credits](#credits)*!

## Feature List

- **[IslandBuilder](https://alanocull.com/island_builder.html)** tool and corresponding Rust backend (Rust required)
- **[StagTest](docs/tools/stagtest.md)** framework for simulating gameplay and performing benchmarks (Rust required for benchmarking)
- **`QueueFloat`** class for handling and analyzing large sets of numbers (Rust required)
- [**Texture/Material** importer](docs/tools/ironpress.md) for `.ironpress` files
- "Simple LOD" importer for meshes/scenes with custom LODs
- Shader includes and debug shaders

Some features can be toggled on/off via the Project Settings under `addons/stag_toolkit`. May require an plugin reload or editor restart.

### In Repository, but Not Maintained
- Island shader variants and textures used in Abyss
- Prototype animation classes

### Used In

- [Abyss](https://stagmath.itch.io/abyss-demo), an action platformer game (in development)

## Documentation

See the [docs](docs/), or see example usage on my [website](https://alanocull.com/).

### General Setup

1. Ensure [Rust](https://www.rust-lang.org/) (I'm using Version 1.82.0) and [Godot](https://godotengine.org/download/archive/) (using version as specified above) are installed
2. Clone this repository
3. `cd` into this repository and run `cargo fetch`

#### Windows

1. Run `$ build.cmd` to build the addon
2. Open the addon project (in `godot/` subdirectory) inside Godot to verify that it works
3. Copy the `godot/addons/stag_toolkit/` directory into your project as `addons/stag_toolkit/`
4. Done!

#### Linux

1. `$ make all` to build the addon
2. Open the addon project in Godot to verify that it works: `cd godot/ && godot project.godot`
3. Copy the `godot/addons/stag_toolkit/` directory into your project as `addons/stag_toolkit/`
4. Done!

### Cross Compiling

Make use Rust's target system!

- Install for key x86_64 platforms: `$ rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu`
- Get a list of [all platforms](https://doc.rust-lang.org/nightly/rustc/platform-support.html): `$ rustup target list`
- Ensure you have the proper linker installed [to make use of cargo](https://stackoverflow.com/a/62853319)

Platforms still working on support for: `x86_64-apple-darwin`

#### On Linux

Install the proper linkers!

- Ubuntu: `$ sudo apt-get install mingw-w64`
- Fedora: `$ sudo dnf install mingw64-gcc`

### Contribution

If making a contribution, please follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard (as best you can).

You may need to install additional toolchains for pre-commit hooks.

- `$ pip install pre-commit gdtoolkit` - Installs pre-commit hooks and a linting/formatting toolchain for GDScript.
- `$ pre-commit install` - Initialize pre-commit hooks.

## Credits

- **[godot-rust](https://godot-rust.github.io/)** is used to hook StagToolkit into Godot Engine
- **[Fast Surface Nets](https://github.com/bonsairobo/fast-surface-nets-rs)** for converting Signed Distance Field data to initial triangle meshes
- **[Godot Engine](https://godotengine.org/)** where the plugin resides

## Disclaimer

This is a mirror of my own private repository.

Issues and feature proposals are tracked there and might not be described here until implementation.
