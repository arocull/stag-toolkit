.PHONY: all clean debug build test godot-abyss-release godot-abyss-debug

all: test debug build

clean:
	@cargo clean
	@rm -rf godot/addons/stag_toolkit/bin/
	@mkdir -p godot/addons/stag_toolkit/bin/
	@touch godot/addons/stag_toolkit/bin/.gdignore

build:
	@cargo build --release
	@mkdir -p godot/addons/stag_toolkit/bin/release/
	@cp target/release/libStagToolkit.so godot/addons/stag_toolkit/bin/release/libStagToolkit.so

debug:
	@cargo build
	@mkdir -p godot/addons/stag_toolkit/bin/debug/
	@cp target/debug/libStagToolkit.so godot/addons/stag_toolkit/bin/debug/libStagToolkit.so

test:
	@cargo test

# Builds Godot Linux export template with encryption for Abyss
godot-abyss-release:
	@./godot-build/build.sh abyss release
godot-abyss-debug:
	@./godot-build/build.sh abyss debug
