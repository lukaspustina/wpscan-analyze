#!/bin/sh
# This script will download, unzip and install binary file to /usr/local/bin/
# Script requirement: wget, gzip
# More infos https://github.com/lukaspustina/wpscan-analyze

# Install config
version=v1.0.4
install_to="/usr/local/bin"
binary_file="${install_to}/wpscan-analyze"

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
rm "${binary_file}" 2>/dev/null || true
# Copy file
echo "Copying binary to ${binary_file}"
cp "${filename}" "${binary_file}"
# Make the file executable
chmod +x ${binary_file}
echo "Installed to:"
if ! which wpscan-analyze; then
    echo "Seems that ${install_to} is not in your PATH"
    exit 1
fi
echo "Success! Restart your terminal to use 'wpscan-analyze' command"