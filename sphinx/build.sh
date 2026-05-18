#!/usr/bin/env bash
set -e

# Cleanup
make clean
rm -rf xml/
rm -rf static/crate/
rm -rf ../target/doc/

# Copy down utility scripts from Godot source
mkdir -p source/misc/utility
curl -sSL https://raw.githubusercontent.com/godotengine/godot/refs/heads/master/misc/utility/color.py > source/misc/utility/color.py
curl -sSL https://raw.githubusercontent.com/godotengine/godot/master/doc/tools/make_rst.py > source/make_rst.py

# Build documentation from Godot
mkdir -p xml/gdscript
mkdir -p xml/gdextension

(
cd ../godot/
godot --headless --doctool ../sphinx/xml/gdscript/ --gdscript-docs res://addons/stag_toolkit/
godot --headless --doctool ../sphinx/xml/gdextension/ --gdextension-docs
)
mv xml/gdextension/doc_classes/* xml/gdextension/

# Parse XML into Godot-usable documentation
# We ignore errors here because we are only including our own documentation,
# ...not Godot-defined types
python3 source/make_rst.py -o "source/plugin" -l "en" xml/gdscript || /bin/true
python3 source/make_rst.py -o "source/gdextension" -l "en" xml/gdextension || /bin/true

rm source/plugin/index.rst
rm source/gdextension/index.rst

make html

# Build documentation for Rust
cargo doc --lib --all-features --no-deps -r
cp -r ../target/doc/ build/html/crate/
