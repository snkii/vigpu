#!/bin/sh
set -eu

os=$(uname -s)
arch=$(uname -m)
case "$os:$arch" in
  Linux:x86_64|Linux:amd64) asset_name='vigpu-linux-x64.tar.gz' ;;
  Darwin:arm64|Darwin:aarch64) asset_name='vigpu-macos-arm64.tar.gz' ;;
  Darwin:*) printf '%s\n' 'ViGPU currently supports Apple Silicon Macs only.' >&2; exit 1 ;;
  Linux:*) printf '%s\n' 'ViGPU currently provides Linux binaries for x86_64 only.' >&2; exit 1 ;;
  *) printf 'Unsupported platform: %s\n' "$os $arch" >&2; exit 1 ;;
esac

command -v curl >/dev/null 2>&1 || { printf '%s\n' 'curl is required.' >&2; exit 1; }
command -v tar >/dev/null 2>&1 || { printf '%s\n' 'tar is required.' >&2; exit 1; }
release_json=$(curl -fsSL -H 'Accept: application/vnd.github+json' \
  https://api.github.com/repos/snkii/vigpu/releases/latest)
asset_url=$(printf '%s\n' "$release_json" \
  | sed -nE "s/^[[:space:]]*\"browser_download_url\":[[:space:]]*\"([^\"]*${asset_name})\".*/\1/p" \
  | head -n 1)
[ -n "$asset_url" ] || { printf 'Could not find %s in the latest ViGPU release.\n' "$asset_name" >&2; exit 1; }

temporary_dir=$(mktemp -d)
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM
curl -fsSL "$asset_url" -o "$temporary_dir/vigpu.tar.gz"
tar -xzf "$temporary_dir/vigpu.tar.gz" -C "$temporary_dir"

install_dir="$HOME/.local/bin"
mkdir -p "$install_dir"
install -m 755 "$temporary_dir/vigpu" "$install_dir/vigpu"
printf 'Installed ViGPU to %s/vigpu\n' "$install_dir"
if [ "$os" = Darwin ]; then
  printf '%s\n' 'Apple GPU telemetry uses shared system memory; per-process GPU readings are unavailable.'
fi
case ":${PATH}:" in
  *":${install_dir}:"*) printf '%s\n' 'Run: vigpu' ;;
  *)
    printf '%s\n' 'Add ViGPU to this terminal session and launch it:'
    printf '  export PATH="$HOME/.local/bin:$PATH"\n'
    printf '  vigpu\n'
    printf '%s\n' 'To keep this setting, add the export command to your shell profile, then open a new terminal.'
    ;;
esac
