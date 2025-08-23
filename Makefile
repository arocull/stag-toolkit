.PHONY: all bindir clean clean-bin derust import debug build build-windows build-mac test test-rust test-godot test-sanity bench bench-rust bench-godot bundle doc doc-clean doc-gdscript doc-gdextension doc-rust sphinx

all: build


## CLEANUP ##

clean: clean-bin doc-clean
	@cargo clean
	@rm -rf build/

clean-bin:
	@rm -rf godot/addons/stag_toolkit/bin/

derust:
	@rm godot/addons/stag_toolkit/*.gdext*
	@rm godot/addons/stag_toolkit/plugin/island_builder.*
	@rm -rf godot/addons/stag_toolkit/plugin/island_builder/


## BUILD ##

bindir:
	@mkdir -p godot/addons/stag_toolkit/bin/
	@touch godot/addons/stag_toolkit/bin/.gdignore
	@mkdir -p godot/addons/stag_toolkit/bin/release/
	@mkdir -p godot/addons/stag_toolkit/bin/debug/

import:
	@cd godot && godot --headless --import

build: bindir
	@cargo build --release --features godot,physics_server
	@cp target/release/libstag_toolkit.so godot/addons/stag_toolkit/bin/release/libstag_toolkit.so

build-windows: bindir
	@cargo build --target x86_64-pc-windows-gnu --release --features godot,physics_server
	@cp target/x86_64-pc-windows-gnu/release/stag_toolkit.dll godot/addons/stag_toolkit/bin/release/stag_toolkit.dll

build-mac: bindir
	@cargo build --target x86_64-apple-darwin  --release --features godot,physics_server
	@cp target/x86_64-apple-darwin/release/libstag_toolkit.dylib godot/addons/stag_toolkit/bin/release/libstag_toolkit.dylib

debug: bindir
	@cargo build
	@cp target/debug/libstag_toolkit.so godot/addons/stag_toolkit/bin/debug/libstag_toolkit.so

bundle: clean build build-windows
	@mkdir -p build/
	@cd godot && zip -qqr9 ../build/addon_StagToolkit.zip addons/


## TEST / BENCH ##

test: test-rust-release test-godot

test-rust:
	@cargo test --all-features

test-rust-release:
	@cargo test --release --features godot,physics_server

test-godot: build
	@godot --path godot/ --headless --no-header --stagtest --timeout=90 --timescale=5.0

test-sanity: build test-rust
	@godot --path godot/ --headless --no-header --stagtest --timeout=90 --test=res://test/sanity


bench: bench-rust bench-godot

bench-rust: bench-rust-mesh bench-rust-island bench-rust-simulation

bench-rust-mesh:
	cargo bench --no-default-features --features physics_server -- Trimesh/

bench-rust-island:
	cargo bench --no-default-features --features physics_server -- IslandBuilder/

bench-rust-simulation:
	cargo bench --no-default-features --features physics_server -- simulation

bench-godot: build
	@godot --path godot/ --headless --no-header --stagtest --bench --timeout=300


## DOC GEN ##

doc: doc-gdscript doc-gdextension sphinx

doc-clean:
	@rm -rf sphinx/gen
	@rm -rf build/public

doc-gdscript:
	@mkdir -p sphinx/gen/gdscript
	@cd godot/ && godot --headless --doctool ../sphinx/gen/gdscript/ --gdscript-docs res://addons/stag_toolkit/

doc-gdextension:
	@mkdir -p sphinx/gen/gdextension
	@cd godot/ && godot --headless --doctool ../sphinx/gen/gdextension/ --gdextension-docs
	@mv sphinx/gen/gdextension/doc_classes/* sphinx/gen/gdextension/

doc-rust:
	@cargo doc --lib --no-default-features

sphinx:
	@cd sphinx && ./build.sh
	@cd sphinx && sphinx-build . ../build/public
