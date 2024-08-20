# Stag Toolkit

All-purpose toolkit for game creation for Lonesome Stag studios.

## To-Dos
- Add Perlin noise output to vertex color alpha
- Add Ambient Occlusion baking for IslandBuilder
- QuickHull and Decimation for IslandBuilder collision hulls
- Multi-thread IslandBuilder
- If possible, find a faster SurfaceNets algorithm (or at least do better performance testing)

## Resources
- [Setting Up](docs/setup.md)
- [rust book](https://doc.rust-lang.org/stable/book/)
- [godot-rust book](https://godot-rust.github.io/book/)
- [Dieter Rams 10 Principles for Good Design](https://www.vitsoe.com/us/about/good-design)
- [tri_mesh](https://docs.rs/tri-mesh/latest/tri_mesh/)

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

