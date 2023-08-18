# Utopia

A multi-emulator

## Current Status

| System           | Status |
| ---------------- | ------ |
| NES              | Playable. Not all mapper (cartridge) types supported. |
| Game Boy         | Playable. Not all MBC (cartridge) types supported. Not Game Boy Color support yet. |
| SNES             | Mostly playable. A few graphical features are missing and sound is glitchy. No on-cartridge enhancement chips (DSP1, SuperFX, etc.) supported yet. |
| Game Boy Advance | Very early stages. |
| Nintendo 64      | Very early stages. |

## Building

First install Rust: https://www.rust-lang.org/tools/install

You will also need to install SDL 2. On Ubuntu-based systems this can be done using:

    sudo apt install libsdl2-dev

Once that is done, run:

    git clone https://github.com/jlippitt/utopia.git
    cd ./utopia
    cargo install --path utopia-cli

## Command Line Interface

    utopia [OPTIONS] <ROM_PATH>

| Option                       | Description |
| -f, --full-screen            | Enables full-screen mode. This can also be toggled while in-game using F11. |

## Important Note

For the SNES emulator to work, you will need a copy of a 64-byte IPL ROM which can be found elsewhere. This should be placed in the same
directory (folder) as the ROM, with the name 'ipl_rom.bin'.
