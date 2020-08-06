#!/bin/sh
# wpscan-analyze install script for MacOS and Linux.
# This script will download, unzip and install binary file to /usr/local/bin/
# Script requirement: wget
# More infos https://github.com/lukaspustina/wpscan-analyze

# Install config
install_to="/usr/local/bin"
binary_file="${install_to}/wpscan-analyze"

# Fast fail if any errors
set -e

# Cd to temp and try to create/delete file
cd /tmp/
touch "wpscan-analayze-install-$(date)" && rm -f "wpscan-analayze-install-*"

# Info banner
echo
echo
echo "[INFO] wpscan-analyze install script for MacOS and Linux"
echo
echo

# Determine latest version based on lastest git tag with github api
version="v`curl --silent "https://api.github.com/repos/lukaspustina/wpscan-analyze/releases/latest" | grep tag_name | sed -E 's/.*"v(.*)",/\1/'`"

# Ask to build from source ?
echo "[QUESTION] Do you want to build latest wpscan-analyze version from source ?"
read -p "[INFO] Answer No to download binary from github and copy it to ${install_to} [y/n] " -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]; then
    #Cleaning git repo
    rm -rf wpscan-analyze
    # init repo
    if ! git clone https://github.com/lukaspustina/wpscan-analyze; then
        echo "[ERROR] Github repo lukaspustina/wpscan-analyze is not accessible"
        exit 1
    fi
    cd wpscan-analyze
    echo "[INFO] Checkout latest stable version"
    git checkout ${version}
    if ! which cargo; then
        read -p "[QUESTION] Cargo is not detected. Do you want install Rust environment? [y/n]" -n 1 -r
        echo    # (optional) move to a new line
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
            source $HOME/.cargo/env
        else
            echo "Quitting"
            exit 1
        fi
    fi
    echo "[INFO] Building..."
    cargo install --path .
    echo "[INFO] Uninstall with: 'cargo uninstall wpscan-analyze'"
else

    # Quit if not a 64 bits arch
    if [ ! "$(uname -m)" = x86_64 ]; then
        echo "[ERROR] Unsupported architecture, only 64 bits are supported by this script. Please install Rust environment and build wpscan-analayze from source. Visit https://github.com/lukaspustina/wpscan-analyze for more infos."
        exit 1
    fi

    # Determine file name based on operating system and version
    filename=""
    if [ "$(uname)" = Linux ]; then
        filename="wpscan-analyze-${version}-x86_64-unknown-linux-gnu"
    else
        if [ "$(uname)" = Darwin ]; then
            filename="wpscan-analyze-${version}-x86_64-apple-darwin"
        else
            echo "[ERROR] Unsupported operating system, please install Rust environment and build wpscan-analayze from source. Visit https://github.com/lukaspustina/wpscan-analyze for more infos."
            exit 1
        fi
    fi
    # Download
    echo "[INFO] Downloading wpscan-analyze ${version} $(uname -m) $(uname) binary"

    # CLeaning before downloading binary
    rm "${filename}.gz" 2>/dev/null || true
    rm "${filename}" 2>/dev/null || true

    # Get binary file
    wget "https://github.com/lukaspustina/wpscan-analyze/releases/download/${version}/${filename}.gz"

    # Unzip it
    gzip -d "${filename}.gz"

    # Copy file
    echo "[INFO] Copying binary from /tmp/${filename} to ${binary_file}"
    rm "${binary_file}" 2>/dev/null || true
    cp "${filename}" "${binary_file}"

    # Make the file executable
    chmod +x ${binary_file}
    echo "[INFO] Uninstall with: 'rm ${binary_file}'"
fi
echo "[INFO] Success! You might have to reopen your terminal window to use 'wpscan-analyze'"
echo "[INFO] Visit https://github.com/lukaspustina/wpscan-analyze for more informations"
