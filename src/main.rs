// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
mod config;
mod gpu;
mod state;
mod ui;

use std::{
    env,
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use gpu::Collector;
use state::AppState;
use ui::ThemeName;

const DEFAULT_INTERVAL_MS: u64 = 250;
const FRAME_TIME: Duration = Duration::from_millis(33);

fn main() -> Result<(), Box<dyn Error>> {
    let interval_ms = parse_args()?;
    let mut collector = Collector::new()?;
    let first = collector.sample()?;
    let mut app = AppState::new();
    app.set_snapshot(first);
    app.notes = config::load_notes();
    let mut theme_name = config::load_theme();
    let mut theme = ui::Theme::from_name(theme_name);
    let mut menu: Option<(ThemeName, usize)> = None;
    let mut detail_open = false;
    let mut detail_scroll = 0usize;
    let running = Arc::new(AtomicBool::new(true));
    let (tx, rx) = mpsc::channel();
    let worker_running = Arc::clone(&running);
    let interval = Duration::from_millis(interval_ms);
    let worker = thread::spawn(move || {
        while worker_running.load(Ordering::Relaxed) {
            let started = Instant::now();
            if tx.send(collector.sample()).is_err() {
                break;
            }
            let remaining = interval.saturating_sub(started.elapsed());
            if !remaining.is_zero() {
                thread::sleep(remaining);
            }
        }
    });

    let mut terminal = ratatui::init();
    let _restore = RestoreTerminal;
    let mut previous = Instant::now();
    let result = loop {
        while let Ok(sample) = rx.try_recv() {
            match sample {
                Ok(snapshot) => {
                    app.set_snapshot(snapshot);
                    if app.snapshot.processes.is_empty() {
                        detail_open = false;
                        detail_scroll = 0;
                    }
                }
                Err(error) => app.error = Some(error),
            }
        }
        let now = Instant::now();
        app.animate(now.duration_since(previous));
        previous = now;
        if let Err(error) = terminal.draw(|frame| {
            ui::render(
                frame,
                &app,
                theme,
                menu.map(|(_, selected)| selected),
                detail_open,
                detail_scroll,
            )
        }) {
            break Err(error);
        }

        if event::poll(FRAME_TIME)? {
            if let Event::Key(key) = event::read()? {
                // Some terminal backends report Escape as a release event.
                if key.code == KeyCode::Esc {
                    if let Some((original, _)) = menu.take() {
                        theme_name = original;
                        theme = ui::Theme::from_name(original);
                    } else if app.note_editor {
                        app.notes = app.note_buffer.clone();
                        app.notice = config::save_notes(&app.notes)
                            .err()
                            .map(|error| format!("Notes could not be saved: {error}"));
                        app.note_editor = false;
                    } else if detail_open {
                        detail_open = false;
                        detail_scroll = 0;
                        continue;
                    } else {
                        break Ok(());
                    }
                }
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if let KeyCode::Char(character) = key.code {
                    if !character.is_ascii() {
                        continue;
                    }
                }
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break Ok(());
                }
                if app.note_editor {
                    match key.code {
                        KeyCode::Enter if app.note_buffer.matches('\n').count() < 2 => {
                            app.note_buffer.push('\n')
                        }
                        KeyCode::Enter => {}
                        KeyCode::Backspace => {
                            app.note_buffer.pop();
                        }
                        KeyCode::Char(character)
                            if character.is_ascii()
                                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            app.note_buffer.push(character);
                        }
                        _ => {}
                    }
                    continue;
                } else if detail_open {
                    match key.code {
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') => {
                            detail_open = false;
                            detail_scroll = 0;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            detail_scroll = detail_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            detail_scroll = detail_scroll.saturating_add(1);
                        }
                        _ => {}
                    }
                    continue;
                } else if let Some((original, selected)) = menu {
                    match key.code {
                        KeyCode::Up | KeyCode::Left | KeyCode::Char('k') => {
                            let next = (selected + ThemeName::ALL.len() - 1) % ThemeName::ALL.len();
                            theme_name = ThemeName::ALL[next];
                            theme = ui::Theme::from_name(theme_name);
                            menu = Some((original, next));
                        }
                        KeyCode::Down | KeyCode::Right | KeyCode::Char('j') => {
                            let next = (selected + 1) % ThemeName::ALL.len();
                            theme_name = ThemeName::ALL[next];
                            theme = ui::Theme::from_name(theme_name);
                            menu = Some((original, next));
                        }
                        KeyCode::Enter => {
                            app.notice = config::save_theme(theme_name)
                                .err()
                                .map(|error| format!("Theme could not be saved: {error}"));
                            menu = None;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            theme_name = original;
                            theme = ui::Theme::from_name(original);
                            menu = None;
                        }
                        _ => {}
                    }
                } else {
                    let columns = ui::process_columns(terminal.size()?.width.saturating_sub(4))
                        .max(1) as usize;
                    let process_count = app.snapshot.processes.len();
                    match key.code {
                        KeyCode::Char('n') => {
                            app.note_buffer =
                                app.notes.lines().take(3).collect::<Vec<_>>().join("\n");
                            app.note_editor = true;
                        }
                        KeyCode::Char('t') => menu = Some((theme_name, theme_name.index())),
                        KeyCode::Char('q') => break Ok(()),
                        KeyCode::Up => {
                            if let Some(selected) =
                                ui::process_neighbor(&app, app.selected_process, columns, 0, -1)
                            {
                                app.selected_process = selected;
                            }
                        }
                        KeyCode::Down => {
                            if let Some(selected) =
                                ui::process_neighbor(&app, app.selected_process, columns, 0, 1)
                            {
                                app.selected_process = selected;
                            }
                        }
                        KeyCode::Left if columns > 1 => {
                            if let Some(selected) =
                                ui::process_neighbor(&app, app.selected_process, columns, -1, 0)
                            {
                                app.selected_process = selected;
                            }
                        }
                        KeyCode::Right if columns > 1 => {
                            if let Some(selected) =
                                ui::process_neighbor(&app, app.selected_process, columns, 1, 0)
                            {
                                app.selected_process = selected;
                            }
                        }
                        KeyCode::Char('k') => {
                            app.selected_process = app.selected_process.saturating_sub(1);
                        }
                        KeyCode::Char('j') => {
                            app.selected_process =
                                (app.selected_process + 1).min(process_count.saturating_sub(1));
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            detail_open =
                                app.snapshot.processes.get(app.selected_process).is_some();
                            detail_scroll = 0;
                        }
                        _ => {}
                    }
                }
            }
        }
    };
    running.store(false, Ordering::Relaxed);
    let _ = worker.join();
    drop(_restore);
    result?;
    println!("bye! (▰ᴗ▱)");
    Ok(())
}

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn parse_args() -> Result<u64, Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let mut interval_ms = DEFAULT_INTERVAL_MS;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("vigpu — simple NVIDIA GPU monitor for Linux\n\nUSAGE\n  vigpu [--interval-ms 250]\n\nOPTIONS\n  --interval-ms N  GPU sampling interval in milliseconds (100–5000; default 250)\n  -h, --help       Show this help\n  -V, --version    Show version\n\nKEYS\n  n              Edit notes\n  t              Choose theme with live preview\n  ↑/↓, j/k       Select process\n  Enter          Open process details\n  q, Esc, Ctrl+C Exit");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("vigpu {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--interval-ms" => {
                let value = args.next().ok_or("--interval-ms requires a value")?;
                interval_ms = value.parse::<u64>()?;
                if !(100..=5000).contains(&interval_ms) {
                    return Err("--interval-ms must be between 100 and 5000".into());
                }
            }
            _ => return Err(format!("Unknown option: {arg}. Use --help.").into()),
        }
    }
    Ok(interval_ms)
}
