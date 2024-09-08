@REM BUILDS RUST BINARY FOR StagToolit Plugin

@REM Build binaries
@REM cargo build
cargo build --release

@REM Store debug binary
@REM mkdir target\debug
@REM mkdir godot\addons\stag_toolkit\bin\debug
@REM copy /y target\debug\StagToolkit.dll /b godot\addons\stag_toolkit\bin\debug\StagToolkit.dll /b

@REM Store release binary
mkdir target\release
mkdir godot\addons\stag_toolkit\bin\release
copy /y target\release\StagToolkit.dll /b godot\addons\stag_toolkit\bin\release\StagToolkit.dll /b
