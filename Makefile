.PHONY: all clean debug build

all: clean debug build

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
