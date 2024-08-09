.PHONY: all clean debug build

all: clean debug build

clean:
	@rm -rf rust/StagToolkit/target/
	@rm -rf godot/addons/stag_toolkit/bin/
	@mkdir -p godot/addons/stag_toolkit/bin/
	@touch godot/addons/stag_toolkit/bin/.gdignore

build:
	@cd rust/StagToolkit && cargo build --release
	@mkdir -p godot/addons/stag_toolkit/bin/release/
	@cp rust/StagToolkit/target/release/libStagToolkit.so godot/addons/stag_toolkit/bin/release/libStagToolkit.so

debug:
	@cd rust/StagToolkit && cargo build
	@mkdir -p godot/addons/stag_toolkit/bin/debug/
	@cp rust/StagToolkit/target/debug/libStagToolkit.so godot/addons/stag_toolkit/bin/debug/libStagToolkit.so
