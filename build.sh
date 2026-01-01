#!/usr/bin/env bash

set -e

COMMAND=$1
if [ ! -n "$COMMAND" ] || [ "$COMMAND" == "help" ]; then
    echo "Usage examples:"
    echo "   ./build.sh help   # Provides this help output"
    echo "   ./build.sh clean  # Removes all build artifacts"
    echo "   ./build.sh derust # Removes gdextension from StagToolkit Godot addon"
    echo "   ./build.sh test   # Perform Rust unit tests"
    echo ""
    echo "   ./build.sh build <sanity|dev|debug|release> <features> [platforms]"
    echo "      sanity - All assertions, no optimization (for when you're losing your mind)"
    echo "      dev - All gdext assertions, light optimization (for development)"
    echo "      debug - Some gdext assertions, heavy optimization (for editor/debug exports)"
    echo "      release - No gdext assertions, heavy optimization (for release exports)"
    echo ""
    echo "      platforms are a comma-separated list of Rust targets, example:"
    echo "         x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu"
    echo ""
    echo "   Build examples:"
    echo "   ./build.sh build debug physics_server,animation x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu"
    echo "   ./build.sh build release physics_server,animation x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu"
	exit 1
fi

# Remove rust from addon as necessary
if [ "$COMMAND" == "derust" ]; then
    echo "Removing gdextension from addon"
    rm -f godot/addons/stag_toolkit/*.gdext*
    rm -rf godot/addons/stag_toolkit/plugin/island_builder/
fi
# Clean build artifacts
if [ "$COMMAND" == "clean" ] || [ "$COMMAND" == "derust" ]; then
    echo "Clearing Rust cache and removing build artifacts"
    cargo clean
    rm -rf build/
    rm -rf godot/addons/stag_toolkit/bin/
    exit 0
fi

if [ "$COMMAND" == "test" ]; then
    cargo test --all-features
    exit 0
fi

RELEASE_TYPE=$2
FEATURES=$3
if [ ! -n "$RELEASE_TYPE" ]; then
	echo "must specify release type: 'sanity' 'dev' 'debug' 'release'"
	exit 1
fi

# Always include Godot feature for building the addon
FEATURES="godot,$FEATURES"
RELEASE_FOLDER="debug"

# Pick build profile and remove safety checks as necessary
if [ "$RELEASE_TYPE" == "sanity" ]; then
    BUILD_PROFILE="dev-sanity"
fi
if [ "$RELEASE_TYPE" == "dev" ]; then
    BUILD_PROFILE="dev"
fi
if [ "$RELEASE_TYPE" == "debug" ]; then
    BUILD_PROFILE="release"
    FEATURES="$FEATURES,godot/safeguards-dev-balanced"
fi
if [ "$RELEASE_TYPE" == "release" ]; then
    RELEASE_FOLDER="release"
    BUILD_PROFILE="release-lto"
    FEATURES="$FEATURES,godot/safeguards-release-disengaged"
fi

# Prepare addon directories
ADDON_PATH=godot/addons/stag_toolkit/bin
mkdir -p ${ADDON_PATH}
touch ${ADDON_PATH}/.gdignore

cargo fetch
BUILD_FLAGS="--lib --profile $BUILD_PROFILE --features $FEATURES"

LIBNAMES=("libstag_toolkit.so" "stag_toolkit.dll" "libstag_toolkit.dylib")
copyartifact() {
    # Looks for any expected library names,
    # and copies them from the target directory to the addon
    TARGETDIR="target/${TARGET}/${BUILD_PROFILE}"
    BINDIR="${ADDON_PATH}/${RELEASE_FOLDER}"
    mkdir -p $BINDIR

    for FILENAME in "${LIBNAMES[@]}"; do
        if [ -f "${TARGETDIR}/${FILENAME}" ]; then
            cp "${TARGETDIR}/${FILENAME}" "${BINDIR}/${FILENAME}";
            echo "copied artifact ${TARGETDIR}/${FILENAME} -> ${BINDIR}/${FILENAME}";
        fi;
    done
}

TARGETS=$4
# If no target was specified, automatically select our own platform
if [ ! -n "$TARGETS" ]; then
    TARGETS=`rustc -vV | sed -n 's|host: ||p'`
    echo "Automatically derived target platform: $TARGETS"
fi

# Split platform target names list into an array, and build all targets
IFS="," read -ra TARGET_NAMES <<< "$TARGETS"
for TARGET in "${TARGET_NAMES[@]}"; do
    echo "Building $TARGET";
    echo "cargo build ${BUILD_FLAGS} --target $TARGET"
    cargo build $(IFS="" echo ${BUILD_FLAGS}) --target $TARGET;
    copyartifact;
done
