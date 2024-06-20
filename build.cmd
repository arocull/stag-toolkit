@REM Build binaries
cd rust\StagToolkit && cargo build
cd ..\..
cd rust\StagToolkit && cargo build --release
cd ..\..

@REM Store debug binary
mkdir rust\StagToolkit\target\debug
copy /y rust\StagToolkit\target\debug\StagToolkit.dll /b godot\addons\stag_toolkit\bin\debug\StagToolkit.dll /b

@REM Store release binary
mkdir rust\StagToolkit\target\release
copy /y rust\StagToolkit\target\release\StagToolkit.dll /b godot\addons\stag_toolkit\bin\release\StagToolkit.dll /b
