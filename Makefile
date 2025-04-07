.PHONY: all clean clean-bin derust debug bindir build build-windows build-mac test test-rust test-godot test-sanity bench bundle godot-abyss-release godot-abyss-debug

all: build

bindir:
	@mkdir -p godot/addons/stag_toolkit/bin/
	@touch godot/addons/stag_toolkit/bin/.gdignore
	@mkdir -p godot/addons/stag_toolkit/bin/release/
	@mkdir -p godot/addons/stag_toolkit/bin/debug/

clean: clean-bin
	@cargo clean

clean-bin:
	@rm -rf godot/addons/stag_toolkit/bin/

derust:
	@rm godot/addons/stag_toolkit/*.gdext*
	@rm godot/addons/stag_toolkit/plugin/island_builder.*
	@rm -rf godot/addons/stag_toolkit/plugin/ui/

build: bindir
	@cargo build --release
	@cp target/release/libStagToolkit.so godot/addons/stag_toolkit/bin/release/libStagToolkit.so

build-windows: bindir
	@cargo build --target x86_64-pc-windows-gnu --release
	@cp target/x86_64-pc-windows-gnu/release/StagToolkit.dll godot/addons/stag_toolkit/bin/release/StagToolkit.dll

build-mac: bindir
	@cargo build --target x86_64-apple-darwin
	@cp target/x86_64-apple-darwin/release/libStagToolkit.dylib godot/addons/stag_toolkit/bin/release/libStagToolkit.dylib

debug: bindir
	@cargo build
	@cp target/debug/libStagToolkit.so godot/addons/stag_toolkit/bin/debug/libStagToolkit.so

test: test-rust test-godot

test-rust:
	@cargo test --release

test-godot: build
	@cd godot/ && godot --headless --stagtest --timeout=90

test-sanity: build
	@cd godot/ && godot --headless --stagtest --timeout=90 --test=res://test/sanity

bench: build
	@cd godot/ && godot --headless --stagtest --bench --timeout=300

# Builds Godot Linux export template with encryption for Abyss
godot-abyss-release:
	@./godot-build/build.sh abyss release 4.4.1-stable 12
godot-abyss-debug:
	@./godot-build/build.sh abyss debug 4.4.1-stable 12

bundle: clean build build-windows
	@mkdir -p build/
	@cd godot && zip -qqr9 ../build/addon_StagToolkit.zip addons/
