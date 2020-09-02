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
cd /tmp/ || ( mkdir -p tmp && cd tmp )
touch "wpscan-analayze-install-$(date)" && rm -f "wpscan-analayze-install-*"

# Info banner
echo "[INFO] wpscan-analyze install script for MacOS and Linux"

# Determine latest version based on lastest git tag with github api
raw_version=$(curl --silent "https://api.github.com/repos/lukaspustina/wpscan-analyze/releases/latest" | grep tag_name | sed -E 's/.*"v(.*)",/\1/')
version="v${raw_version}"

old_install=""
old_install_version=""
if which wpscan-analyze > /dev/null 2>&1; then
    old_install=$(which wpscan-analyze)
    old_install_version=$(wpscan-analyze --version)
    echo "[INFO] wpscan-analyze in already installed: ${old_install} (${old_install_version})"
    echo "[INFO] The script will remove this installation"
fi
if [ ! -z "${old_install}" ] && [ "${old_install_version}" = "wpscan-analyze ${raw_version}" ]; then
    echo "[INFO] You already have the latest wpscan-analyze version. If you're looking for the dev version please install it manually."
fi
echo "[INFO] Latest version is ${raw_version}"
# Ask to build from source ?
echo "[QUESTION] Do you want to build latest wpscan-analyze version from source ?"
echo "[QUESTION] Answer No to download binary from github and place it in ${install_to} (You'll be asked to confirm the install path) [y/n/cancel]"
# Reading from tty
read REPLY < /dev/tty
if [ "$REPLY" = "y" ] || [ "$REPLY" = "yes" ] || [ "$REPLY" = "Y" ] || [ "$REPLY" = "Yes" ]; then
    #Cleaning git repo
    rm -rf wpscan-analyze
    # init repo
    if ! git clone --quiet https://github.com/lukaspustina/wpscan-analyze; then
        echo "[ERROR] Github repo lukaspustina/wpscan-analyze is not accessible"
        exit 1
    fi
    cd wpscan-analyze
    echo "[INFO] Checkout latest stable version"
    git checkout --quiet "${version}"
    if ! which cargo > /dev/null 2>&1; then
        echo "[QUESTION] Cargo is not detected. Do you want install Rust environment? No will cancel installation. [y/n]"
        # Reading from tty
        read REPLY < /dev/tty
        if [ "$REPLY" = "y" ] || [ "$REPLY" = "yes" ] || [ "$REPLY" = "Y" ] || [ "$REPLY" = "Yes" ]; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
            sleep 1
            # According to https://github.com/koalaman/shellcheck/wiki/SC1090
            # ERROR: SC1090: Can't follow non-constant source. Use a directive to specify locationn
            # Is fixed by:
            # shellcheck source=$("$HOME")/.cargo/env
            . "$HOME/.cargo/env"
        else
            echo "Canceled"
            exit 1
        fi
    fi

    if [ ! -z "${old_install}" ]; then
        echo "[INFO] Removing old install ${old_install}"
        if ! cargo uninstall wpscan-analyze > /dev/null 2>&1; then
            rm -rf "${old_install}"
        fi
    fi
    
    echo "[INFO] Building..."
    cargo install --path .
    echo "[INFO] Uninstall with: 'cargo uninstall wpscan-analyze'"

else
    if [ "$REPLY" = "n" ] || [ "$REPLY" = "no" ] || [ "$REPLY" = "N" ] || [ "$REPLY" = "No" ]; then
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
        rm -f "${filename}.gz"  2>/dev/null || true
        rm -f "${filename}"  2>/dev/null || true

        # Get binary file

        if ! wget --quiet "https://github.com/lukaspustina/wpscan-analyze/releases/download/${version}/${filename}.gz"; then
            echo "[ERROR] Make sure 'wget' is installed on your system and your have internet"
        fi

        # Unzip it
        gzip -d "${filename}.gz"

        if [ ! -z "${old_install}" ]; then
            echo "[INFO] Removing old install ${old_install}"
            if ! cargo uninstall wpscan-analyze > /dev/null 2>&1; then
                rm -rf "${old_install}" 
            fi
        fi

        echo "[QUESTION] Set installation path. Press ENTER to skip and use default: [${install_to}]"
        # Reading from tty
        read REPLY < /dev/tty
        if [ ! -z "$REPLY" ]; then
            pwd_i=$(pwd)
            cd "$REPLY"
            touch "write-permissions-test.txt" && rm -f "write-permissions-test.txt"
            echo "$PATH" > "PATH.txt"
            if ! grep -q "$REPLY" "PATH.txt"; then
                echo "[WARNING] ${REPLY} is not in your PATH. Your PATH is $(cat PATH.txt)"
                echo "[INFO] Add a shorcut in your PATH or use the full '$(pwd)/wpscan-analyze'. "
            fi
            rm -f "PATH.txt"
            install_to="$REPLY"
            cd "$pwd_i"
        fi
        binary_file="${install_to}/wpscan-analyze"
        # Copy file
        echo "[INFO] Copying binary from $(pwd)/${filename} to ${binary_file}"
        # Clean just in case
        rm -f "${binary_file}"  2>/dev/null || true
        cp "${filename}" "${binary_file}"

        # Make the file executable
        chmod +x "${binary_file}"
        echo "[INFO] Uninstall with: 'rm ${binary_file}'"
    
    else
        echo "Canceled"
        exit 1
    fi

fi

echo "[INFO] Success! You might have to reopen your terminal window to use 'wpscan-analyze'"
echo "[INFO] Visit https://github.com/lukaspustina/wpscan-analyze for more informations"