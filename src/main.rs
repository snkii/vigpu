// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
mod config;
mod gpu;
mod state;
mod ui;
mod update;

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

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use gpu::Collector;
use state::AppState;
use ui::ThemeName;
use unicode_segmentation::UnicodeSegmentation;

const DEFAULT_INTERVAL_MS: u64 = 250;
const FRAME_TIME: Duration = Duration::from_millis(33);

fn main() -> Result<(), Box<dyn Error>> {
    if update::handle_internal_args() {
        return Ok(());
    }
    update::dispatch_user_overlay();
    let options = parse_args()?;
    if !options.no_update {
        update::spawn_background_check();
    }
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
    let interval = Duration::from_millis(options.interval_ms);
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
    let _ = execute!(
        std::io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );
    let _restore = RestoreTerminal;
    let mut previous = Instant::now();
    let mut redraw_needed = true;
    let result = loop {
        while let Ok(sample) = rx.try_recv() {
            match sample {
                Ok(snapshot) => {
                    app.set_snapshot(snapshot);
                    if !app.note_editor {
                        redraw_needed = true;
                    }
                    if app.snapshot.processes.is_empty() {
                        detail_open = false;
                        detail_scroll = 0;
                    }
                }
                Err(error) => {
                    app.error = Some(error);
                    if !app.note_editor {
                        redraw_needed = true;
                    }
                }
            }
        }
        let now = Instant::now();
        if !app.note_editor {
            app.animate(now.duration_since(previous));
            redraw_needed = true;
        }
        previous = now;
        if redraw_needed {
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
            if !app.note_editor {
                // Keep native IME composition anchored in the empty outer gutter,
                // away from telemetry that is redrawn on every frame.
                let size = terminal.size()?;
                terminal.set_cursor_position((0, size.height.saturating_sub(1)))?;
            }
            redraw_needed = false;
        }

        if event::poll(FRAME_TIME)? {
            match event::read()? {
                Event::Key(key) => {
                    redraw_needed = true;
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
                            continue;
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
                            if app.note_editor
                                && !key.modifiers.contains(KeyModifiers::CONTROL)
                                && !is_hangul_jamo(character)
                            {
                                app.note_buffer.push(character);
                            } else {
                                // Korean IMEs send uncommitted jamo while composing.
                                // Let the terminal render that preedit without a TUI repaint.
                                redraw_needed = false;
                            }
                            // Keep IME characters away from dashboard shortcuts.
                            // Notes stores them as Unicode and renders by cell width.
                            continue;
                        }
                    }
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break Ok(());
                    }
                    if app.note_editor {
                        match key.code {
                            KeyCode::Enter if app.note_buffer.matches('\n').count() < 2 => {
                                app.note_buffer.push('\n')
                            }
                            KeyCode::Enter => {}
                            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::SUPER) => {
                                delete_to_line_start(&mut app.note_buffer);
                            }
                            KeyCode::Backspace
                                if key
                                    .modifiers
                                    .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL) =>
                            {
                                delete_previous_word(&mut app.note_buffer);
                            }
                            KeyCode::Backspace => delete_previous_grapheme(&mut app.note_buffer),
                            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                delete_previous_word(&mut app.note_buffer);
                            }
                            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                delete_to_line_start(&mut app.note_buffer);
                            }
                            KeyCode::Char(character)
                                if !key.modifiers.contains(KeyModifiers::CONTROL) =>
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
                            KeyCode::Up
                            | KeyCode::Left
                            | KeyCode::Char('k')
                            | KeyCode::Char('h') => {
                                let next =
                                    (selected + ThemeName::ALL.len() - 1) % ThemeName::ALL.len();
                                theme_name = ThemeName::ALL[next];
                                theme = ui::Theme::from_name(theme_name);
                                menu = Some((original, next));
                            }
                            KeyCode::Down
                            | KeyCode::Right
                            | KeyCode::Char('j')
                            | KeyCode::Char('l') => {
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
                        let columns = ui::process_columns(
                            terminal.size()?.width.saturating_sub(ui::PAGE_GUTTER_X * 2),
                        )
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
                            KeyCode::Up | KeyCode::Char('k') => {
                                if process_count == 0 && !app.snapshot.gpus.is_empty() {
                                    if !app.gpu_navigation_active {
                                        app.selected_gpu = 0;
                                        app.gpu_navigation_active = true;
                                    } else if let Some(selected) =
                                        ui::gpu_neighbor(&app, app.selected_gpu, columns, 0, -1)
                                    {
                                        app.selected_gpu = selected;
                                    } else {
                                        app.gpu_navigation_active = false;
                                    }
                                } else if app.selected_process > 0 {
                                    app.selected_process = ui::process_neighbor(
                                        &app,
                                        app.selected_process,
                                        columns,
                                        0,
                                        -1,
                                    )
                                    .unwrap_or(app.selected_process - 1);
                                    app.process_navigation_active = true;
                                } else {
                                    app.process_navigation_active = false;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if process_count == 0 && !app.snapshot.gpus.is_empty() {
                                    if !app.gpu_navigation_active {
                                        app.selected_gpu = 0;
                                        app.gpu_navigation_active = true;
                                    } else if let Some(selected) =
                                        ui::gpu_neighbor(&app, app.selected_gpu, columns, 0, 1)
                                    {
                                        app.selected_gpu = selected;
                                    }
                                } else if process_count > 0 && !app.process_navigation_active {
                                    app.process_navigation_active = true;
                                } else if app.selected_process < process_count.saturating_sub(1) {
                                    app.selected_process = ui::process_neighbor(
                                        &app,
                                        app.selected_process,
                                        columns,
                                        0,
                                        1,
                                    )
                                    .unwrap_or(app.selected_process + 1);
                                    app.process_navigation_active = true;
                                }
                            }
                            KeyCode::Left | KeyCode::Char('h') if columns > 1 => {
                                if process_count == 0 && !app.snapshot.gpus.is_empty() {
                                    if let Some(selected) =
                                        ui::gpu_neighbor(&app, app.selected_gpu, columns, -1, 0)
                                    {
                                        app.selected_gpu = selected;
                                        app.gpu_navigation_active = true;
                                    }
                                } else if let Some(selected) =
                                    ui::process_neighbor(&app, app.selected_process, columns, -1, 0)
                                {
                                    app.selected_process = selected;
                                    app.process_navigation_active = true;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') if columns > 1 => {
                                if process_count == 0 && !app.snapshot.gpus.is_empty() {
                                    if let Some(selected) =
                                        ui::gpu_neighbor(&app, app.selected_gpu, columns, 1, 0)
                                    {
                                        app.selected_gpu = selected;
                                        app.gpu_navigation_active = true;
                                    }
                                } else if let Some(selected) =
                                    ui::process_neighbor(&app, app.selected_process, columns, 1, 0)
                                {
                                    app.selected_process = selected;
                                    app.process_navigation_active = true;
                                }
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
                Event::Resize(_, _) => redraw_needed = true,
                _ => {}
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
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
        ratatui::restore();
    }
}

struct Options {
    interval_ms: u64,
    no_update: bool,
}

fn parse_args() -> Result<Options, Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let mut interval_ms = DEFAULT_INTERVAL_MS;
    let mut no_update = env::var("VIGPU_NO_UPDATE").is_ok_and(|value| {
        !value.is_empty() && !matches!(value.as_str(), "0" | "false" | "no" | "off")
    });
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("vigpu — cross-platform GPU monitor for NVIDIA and Apple Silicon\n\nUSAGE\n  vigpu [--interval-ms 250] [--no-update]\n\nOPTIONS\n  --interval-ms N  GPU sampling interval in milliseconds (100–5000; default 250)\n  --no-update      Skip the background update check for this run\n  -h, --help       Show this help\n  -V, --version    Show version\n\nSet VIGPU_NO_UPDATE=1 to disable update checks by default.");
                std::process::exit(0);
            }
            "--no-update" => no_update = true,
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
    Ok(Options {
        interval_ms,
        no_update,
    })
}

fn delete_previous_grapheme(text: &mut String) {
    if let Some((index, _)) = text.grapheme_indices(true).next_back() {
        text.truncate(index);
    }
}

fn delete_previous_word(text: &mut String) {
    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    let mut start = graphemes.len();
    while start > 0 && graphemes[start - 1].chars().all(char::is_whitespace) {
        start -= 1;
    }
    while start > 0 && !graphemes[start - 1].chars().all(char::is_whitespace) {
        start -= 1;
    }
    text.truncate(graphemes[..start].iter().map(|g| g.len()).sum());
}

fn delete_to_line_start(text: &mut String) {
    let line_start = text.rfind('\n').map_or(0, |index| index + 1);
    text.replace_range(line_start.., "");
}

fn is_hangul_jamo(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x11FF | 0x3130..=0x318F | 0xA960..=0xA97F | 0xD7B0..=0xD7FF
    )
}
