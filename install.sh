#!/bin/sh
# This script will download, unzip and install binary file to /usr/local/opt/wpscan-analyze
# Additionally, it will link a shortcut in /usr/local/bin/
# Script requirement: wget, gzip
# More infos https://github.com/lukaspustina/wpscan-analyze

# Install config
version=v1.0.4
install_link_to="/usr/local/bin"
target_binary_file="/usr/local/opt/wpscan-analyze"
symbolic_link="${install_link_to}/wpscan-analyze"

# Install script
# Fast fail if any errors
set -e
# Move to temp dir
cd /tmp/
# Quit if not a 64 bits arch
if [ ! "$(uname -m)" = x86_64 ]; then
    echo "Unsupported architecture, only 64 bits are supported by this script. Please install Rust environment and build wpscan-analayze from source. Check https://github.com/lukaspustina/wpscan-analyze for more infos."
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
        echo "Unsupported operating system, please install Rust environment and build wpscan-analayze from source. Check https://github.com/lukaspustina/wpscan-analyze for more infos."
        exit 1
    fi
fi
# Download
echo "Downloading wpscan-analyze ${version} x64 bits $(uname) binary"
# Remove previous file if any
rm "${filename}.gz" 2>/dev/null || true
# Get binary file
wget --quiet "https://github.com/lukaspustina/wpscan-analyze/releases/download/${version}/${filename}.gz"
# Remove previous file if any
rm "${filename}" 2>/dev/null || true
# Unzip it
gzip -d "${filename}.gz"
# Remove previous file if any
rm "${target_binary_file}" 2>/dev/null || true
# Copy file
echo "Copying binary to ${target_binary_file}"
cp "${filename}" "${target_binary_file}"
# Remove previous file if any
rm "${symbolic_link}" 2>/dev/null || true
# Link
echo "Linking ${symbolic_link}"
ln -s "${target_binary_file}" "${symbolic_link}"
# Chmod +x
chmod +x ${symbolic_link}
chmod +x ${target_binary_file}
echo "Installed as:"
if ! which wpscan-analyze; then
    echo "Seems that ${install_link_to} is not in your PATH"
    exit 1
fi
echo "Success! Restart your terminal to use 'wpscan-analyze' command"