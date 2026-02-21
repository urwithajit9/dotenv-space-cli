#!/usr/bin/env bash

set -e

REPO="urwithajit9/dotenv-space-cli"
BINARY_NAME="dotenv-space"

echo "Installing dotenv-space..."

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux) OS="unknown-linux-gnu" ;;
  Darwin) OS="apple-darwin" ;;
  *) echo "Unsupported OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture"; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"

LATEST=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep tag_name | cut -d '"' -f4)

URL="https://github.com/$REPO/releases/download/$LATEST/${BINARY_NAME}-${TARGET}.tar.gz"

TMP_DIR=$(mktemp -d)

echo "Downloading $URL"

curl -L $URL -o $TMP_DIR/bin.tar.gz

tar -xzf $TMP_DIR/bin.tar.gz -C $TMP_DIR

INSTALL_DIR="/usr/local/bin"

if [ ! -w "$INSTALL_DIR" ]; then
  echo "Need sudo permission to install"
  sudo mv $TMP_DIR/${BINARY_NAME}-${TARGET} $INSTALL_DIR/$BINARY_NAME
else
  mv $TMP_DIR/${BINARY_NAME}-${TARGET} $INSTALL_DIR/$BINARY_NAME
fi

chmod +x $INSTALL_DIR/$BINARY_NAME

echo "Installed successfully 🎉"
echo "Run: dotenv-space --help"
