# ViGPU

```
▰▱ ViGPU

hi! (▰ᴗ▱)
```

A simple, pretty GPU monitor for Linux, Windows, and Apple Silicon, built in Rust. ViGPU puts the live stats you use in `nvidia-smi` and `nvtop` into a calmer terminal dashboard.

[![CI](https://github.com/snkii/vigpu/actions/workflows/ci.yml/badge.svg)](https://github.com/snkii/vigpu/actions/workflows/ci.yml)
[![CodeQL](https://github.com/snkii/vigpu/actions/workflows/codeql.yml/badge.svg)](https://github.com/snkii/vigpu/actions/workflows/codeql.yml)
[![Latest release](https://img.shields.io/github/v/release/snkii/vigpu)](https://github.com/snkii/vigpu/releases)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

## Install on Debian and Ubuntu

Add the signed ViGPU APT repository, then install the package:

```sh
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://seon.uk/vigpu/vigpu-archive-keyring.asc \
  | sudo gpg --dearmor -o /etc/apt/keyrings/vigpu.gpg
echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/vigpu.gpg] https://seon.uk/vigpu stable main" \
  | sudo tee /etc/apt/sources.list.d/vigpu.list
sudo apt update
sudo apt install vigpu
```

The APT package targets 64-bit Debian 12+ and Ubuntu 22.04+ servers. The NVIDIA driver must be installed and NVML available on the host. Package releases are published from version tags; see [APT repository setup](docs/APT_REPOSITORY.md).

## Install without sudo

The user installer downloads a prebuilt Linux x86_64 binary and installs it in `~/.local/bin`. It needs `curl` and `tar`, but no Rust toolchain or administrator access:

```sh
curl -fsSL https://seon.uk/vigpu/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
vigpu
```

If `vigpu` is not found in later terminals, add the `export PATH=...` line to your shell profile and open a new terminal. An NVIDIA driver with NVML is required.

## Windows

In PowerShell, install the prebuilt x64 binary for the current user:

```powershell
irm https://seon.uk/vigpu/install.ps1 | iex
vigpu
```

The installer adds ViGPU to the user PATH and current PowerShell session. Open a new PowerShell window if a later session cannot find `vigpu`. An NVIDIA driver is required.

## macOS

On Apple Silicon, install the prebuilt binary with Homebrew:

```sh
brew install --formula https://seon.uk/vigpu/vigpu.rb
vigpu
```

ViGPU reports Apple GPU utilization, unified memory, temperature, power, and fan data. macOS does not expose per-process GPU readings through this backend, so the Runs section has no process entries. Intel Macs are not currently supported.

## Update

Re-run the matching installer to replace a user-local binary with the latest release. Package-manager installs use their normal upgrade commands:

```sh
# Debian or Ubuntu
sudo apt update && sudo apt upgrade vigpu

# Linux user install, no sudo
curl -fsSL https://seon.uk/vigpu/install.sh | sh

# Apple Silicon with Homebrew
brew reinstall --formula https://seon.uk/vigpu/vigpu.rb
```

On Windows, run `irm https://seon.uk/vigpu/install.ps1 | iex` again in PowerShell. Check the installed version with `vigpu --version`.

## Install from source

Requires Rust/Cargo, an NVIDIA driver, and NVML:

```sh
cargo install --git https://github.com/snkii/vigpu.git --locked
vigpu
```

To build the current checkout locally:

```sh
cargo install --path . --force --locked
```

## Dashboard

ViGPU samples GPU readings every **250 ms** and renders animation at about **30 frames per second**. Rounded GPU cards show utilization, VRAM, power, temperature, fan speed, and a live history graph. Process cards are grouped under their GPU and include the executable, command filename, PID, user, CPU percentage, and VRAM percentage. Long process commands are available in a scrollable detail view.

The official wordmark is `▰▱ ViGPU`. Gauges use a red-to-yellow-to-green load scale, and the green Braille graph has a soft ivory and yellow shimmer. The layout adapts to terminal width and keeps Notes pinned at the bottom while Runs scroll with the selection.

## Controls

- Arrow keys or `j`/`k`: select a process
- `Enter` or `Space`: open process details
- `n`: edit persistent Notes
- `t`: preview and choose a theme
- `q`, `Esc`, or `Ctrl+C`: close the active window or exit the dashboard

Theme previews apply immediately. Press `Enter` to save a theme or `Esc` to close the picker and restore the previous choice. Notes are saved when their editor closes with `Esc`.

Available themes: ViGPU Default (Gruvbox), Alucard, Ayu Mirage, Catppuccin Mocha, Dracula, Everforest, GitHub Dark, Kanagawa, Nightfox, Nord, One Dark, Rosé Pine, Solarized Dark, Sonokai, and Tokyo Night. Palette origins and licenses are listed in [THEME_CREDITS.md](THEME_CREDITS.md).

Set a custom GPU sampling interval with:

```sh
vigpu --interval-ms 500
```

The interval range is 100–5000 ms. On Linux, process commands and user names are read from `/proc`; restricted process details fall back to the short process name, numeric user ID, or `?`. Unsupported GPU metrics appear as `--` or `n/a`.

## Version

The initial release is **0.1.0**. Release tags use the `v0.1.0` format.

## License

ViGPU is dual-licensed under your choice of the [MIT License](LICENSE-MIT) or [Apache License, Version 2.0](LICENSE-APACHE). Theme palette sources and their licenses are listed in [THEME_CREDITS.md](THEME_CREDITS.md).

## Contact

For project questions, contact [iamseonuk@gmail.com](mailto:iamseonuk@gmail.com). Please report security issues privately as described in [SECURITY.md](SECURITY.md).
