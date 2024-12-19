.PHONY: all clean debug build test test-rust test-godot bench godot-abyss-release godot-abyss-debug

all: debug build

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

test: test-rust test-godot

test-rust:
	@cargo test

test-godot:
	@cd godot/ && godot --headless --stagtest --timeout=15

bench:
	@cd godot/ && godot --headless --stagtest --bench --timeout=60

# Builds Godot Linux export template with encryption for Abyss
godot-abyss-release:
	@./godot-build/build.sh abyss release 4.2.2-stable
godot-abyss-debug:
	@./godot-build/build.sh abyss debug 4.2.2-stable
