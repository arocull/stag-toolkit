#!/usr/bin/bash
TARGET=release
PROJECT=$1
VERSION=$3
THREADS=$4

BUILD_SETTINGS=`pwd`/godot-build
BUILD_OUTPUT=`pwd`/build
GODOT_REPO=`pwd`/../godot/
GODOT_OUTPUT=$GODOT_REPO/bin

if [ "$2" = "debug" ]; then
echo "Building '$PROJECT' DEBUG from `pwd` ---"
TARGET=debug
else
echo "Building '$PROJECT' RELEASE from `pwd` ---"
fi

OUTPUT_FILENAME=godot.linuxbsd.template_$TARGET.x86_64

# Generate encryption key, store as environment variable and in file for export
echo "--- generating encryption key..."
SCRIPT_AES256_ENCRYPTION_KEY=`openssl rand -hex 32`
echo "$SCRIPT_AES256_ENCRYPTION_KEY" > godot.gdkey

# Build Godot export templates with link-time optimization
cd $GODOT_REPO

git switch -
git fetch
git pull
git checkout $VERSION

# See optimization options:
# https://docs.godotengine.org/en/stable/contributing/development/compiling/introduction_to_the_buildsystem.html#optimization-level
if [ "$TARGET" = "debug" ]; then
    echo "--- starting DEBUG build..."
    scons -j$THREADS target=template_debug optimize=debug build_feature_profile="$BUILD_SETTINGS/$PROJECT.build" extra_suffix="$PROJECT_DEBUG" platform=linuxbsd
else
    echo "--- starting RELEASE build..."
    scons -j$THREADS target=template_release platform=linuxbsd production=yes lto=full optimize=speed build_feature_profile="$BUILD_SETTINGS/$PROJECT.build" extra_suffix="$PROJECT"
fi

echo `pwd`
mkdir -p $BUILD_OUTPUT
mv $(pwd)/bin/$OUTPUT_FILENAME.$PROJECT $BUILD_OUTPUT/$OUTPUT_FILENAME.$PROJECT

# Strip debug symbols from binary to optimize size. Note: this makes crash backtraces impossible to find
strip $BUILD_OUTPUT/$OUTPUT_FILENAME.$PROJECT

echo "--- build outputed to 'build/$OUTPUT_FILENAME.$PROJECT'"
