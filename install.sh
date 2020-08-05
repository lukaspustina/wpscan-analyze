#!/bin/sh
# wpscan-analyze install script for MacOS and Linux. Only works for 64bits devices for now.
# This script will download, unzip and install binary file to /usr/local/bin/
# Script requirement: wget
# More infos https://github.com/lukaspustina/wpscan-analyze

# Install config
install_to="/usr/local/bin"
binary_file="${install_to}/wpscan-analyze"

# Fast fail if any errors
set -e

# Move to temp dir
cd /tmp/

# Quit if not a 64 bits arch
if [ ! "$(uname -m)" = x86_64 ]; then
    echo "Unsupported architecture, only 64 bits are supported by this script. Please install Rust environment and build wpscan-analayze from source. Visit https://github.com/lukaspustina/wpscan-analyze for more infos."
    exit 1
fi

# Determine version based on lastest git tag
git clone https://github.com/lukaspustina/wpscan-analyze --quiet 
cd wpscan-analyze
version=`git tag | tail -1`
cd ..
rm -rf wpscan-analyze 2>/dev/null || true

# Determine file name based on operating system and version
filename=""
if [ "$(uname)" = Linux ]; then
    filename="wpscan-analyze-${version}-x86_64-unknown-linux-gnu"
else
    if [ "$(uname)" = Darwin ]; then
        filename="wpscan-analyze-${version}-x86_64-apple-darwin"
    else
        echo "Unsupported operating system, please install Rust environment and build wpscan-analayze from source. Visit https://github.com/lukaspustina/wpscan-analyze for more infos."
        exit 1
    fi
fi
# Download
echo "Downloading wpscan-analyze ${version} $(uname -m) $(uname) binary"

# CLeaning before downloading binary
rm "${filename}.gz" 2>/dev/null || true
rm "${filename}" 2>/dev/null || true

# Get binary file
wget --quiet "https://github.com/lukaspustina/wpscan-analyze/releases/download/${version}/${filename}.gz"

# Unzip it
gzip -d "${filename}.gz"

# Copy file
echo "Copying binary to ${binary_file}"
rm "${binary_file}" 2>/dev/null || true
cp "${filename}" "${binary_file}"

# Make the file executable
chmod +x ${binary_file}

echo "Success! Visit https://github.com/lukaspustina/wpscan-analyze for more informations."