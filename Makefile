.PHONY: all clean derust import sanity dev debug debug-all release bundle test test-rust test-godot test-sanity bench bench-rust bench-godot doc doc-clean doc-gdscript doc-gdextension doc-rust sphinx

FEATURE_FLAGS=physics_server

all: debug

## CLEANUP ##

clean: doc-clean
	@./build.sh clean

derust:
	@./build.sh derust

## BUILD ##

import:
	@cd godot && godot --headless --import
sanity:
	./build.sh build sanity $(FEATURE_FLAGS)
dev:
	./build.sh build dev $(FEATURE_FLAGS)
debug:
	./build.sh build debug $(FEATURE_FLAGS)
debug-all:
	./build.sh build debug $(FEATURE_FLAGS) x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu
release:
	./build.sh build release $(FEATURE_FLAGS) x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu
bundle: debug-all release
	@mkdir -p build/
	@cd godot && zip -qqr9 ../build/addon_StagToolkit.zip addons/
	$(MAKE) derust && cd godot && zip -qrr9 ../build/addon_StagToolkit_nogdext.zip addons/


## TEST / BENCH ##

coverage:
	cargo tarpaulin -o html --output-dir reports/ --packages stag-toolkit --all-features

test: test-rust test-godot
test-rust:
	@./build.sh test
test-godot: dev
	@godot --path godot/ --headless --no-header --stagtest --timeout=90 --timescale=5.0
test-sanity: sanity test-rust
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

doc-rust-strict:
	@RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links -D rustdoc::invalid-codeblock-attributes -D rustdoc::invalid-rust-codeblocks -D rustdoc::invalid-html-tags -D rustdoc::bare-urls -D rustdoc::unescaped-backticks -D warnings" cargo doc -p stag-toolkit --no-deps --all-features

sphinx:
	@cd sphinx && ./build.sh
	@cd sphinx && sphinx-build . ../build/public
