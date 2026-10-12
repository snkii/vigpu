// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{env, fs, io, path::PathBuf};

use crate::ui::ThemeName;

#[cfg(windows)]
fn path() -> Option<PathBuf> {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|base| base.join("ViGPU").join("config"))
}

#[cfg(not(windows))]
fn path() -> Option<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("vigpu").join("config"))
}

fn notes_path() -> Option<PathBuf> {
    path().map(|path| path.with_file_name("notes"))
}

pub fn load_notes() -> String {
    notes_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|notes| notes.lines().take(3).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

pub fn save_notes(notes: &str) -> io::Result<()> {
    let path = notes_path()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, notes)
}

pub fn load_theme() -> ThemeName {
    let Some(path) = path() else {
        return ThemeName::ViGpuDefault;
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("theme=")
                    .and_then(|value| ThemeName::from_slug(value.trim()))
            })
        })
        .unwrap_or(ThemeName::ViGpuDefault)
}

pub fn save_theme(theme: ThemeName) -> io::Result<()> {
    let path =
        path().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config directory"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("theme={}\n", theme.slug()))
}
