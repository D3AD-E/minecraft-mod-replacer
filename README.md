# Minecraft Mod Replacer

A Rust utility for replacing Minecraft mod JAR files while maintaining exact file sizes. Can be used to bypass the hashsum checks for prebuilt minecraft server clients. For educational purposes only.

## Features

- Exact size matching with automatic padding
- Interactive file selection and console menu
- Smart ZIP comment padding with fallback method
- Automatic mod folder scanning
- Size validation

## Installation

1. Clone or download this project
2. Build with Cargo:

```bash
cargo build --release
```

## Usage

1. Run the executable
2. Select your replacement JAR file via file dialog
3. Enter path to your Minecraft mods folder
4. Choose which mod to replace from the list
5. Tool automatically pads to match original file size

## How It Works

Maintains file size through two methods:
1. **ZIP Comment Padding**: Modifies JAR's End of Central Directory record (preferred)
2. **Simple Append**: Adds null bytes if ZIP method fails (fallback)

## Dependencies

```toml
[package]
name = "minecraft_mod_replacer"
version = "0.1.0"
edition = "2025"

[dependencies]
dialoguer = "0.11.0"
rfd = "0.15.4"
```
