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
    echo "   ./build.sh build <dev|debug|release|sanity> <features> [platforms]"
    echo "      dev - All gdext assertions, light optimization, documentation (for development)"
    echo "      debug - Some gdext assertions, heavy optimization, documentation (for editor/debug exports)"
    echo "      release - No gdext assertions, heavy optimization, no documentation (for release exports)"
    echo "      sanity - All assertions, no optimization, documentation (for when you're losing your mind)"
    echo ""
    echo "      features are a comma-separated list of crate features, example:"
    echo "          physics_server,animation   - for physics + animation features"
    echo "          ,                          - for no extra features"
    echo ""
    echo "      platforms are a comma-separated list of Rust targets, example:"
    echo "         x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu"
    echo ""
    echo "		wasm32-unknown-emscripten is supported, but takes significantly longer to build"
    echo "			threading is enabled by default, STAGTOOLKIT_THREADING=0 to disable it"
    echo ""
    echo "   Build examples:"
    echo "   ./build.sh build debug physics_server,animation x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu"
    echo "   ./build.sh build release physics_server,animation x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu"
    echo "   STAGTOOLKIT_THREADING=0 ./build.sh build release physics_server wasm32-unknown-emscripten"
    exit 1
fi

# Remove rust from addon as necessary
if [ "$COMMAND" == "derust" ]; then
    echo "Removing gdextension from addon"
    rm -f godot/addons/stag_toolkit/*.gdext*
    rm -rf godot/addons/stag_toolkit/plugin/island_builder/
    rm -rf godot/addons/stag_toolkit/bin/
    exit 0
fi
# Clean build artifacts
if [ "$COMMAND" == "clean" ]; then
    echo "Clearing Rust cache and removing build artifacts"
    cargo clean
    rm -rf build/
    rm -rf godot/addons/stag_toolkit/bin/
    exit 0
fi

if [ "$COMMAND" == "test" ]; then
    cargo test --features default,godot,physics_server,animation
    exit 0
fi

RELEASE_TYPE=$2
FEATURES=$3
if [ ! -n "$RELEASE_TYPE" ]; then
    echo "must specify release type: 'dev' 'debug' 'release' 'sanity'"
    exit 1
fi

# Always include Godot feature for building the addon
FEATURES="godot,$FEATURES"
if [[ "$STAGTOOLKIT_THREADING" == "0" ]]; then
    FEATURES="$FEATURES"
else
    FEATURES="$FEATURES,godot/experimental-threads"
fi
RELEASE_FOLDER="debug"

# Pick build profile and remove safety checks as necessary
if [ "$RELEASE_TYPE" == "sanity" ]; then
    BUILD_PROFILE="dev-sanity"
    FEATURES="$FEATURES,godot/register-docs"
fi
if [ "$RELEASE_TYPE" == "dev" ]; then
    BUILD_PROFILE="dev"
    FEATURES="$FEATURES,godot/register-docs"
fi
if [ "$RELEASE_TYPE" == "debug" ]; then
    BUILD_PROFILE="release"
    FEATURES="$FEATURES,godot/safeguards-dev-balanced,godot/register-docs"
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
BUILD_FLAGS="--lib --profile $BUILD_PROFILE"

LIBNAMES=("libstag_toolkit.so" "stag_toolkit.dll" "libstag_toolkit.dylib" "stag_toolkit.wasm" "stag_toolkit.threads.wasm")
copyartifact() {
    # Looks for any expected library names,
    # and copies them from the target directory to the addon
    TARGETDIR="target/${TARGET}/${BUILD_PROFILE}"
    BINDIR="${ADDON_PATH}/${RELEASE_FOLDER}"
    mkdir -p $BINDIR

    for FILENAME in "${LIBNAMES[@]}"; do
        if [ -f "${TARGETDIR}/${FILENAME}" ]; then
            TARGET_FILENAME=$FILENAME

            if [[ $FILENAME == "stag_toolkit.wasm" ]] && [[ $STAGTOOLKIT_THREADING != "0" ]]; then
                TARGET_FILENAME="stag_toolkit.threads.wasm"
            fi

            cp "${TARGETDIR}/${FILENAME}" "${BINDIR}/${TARGET_FILENAME}";
            echo "copied artifact ${TARGETDIR}/${FILENAME} -> ${BINDIR}/${TARGET_FILENAME}";
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

    # Certain targets may require a build with nightly
    NIGHTLY=""

    # Certain targets may require additional flags
    TARGET_FLAGS=""
    TARGET_RUSTFLAGS=""

    # Certain targets may require additional features
    TARGET_FEATURES="${FEATURES}"
    if [ "$TARGET" == "wasm32-unknown-emscripten" ]; then
        # godot-rust requires nightly Rust...
        NIGHTLY="+nightly"
        # ...requires standard library
        TARGET_FLAGS="${TARGET_FLAGS} -Zbuild-std"
        # ...requires additional crate features

        # If threading is disabled, enable threads in build
        if [[ $STAGTOOLKIT_THREADING != "0" ]]; then
            TARGET_FEATURES="${TARGET_FEATURES},godot/experimental-wasm"
            TARGET_RUSTFLAGS='RUSTFLAGS="-C link-args=-pthread \
            -C target-feature=+atomics \
            -C link-args=-sSIDE_MODULE=2 \
            -C llvm-args=-enable-emscripten-cxx-exceptions=0 \
            -Z default-visibility=hidden \
            -Z link-native-libraries=no \
            -Z unstable options \
            -C panic=immediate-abort"'
        else
            TARGET_FEATURES="${TARGET_FEATURES},godot/experimental-wasm-nothreads,godot/lazy-function-tables"
        fi
    fi

    echo "${TARGET_RUSTFLAGS} cargo $NIGHTLY build ${TARGET_FLAGS} ${BUILD_FLAGS} --features ${TARGET_FEATURES} --no-default-features --target $TARGET"
    ${TARGET_RUSTFLAGS} cargo $NIGHTLY build ${TARGET_FLAGS} $(IFS="" echo ${BUILD_FLAGS}) --features ${TARGET_FEATURES} --no-default-features --target $TARGET;
    copyartifact;
done
