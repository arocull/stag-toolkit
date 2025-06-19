#!/usr/bin/env bash

# Copy down utility scripts from Godot source
mkdir -p misc/utility
curl -sSL https://raw.githubusercontent.com/godotengine/godot/refs/heads/master/misc/utility/color.py > misc/utility/color.py
curl -sSL https://raw.githubusercontent.com/godotengine/godot/master/doc/tools/make_rst.py > make_rst.py

# Parse XML into Godot-usable documentation
# We ignore errors here because we are only including our own documentation,
# ...not Godot-defined types
python3 make_rst.py -o "docs/plugin" -l "en" gen/gdscript || /bin/true
python3 make_rst.py -o "docs/gdextension" -l "en" gen/gdextension || /bin/true
