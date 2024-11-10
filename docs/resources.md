# Resources

Abritrary list of resources for working on StagToolkit.

## Rust Resources
- [Setting Up](docs/setup.md)
- [rust book](https://doc.rust-lang.org/stable/book/)
- [godot-rust book](https://godot-rust.github.io/book/)
- [Dieter Rams 10 Principles for Good Design](https://www.vitsoe.com/us/about/good-design)
- [Cross-Compiling Rust for Windows](https://stackoverflow.com/questions/31492799/cross-compile-a-rust-application-from-linux-to-windows)

### Speculative Crates
- [glam](https://docs.rs/glam/0.29.0/glam/) for 3D linear algebra
- [tri_mesh](https://docs.rs/tri-mesh/latest/tri_mesh/)
- [meshopt](https://crates.io/crates/meshopt) Mesh optimization library
- [plexus](https://docs.rs/plexus/0.0.11/plexus/) for 3D geometry
- [plexus mesh graphs](https://plexus.rs/user-guide/graphs/) guide
- [stopwatch2](https://crates.io/crates/stopwatch2)

### Mesh Data
- [Working with node graphs](https://medium.com/@muhamadmehrozkhan/learning-mesh-based-simulation-with-graph-networks-0feddf52adeb)
- [Face winding](https://cmichel.io/understanding-front-faces-winding-order-and-normals)

## Building Godot
- [Guide List](https://docs.godotengine.org/en/stable/contributing/development/compiling/index.html)
- [Building for Linux](https://docs.godotengine.org/en/stable/contributing/development/compiling/compiling_for_linuxbsd.html)
- [Building with Encryption](https://docs.godotengine.org/en/stable/contributing/development/compiling/compiling_with_script_encryption_key.html)
- [Optimizing Build Size](https://docs.godotengine.org/en/stable/contributing/development/compiling/optimizing_for_size.html)
- [Extra Build Options](https://docs.godotengine.org/en/stable/contributing/development/compiling/introduction_to_the_buildsystem.html#doc-overriding-build-options)

### For Linux
Needed packages: `scons strip`

### For Windows
Install...
- [LLVM-MinGW](https://github.com/mstorsjo/llvm-mingw/releases) for compilation
- Python + pip module
- ...and Python SCons: `$ python -m pip install scons` (may need `--user` flag if missing privileges, ensure the executable is added to the path
- OpenSSL for encryption ([comes with your `git` install](https://stackoverflow.com/questions/50625283/how-to-install-openssl-in-windows-10))
