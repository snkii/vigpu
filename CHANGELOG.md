# Changelog

## 0.1.4

- Add `--only-visible` to show only the GPUs in `CUDA_VISIBLE_DEVICES` and their processes. Set `VIGPU_ONLY_VISIBLE=1` to turn it on by default.

## 0.1.3

- Rename the default theme to "ViGPU Default" and give it its own palette of exact xterm-256 colors, so it looks the same in truecolor and 256-color terminals.
- Fix 256-color mapping to use the real xterm cube levels and gray ramp, so every theme lands on its nearest color.

## 0.1.2

- Use a neutral off-white for the default theme's text, which no longer turns peach in 256-color terminals.
- Keep a single column of cards until the terminal is 120 cells wide.

## 0.1.1

- Leave the screen background to the terminal by default, so translucent or blurred terminal windows show through. Set `transparent=false` in the config file to restore the solid theme fill.
- Keep other config lines when saving a theme.

## 0.1.0 — Initial release

- Add a Rust terminal dashboard for NVIDIA GPU telemetry.
- Add responsive GPU and process cards, live load graphs, selectable color themes, and persistent Notes.
- Add Debian package and signed APT repository release automation.
