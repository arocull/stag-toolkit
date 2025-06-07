# Stag Toolkit

[Godot](https://godotengine.org/) addon for real-time 3D games, art, and simulations.
Also usable as a [Rust library](#as-a-library) for non-Godot development.

Currently equipped for Godot **4.4**+ on Windows and Linux.

> [!WARNING]
> Areas of this addon are highly experimental and frequently subject to change based on personal needs.
> These updates will sometimes require usage to be updated accordingly.
> Use at your own risk!
> Check for `@experimental` in docs to help identify unstable features.

*If this tool has helped you, feel free to [send a Kofi](https://ko-fi.com/stagmath) my way, or to anyone listed in the [credits](#credits)*!

## Feature List

- **[StagTest](docs/tools/stagtest.md)** framework for simulating gameplay and performing benchmarks
- **[IslandBuilder](https://alanocull.com/island_builder.html)** tool and corresponding Rust backend (Rust required)
- **SimulatedRope** and **SimulatedRopeBinding** nodes for performing and interacting with rope simulations (Rust required)
- **QueueFloat** object for handling and analyzing large sets of numbers (Rust required)
- [**Texture/Material** importer](docs/tools/ironpress.md) for `.ironpress` files
- "Simple LOD" importer for meshes/scenes with custom LODs
- Shader includes and debug shaders

Some features can be toggled on/off via the Project Settings under `addons/stag_toolkit`. May require an plugin reload or editor restart.

### In Repository, but Not Maintained
- Island shader variants and textures used in Abyss
- Prototype animation classes

### Used In

- [Abyss](https://stagmath.itch.io/abyss-demo), an action platformer game (in development)

## Installation

The latest stable plugin versions are published in the [releases tab](https://github.com/arocull/stag-toolkit/releases). Download and extract the zip archive, and copy the `addons/stag_toolkit` directory into your project's `addons` directory.

### As a Library

The `stag-toolkit` crate can also be used as a library for your own Rust projects, with the option to disable Godot-related features. In your `Cargo.toml`, add:

```toml
[dependencies]
stag-toolkit = { version = "0.4.0", git = "https://github.com/arocull/stag-toolkit", default-features = false }
```

This crate is not officially published to `crates.io` yet, but may be in the future.

## Documentation

For the most up-to-date API documentation, use Godot's internal "Search Help" feature (hotkey F1).

You can read manually-updated [docs](docs/) online too, or see example usage on my [website](https://alanocull.com/).

![](docs/images/godot-internal-docs.png)

## Building Manually

1. Ensure [Godot](https://godotengine.org/download/archive/) (using version as specified above) and [Rust](https://www.rust-lang.org/) (I use the `stable` version) are installed
2. Clone this repository
3. `cd` into this repository and run `cargo fetch`
4. Run `$ make` if on Linux, or `$ build.cmd` if on Windows
5. Open the addon project in Godot to verify that it works: `cd godot/ && godot project.godot`
6. Copy the `godot/addons/stag_toolkit/` directory into your project as `addons/stag_toolkit/`

If desired, you can run `$ make derust` on Linux to remove any Rust-dependent stuff from the addon.

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
