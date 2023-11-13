#!/usr/bin/env bash

mv example.env .env

# Function to prompt for 'y' input
prompt_confirmation() {
    local prompt_message="$1"
    read -p "$prompt_message (y/n): " -n 1 -r
    echo    # Move to a new line
    [[ $REPLY =~ ^[Yy]$ ]]
}

# Check if 'just' command is available
if ! command -v just &> /dev/null
then
    echo "'just' command not found. 🤨"

    # Ask to install 'just'
    if prompt_confirmation "Do you want to install the 'just' command runner?"
    then
        cargo install just
        echo "'just' has been installed."
    else
        echo "Installation of 'just' cancelled. Can't install tools. ❌"
        exit 0
    fi
fi

# Ask to install tools using 'just'
if prompt_confirmation "Do you want to install tools (cargo-nextest, taplo-cli, cargo-watch, cargo-limit)?"
then
    just install-tools
    echo "Tools have been installed! 👷"
else
    echo "Tools installation cancelled. ❌"
    exit 0
fi
