# N O O T

Noots at each key type. For Linux only.

## Installation

Use homebrew:

    brew tap bachrc/noot https://github.com/bachrc/nootnoot
    brew install --HEAD bachrc/noot/noot

Without it, clone the repo and run the installer:

    git clone https://github.com/bachrc/nootnoot
    cd nootnoot
    ./install.sh

Both put the binaries in place. Then open Noot mode from your app launcher.

One permission is needed before anything works: reading the keyboard. The input devices belong to the input group.

    sudo usermod -aG input "$USER"

Log out and back in. Without this, the window says the keyboard is unreadable and the daemon stays quiet.


## Usage

The service is the mode. While nootd runs, every key press plays a noot. Stop the service and nothing plays.

Click the picture to toggle it. Grey is off, green is on. Drag to move the window, right click to close it.

Each press picks one of the two sounds at random. There is a 90 ms gap between plays, otherwise fast typing turns into noise.

## Removing it

    systemctl --user disable --now nootd.service
    rm ~/.config/systemd/user/nootd.service
    rm ~/.local/bin/nootd ~/.local/bin/noot-settings

With homebrew, run brew uninstall noot, then delete the unit by hand as above.