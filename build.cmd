@REM Sets up a basic debug build for StagToolkit
cargo build --release --features godot,physics_server,godot/safeguards-dev-balanced
mkdir godot\addons\stag_toolkit\bin\debug
copy /y target\release\stag_toolkit.dll /b godot\addons\stag_toolkit\bin\debug\stag_toolkit.dll /b
