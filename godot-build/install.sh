#!/usr/bin/bash
set -e

echo "Installing ${GODOT_VERSION}"

BINDIR="/usr/local/bin"
INSTALLDIR="/usr/local/threed"

sudo mkdir -p $INSTALLDIR
sudo chown $USER:users $INSTALLDIR

GODOT_URL="https://github.com/godotengine/godot-builds/releases/download/${GODOT_VERSION}/Godot_v${GODOT_VERSION}_linux.x86_64.zip"
GODOT_BIN="/usr/local/threed/Godot_v${GODOT_VERSION}_linux.x86_64"

if [ ! -f $GODOT_BIN ]; then
    wget --waitretry=0.5 --tries=100 -O /tmp/godot.zip $GODOT_URL && sudo unzip -o /tmp/godot.zip -d $INSTALLDIR && rm -rf /tmp/godot.zip && ln -s -f $GODOT_BIN $BINDIR/godot
else
    ln -s -f $GODOT_BIN $BINDIR/godot
fi

sudo chmod a-w /usr/local/bin
echo "Finished Godot installation"
