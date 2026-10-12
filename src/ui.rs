// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::env;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    gpu::GpuSample,
    state::{AppState, GpuVisual},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeName {
    ViGpuDefault,
    Alucard,
    AyuMirage,
    Catppuccin,
    Dracula,
    Everforest,
    GitHubDark,
    Kanagawa,
    Nightfox,
    Nord,
    OneDark,
    RosePine,
    Solarized,
    Sonokai,
    TokyoNight,
}

impl ThemeName {
    pub const ALL: [Self; 15] = [
        Self::ViGpuDefault,
        Self::Alucard,
        Self::AyuMirage,
        Self::Catppuccin,
        Self::Dracula,
        Self::Everforest,
        Self::GitHubDark,
        Self::Kanagawa,
        Self::Nightfox,
        Self::Nord,
        Self::OneDark,
        Self::RosePine,
        Self::Solarized,
        Self::Sonokai,
        Self::TokyoNight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ViGpuDefault => "ViGPU Default (Gruvbox)",
            Self::Alucard => "Alucard",
            Self::AyuMirage => "Ayu Mirage",
            Self::Catppuccin => "Catppuccin Mocha",
            Self::Dracula => "Dracula",
            Self::Everforest => "Everforest",
            Self::GitHubDark => "GitHub Dark",
            Self::Kanagawa => "Kanagawa",
            Self::Nightfox => "Nightfox",
            Self::Nord => "Nord",
            Self::OneDark => "One Dark",
            Self::RosePine => "Rosé Pine",
            Self::Solarized => "Solarized Dark",
            Self::Sonokai => "Sonokai",
            Self::TokyoNight => "Tokyo Night",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::ViGpuDefault => "vigpu-default",
            Self::Alucard => "alucard",
            Self::AyuMirage => "ayu-mirage",
            Self::Catppuccin => "catppuccin-mocha",
            Self::Dracula => "dracula",
            Self::Everforest => "everforest",
            Self::GitHubDark => "github-dark",
            Self::Kanagawa => "kanagawa",
            Self::Nightfox => "nightfox",
            Self::Nord => "nord",
            Self::OneDark => "one-dark",
            Self::RosePine => "rose-pine",
            Self::Solarized => "solarized-dark",
            Self::Sonokai => "sonokai",
            Self::TokyoNight => "tokyo-night",
        }
    }

    pub fn from_slug(value: &str) -> Option<Self> {
        if value == "gruvbox" || value == "gruvbox-material" {
            return Some(Self::ViGpuDefault);
        }
        Self::ALL.into_iter().find(|theme| theme.slug() == value)
    }

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|theme| *theme == self)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    bg: Color,
    panel: Color,
    border: Color,
    text: Color,
    muted: Color,
    yellow: Color,
    aqua: Color,
    purple: Color,
    glow_rgb: [u8; 3],
    yellow_rgb: [u8; 3],
    green_rgb: [u8; 3],
    red_rgb: [u8; 3],
    depth: u8,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        let term = env::var("TERM").unwrap_or_default();
        let color_term = env::var("COLORTERM").unwrap_or_default().to_lowercase();
        let depth = if color_term.contains("truecolor")
            || color_term.contains("24bit")
            || term.contains("direct")
        {
            24
        } else if term.contains("256color") {
            8
        } else {
            4
        };
        let palette: [[u8; 3]; 10] = match name {
            ThemeName::ViGpuDefault => [
                [40, 40, 40],
                [50, 48, 47],
                [80, 73, 69],
                [235, 219, 178],
                [168, 153, 132],
                [250, 189, 47],
                [142, 192, 124],
                [184, 187, 38],
                [251, 73, 52],
                [211, 134, 155],
            ],
            ThemeName::Nord => [
                [46, 52, 64],
                [59, 66, 82],
                [76, 86, 106],
                [236, 239, 244],
                [163, 177, 194],
                [235, 203, 139],
                [136, 192, 208],
                [163, 190, 140],
                [191, 97, 106],
                [180, 142, 173],
            ],
            ThemeName::AyuMirage => [
                [36, 41, 54],
                [31, 36, 48],
                [56, 66, 84],
                [204, 202, 194],
                [112, 122, 140],
                [252, 202, 96],
                [147, 226, 200],
                [135, 217, 108],
                [242, 130, 115],
                [221, 187, 255],
            ],
            ThemeName::OneDark => [
                [40, 44, 52],
                [33, 37, 43],
                [62, 68, 81],
                [171, 178, 191],
                [92, 99, 112],
                [229, 192, 123],
                [86, 182, 194],
                [152, 195, 121],
                [224, 108, 117],
                [198, 120, 221],
            ],
            ThemeName::GitHubDark => [
                [13, 17, 23],
                [22, 27, 34],
                [48, 54, 61],
                [230, 237, 243],
                [145, 152, 161],
                [210, 153, 34],
                [121, 192, 255],
                [63, 185, 80],
                [248, 81, 73],
                [210, 168, 255],
            ],
            ThemeName::TokyoNight => [
                [26, 27, 38],
                [36, 40, 59],
                [65, 72, 104],
                [192, 202, 245],
                [154, 165, 206],
                [224, 175, 104],
                [125, 207, 255],
                [158, 206, 106],
                [247, 118, 142],
                [187, 154, 247],
            ],
            ThemeName::Catppuccin => [
                [30, 30, 46],
                [49, 50, 68],
                [69, 71, 90],
                [205, 214, 244],
                [166, 173, 200],
                [249, 226, 175],
                [148, 226, 213],
                [166, 227, 161],
                [243, 139, 168],
                [203, 166, 247],
            ],
            ThemeName::Solarized => [
                [0, 43, 54],
                [7, 54, 66],
                [88, 110, 117],
                [238, 232, 213],
                [147, 161, 161],
                [181, 137, 0],
                [42, 161, 152],
                [133, 153, 0],
                [220, 50, 47],
                [108, 113, 196],
            ],
            ThemeName::Sonokai => [
                [44, 46, 52],
                [51, 53, 63],
                [89, 95, 111],
                [226, 226, 227],
                [127, 132, 144],
                [231, 198, 100],
                [118, 204, 224],
                [158, 208, 114],
                [252, 93, 124],
                [179, 157, 243],
            ],
            ThemeName::RosePine => [
                [25, 23, 36],
                [38, 35, 58],
                [64, 61, 82],
                [224, 222, 244],
                [144, 140, 170],
                [246, 193, 119],
                [156, 207, 216],
                [49, 116, 143],
                [235, 111, 146],
                [196, 167, 231],
            ],
            ThemeName::Dracula => [
                [40, 42, 54],
                [68, 71, 90],
                [98, 114, 164],
                [248, 248, 242],
                [189, 147, 249],
                [241, 250, 140],
                [139, 233, 253],
                [80, 250, 123],
                [255, 85, 85],
                [189, 147, 249],
            ],
            ThemeName::Alucard => [
                [255, 251, 235],
                [226, 222, 202],
                [108, 102, 75],
                [31, 31, 31],
                [108, 102, 75],
                [132, 110, 21],
                [3, 106, 150],
                [20, 113, 10],
                [203, 58, 42],
                [100, 74, 201],
            ],
            ThemeName::Kanagawa => [
                [31, 31, 40],
                [42, 42, 55],
                [84, 84, 109],
                [220, 215, 186],
                [114, 113, 105],
                [230, 195, 132],
                [127, 180, 202],
                [152, 187, 108],
                [228, 104, 118],
                [149, 127, 184],
            ],
            ThemeName::Nightfox => [
                [25, 35, 48],
                [33, 46, 63],
                [57, 80, 109],
                [205, 206, 207],
                [113, 131, 155],
                [219, 192, 116],
                [99, 205, 207],
                [129, 178, 154],
                [201, 79, 109],
                [157, 121, 214],
            ],
            ThemeName::Everforest => [
                [45, 53, 59],
                [52, 63, 68],
                [71, 82, 88],
                [211, 198, 170],
                [133, 146, 137],
                [219, 188, 127],
                [127, 187, 179],
                [167, 192, 128],
                [230, 126, 128],
                [214, 153, 182],
            ],
        };
        let color = |index| mapped_color(palette[index], depth);
        Self {
            bg: color(0),
            panel: color(1),
            border: color(2),
            text: color(3),
            muted: color(4),
            yellow: color(5),
            aqua: color(6),
            purple: color(9),
            glow_rgb: palette[3],
            yellow_rgb: palette[5],
            green_rgb: palette[7],
            red_rgb: palette[8],
            depth,
        }
    }

    fn glow(self, brightness: f64) -> Color {
        let rgb = self
            .glow_rgb
            .map(|channel| (channel as f64 * brightness).round().clamp(0.0, 255.0) as u8);
        mapped_color(rgb, self.depth)
    }

    fn heat(self, percent: f64) -> Color {
        mapped_color(self.heat_rgb(percent), self.depth)
    }

    fn heat_rgb(self, percent: f64) -> [u8; 3] {
        if percent >= 90.0 {
            self.green_rgb
        } else if percent >= 70.0 {
            self.yellow_rgb
        } else {
            self.red_rgb
        }
    }

    fn blend_rgb(self, from: [u8; 3], to: [u8; 3], amount: f64) -> [u8; 3] {
        let amount = amount.clamp(0.0, 1.0);
        let rgb = std::array::from_fn(|index| {
            (from[index] as f64 + (to[index] as f64 - from[index] as f64) * amount)
                .round()
                .clamp(0.0, 255.0) as u8
        });
        rgb
    }
}

fn mapped_color([r, g, b]: [u8; 3], depth: u8) -> Color {
    match depth {
        24 => Color::Rgb(r, g, b),
        8 => {
            let cube = |v: u8| ((v as f64 / 255.0 * 5.0).round() as u8).min(5);
            let cube_index = 16 + 36 * cube(r) + 6 * cube(g) + cube(b);
            let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
            let gray_index =
                232 + ((gray.saturating_sub(8) as f64 / 247.0 * 24.0).round() as u8).min(23);
            let cube_rgb = [r, g, b].map(|v| {
                let n = cube(v);
                if n == 0 {
                    0
                } else {
                    55 + 40 * n
                }
            });
            let gray_value = 8 + 10 * (gray_index - 232);
            let cube_error: u32 = [r, g, b]
                .iter()
                .zip(cube_rgb)
                .map(|(a, b)| (*a as i32 - b as i32).unsigned_abs())
                .sum();
            let gray_error: u32 = [r, g, b]
                .iter()
                .map(|v| (*v as i32 - gray_value as i32).unsigned_abs())
                .sum();
            Color::Indexed(if gray_error < cube_error {
                gray_index
            } else {
                cube_index
            })
        }
        _ => {
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            if max.saturating_sub(min) < 24 {
                if max < 45 {
                    Color::Black
                } else if max < 100 {
                    Color::DarkGray
                } else if max < 190 {
                    Color::Gray
                } else {
                    Color::White
                }
            } else if r >= g && r >= b {
                if g > 110 {
                    Color::Yellow
                } else if b > 110 {
                    Color::Magenta
                } else {
                    Color::Red
                }
            } else if g >= r && g >= b {
                if b > 110 {
                    Color::Cyan
                } else {
                    Color::Green
                }
            } else if r > 110 {
                Color::Magenta
            } else {
                Color::Blue
            }
        }
    }
}

pub fn render(
    frame: &mut Frame,
    app: &AppState,
    theme: Theme,
    menu: Option<usize>,
    detail_open: bool,
    detail_scroll: usize,
) {
    let screen = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        screen,
    );
    let area = Rect {
        x: screen.x + 2.min(screen.width),
        y: screen.y + 1.min(screen.height),
        width: screen.width.saturating_sub(4),
        height: screen.height.saturating_sub(2),
    };
    if area.width < 36 || area.height < (8 + app.snapshot.gpus.len() as u16 * 4).min(20) {
        render_tiny(frame, app, theme, area);
        if detail_open {
            render_process_detail(frame, app, theme, detail_scroll);
        } else if let Some(selected) = menu {
            render_theme_menu(frame, theme, screen, selected);
        }
        if app.note_editor {
            render_notes_editor(frame, app, theme, screen);
        }
        if !detail_open && menu.is_none() && !app.note_editor {
            render_dashboard_hint(frame, app, theme, screen);
        }
        return;
    }

    let header_height = 1.min(area.height);
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(header_height), Constraint::Min(1)])
        .split(area);
    render_header(frame, app, theme, content[0]);

    let body = content[1];
    let columns = process_columns(body.width) as usize;
    let gpu_rows = app.snapshot.gpus.len().div_ceil(columns).max(1);
    let ideal_card = if body.height >= 24 { 8 } else { 6 };
    let section_gap = 0;
    let notes_space = notes_height(app, body.width);
    let minimum_runs = 1
        + notes_space
        + if app.snapshot.processes.is_empty() {
            1
        } else {
            3
        };
    let mut minimum_card_height = 5;
    let capacity_for_cards = body
        .height
        .saturating_sub(1)
        .saturating_sub(gpu_rows as u16 * minimum_card_height)
        .saturating_sub(section_gap);
    if capacity_for_cards < minimum_runs {
        minimum_card_height = 3;
    }
    let process_capacity = body
        .height
        .saturating_sub(1)
        .saturating_sub(gpu_rows as u16 * minimum_card_height)
        .saturating_sub(section_gap);
    let requested_process_area = process_area_height(app, body.width);
    let process_area_height = if process_capacity >= minimum_runs {
        requested_process_area.min(process_capacity)
    } else if process_capacity >= 1 + notes_space {
        process_capacity
    } else {
        0
    };
    let reserved_processes = if process_area_height > 0 {
        process_area_height + section_gap
    } else {
        0
    };
    let overview_height = 1;
    let cards_space = body
        .height
        .saturating_sub(overview_height)
        .saturating_sub(reserved_processes);
    let card_height = (cards_space / gpu_rows as u16)
        .min(ideal_card)
        .max(minimum_card_height);
    let cards_height = (gpu_rows as u16 * card_height).min(body.height);
    let overview = section_heading("GPUs", body.width, theme);
    frame.render_widget(
        overview,
        Rect {
            y: body.y,
            height: overview_height,
            ..body
        },
    );
    let cards_area = Rect {
        y: body.y + overview_height,
        height: cards_height,
        ..body
    };
    let process_y = body.y
        + overview_height
        + cards_height
        + if reserved_processes > 0 {
            section_gap
        } else {
            0
        };
    let process_area = Rect {
        y: process_y,
        height: body.y + body.height - process_y,
        ..body
    };
    render_cards(frame, app, theme, cards_area, columns, card_height);
    if process_area.height >= 3 {
        render_processes(frame, app, theme, process_area);
    }
    if detail_open {
        render_process_detail(frame, app, theme, detail_scroll);
    } else if let Some(selected) = menu {
        render_theme_menu(frame, theme, screen, selected);
    }
    if app.note_editor {
        render_notes_editor(frame, app, theme, screen);
    }
    if !detail_open && menu.is_none() && !app.note_editor {
        render_dashboard_hint(frame, app, theme, screen);
    }
}

fn process_area_height(app: &AppState, width: u16) -> u16 {
    if app.snapshot.processes.is_empty() {
        return 2 + notes_height(app, width);
    }
    let columns = process_columns(width) as usize;
    let rows = process_grid_rows(app, columns);
    let rows_height = rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .filter_map(|(column, entry)| {
                    let (_, card_width) = grid_column_span(0, width, columns, column);
                    entry.map(|(_, process)| process_card_height(app, process, card_width))
                })
                .max()
                .unwrap_or(3) as usize
        })
        .sum::<usize>();
    (1 + rows_height + notes_height(app, width) as usize).min(u16::MAX as usize) as u16
}

fn notes_height(app: &AppState, width: u16) -> u16 {
    if app.notes.trim().is_empty() || width == 0 {
        return 0;
    }
    let line_count = app
        .notes
        .lines()
        .map(|line| wrap_text(line, width.saturating_sub(2) as usize).len())
        .sum::<usize>()
        .max(1);
    (line_count + 1).min(u16::MAX as usize) as u16
}

fn grid_column_span(start: u16, width: u16, columns: usize, column: usize) -> (u16, u16) {
    let columns = columns.max(1);
    let gutter = usize::from(columns > 1) * 2;
    let usable = (width as usize).saturating_sub(gutter * (columns - 1));
    let left = gutter * column + usable * column / columns;
    let right = gutter * column + usable * (column + 1) / columns;
    (start + left as u16, (right - left) as u16)
}

pub fn process_columns(width: u16) -> u16 {
    if width >= 220 {
        4
    } else if width >= 160 {
        3
    } else if width >= 76 {
        2
    } else {
        1
    }
}

fn process_grid_rows<'a>(
    app: &'a AppState,
    columns: usize,
) -> Vec<Vec<Option<(usize, &'a crate::gpu::GpuProcess)>>> {
    let columns = columns.max(1);
    let mut grouped = vec![Vec::new(); columns];
    for (index, process) in app.snapshot.processes.iter().enumerate() {
        grouped[process.gpu as usize % columns].push((index, process));
    }
    let row_count = grouped.iter().map(Vec::len).max().unwrap_or(0);
    (0..row_count)
        .map(|row| {
            grouped
                .iter()
                .map(|column| column.get(row).copied())
                .collect()
        })
        .collect()
}

pub fn process_neighbor(
    app: &AppState,
    selected: usize,
    columns: usize,
    delta_column: isize,
    delta_row: isize,
) -> Option<usize> {
    let rows = process_grid_rows(app, columns);
    let (row, column) = rows.iter().enumerate().find_map(|(row_index, row)| {
        row.iter().enumerate().find_map(|(column_index, process)| {
            process
                .filter(|(index, _)| *index == selected)
                .map(|_| (row_index, column_index))
        })
    })?;
    if delta_column != 0 {
        let mut next_column = column;
        loop {
            next_column = next_column.checked_add_signed(delta_column)?;
            let Some(entry) = rows.get(row)?.get(next_column)? else {
                continue;
            };
            return Some(entry.0);
        }
    }
    let mut next_row = row;
    loop {
        next_row = next_row.checked_add_signed(delta_row)?;
        let Some(entry) = rows.get(next_row)?.get(column)? else {
            continue;
        };
        return Some(entry.0);
    }
}

fn process_card_height(app: &AppState, process: &crate::gpu::GpuProcess, width: u16) -> u16 {
    let inner_width = width.saturating_sub(3) as usize;
    let detail_lines = process_detail_lines(app, process, inner_width).len();
    (detail_lines + 2) as u16
}

fn process_detail_lines(
    app: &AppState,
    process: &crate::gpu::GpuProcess,
    width: usize,
) -> Vec<String> {
    let vram = app
        .snapshot
        .gpus
        .iter()
        .find(|gpu| gpu.index == process.gpu)
        .and_then(|gpu| match (process.memory_bytes, gpu.memory_total) {
            (Some(used), Some(total)) if total > 0 => {
                Some(format!("VRAM {:.0}%", used as f64 / total as f64 * 100.0))
            }
            _ => None,
        })
        .unwrap_or_else(|| "VRAM --%".into());
    let cpu = process
        .cpu_percent
        .map(|percent| format!("CPU {percent:.0}%"))
        .unwrap_or_else(|| "CPU --%".into());
    let mut lines = Vec::new();
    let groups = [
        vec![
            format!("GPU {}:", process.gpu),
            format!("PID {}", process.pid),
            format!("USER {}", process.user),
        ],
        vec![cpu, vram],
    ];
    for group in groups {
        let mut line = String::new();
        for field in group {
            let separator = if line.is_empty() {
                ""
            } else if line.ends_with(':') {
                " "
            } else {
                " · "
            };
            let candidate = format!("{line}{separator}{field}");
            if UnicodeWidthStr::width(candidate.as_str()) <= width {
                line = candidate;
            } else {
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                line = truncate(&field, width);
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

fn process_executable(command: &str) -> &str {
    command
        .split_whitespace()
        .next()
        .unwrap_or(command)
        .rsplit('/')
        .next()
        .unwrap_or(command)
}

fn process_subtitle(command: &str) -> Option<String> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let mut index = 1;
    while let Some(argument) = args.get(index) {
        match *argument {
            "-m" => return args.get(index + 1).map(|module| (*module).to_owned()),
            "-c" => return Some("inline Python".to_owned()),
            "-W" | "-X" => index += 2,
            value if value.starts_with('-') => index += 1,
            value if !value.is_empty() => {
                return Some(value.rsplit('/').next().unwrap_or(value).to_owned())
            }
            _ => index += 1,
        }
    }
    None
}

fn render_header(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let title_width = area.width.saturating_sub(14);
    frame.render_widget(
        Paragraph::new(wordmark_line(theme, app.elapsed)).style(Style::default().bg(theme.bg)),
        Rect {
            width: title_width.max(1),
            height: 1,
            ..area
        },
    );
    frame.render_widget(
        Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Right)
            .style(Style::default().fg(theme.text).bg(theme.bg)),
        Rect {
            x: area.x + title_width,
            width: area.width - title_width,
            y: area.y,
            height: 1,
        },
    );
}

fn render_dashboard_hint(frame: &mut Frame, app: &AppState, theme: Theme, screen: Rect) {
    if screen.width == 0 || screen.height == 0 {
        return;
    }
    let hint = match (&app.notice, &app.error) {
        (Some(message), _) => format!("! {message}"),
        (_, Some(message)) => format!("! {message} · retrying"),
        _ => "arrows select · enter details · n notes · t themes · q exit".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(format!(
            "{} ",
            truncate(&hint, screen.width.saturating_sub(1) as usize)
        ))
        .alignment(Alignment::Right)
        .style(Style::default().fg(theme.muted).bg(theme.bg)),
        Rect {
            x: screen.x,
            y: screen.y + screen.height - 1,
            width: screen.width,
            height: 1,
        },
    );
}

fn section_heading(label: &str, width: u16, theme: Theme) -> Paragraph<'static> {
    let divider_width = (width as usize).saturating_sub(label.len() + 2);
    Paragraph::new(Line::from(vec![
        Span::styled(
            label.to_owned(),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("─".repeat(divider_width), Style::default().fg(theme.border)),
    ]))
}

fn wordmark_line(theme: Theme, elapsed: std::time::Duration) -> Line<'static> {
    let phase = (elapsed.as_secs_f64() * 2.4) % 7.5 - 1.25;
    let mut spans = vec![
        Span::styled(
            "▰",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "▱",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    spans.extend("ViGPU".chars().enumerate().map(|(index, glyph)| {
        let distance = index as f64 - phase;
        let brightness = 0.68 + 0.58 * (-distance * distance / 1.25).exp();
        Span::styled(
            glyph.to_string(),
            Style::default()
                .fg(theme.glow(brightness))
                .add_modifier(Modifier::BOLD),
        )
    }));
    Line::from(spans)
}

fn render_theme_menu(frame: &mut Frame, theme: Theme, screen: Rect, selected: usize) {
    if screen.width < 18 || screen.height < 5 {
        let name = ThemeName::ALL[selected].label();
        frame.render_widget(
            Paragraph::new(format!(" theme: {name}  ↑↓ enter · esc close"))
                .style(Style::default().fg(theme.text).bg(theme.panel)),
            Rect {
                y: screen.y + screen.height.saturating_sub(1),
                height: 1,
                ..screen
            },
        );
        return;
    }
    let width = screen.width.saturating_sub(2).min(44);
    let height = screen
        .height
        .saturating_sub(2)
        .min(14)
        .min(ThemeName::ALL.len() as u16 + 3);
    let area = Rect {
        x: screen.x + (screen.width - width) / 2,
        y: screen.y + (screen.height - height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme.text))
        .title(Span::styled(
            " ▰ Themes ",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel));
    let inner = block.inner(area).inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let visible = inner.height.saturating_sub(1) as usize;
    let start = selected
        .saturating_sub(visible.saturating_sub(1) / 2)
        .min(ThemeName::ALL.len().saturating_sub(visible));
    let list_area = Rect {
        width: inner.width.saturating_sub(1),
        ..inner
    };
    for (row, index) in (start..ThemeName::ALL.len()).take(visible).enumerate() {
        let name = ThemeName::ALL[index];
        let preview = Theme::from_name(name);
        let active = index == selected;
        let label_width = list_area
            .width
            .saturating_sub(if list_area.width >= 32 { 7 } else { 0 })
            as usize;
        let mut spans = vec![Span::styled(
            format!(
                "{:<width$}",
                truncate(name.label(), label_width),
                width = label_width
            ),
            Style::default()
                .fg(if active { theme.text } else { theme.muted })
                .add_modifier(if active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )];
        if list_area.width >= 32 {
            spans.push(Span::styled("■ ", Style::default().fg(preview.yellow)));
            spans.push(Span::styled("■ ", Style::default().fg(preview.aqua)));
            spans.push(Span::styled("■", Style::default().fg(preview.purple)));
        }
        let line = Line::from(spans);
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(theme.panel)),
            Rect {
                y: inner.y + row as u16,
                height: 1,
                ..list_area
            },
        );
    }
    if visible > 0 && inner.width > 0 {
        let track_height = visible;
        let total = ThemeName::ALL.len();
        let thumb_height = (track_height * track_height / total).max(1);
        let max_start = total.saturating_sub(track_height);
        let thumb_start = if max_start == 0 {
            0
        } else {
            start * track_height.saturating_sub(thumb_height) / max_start
        };
        for row in 0..track_height {
            let thumb = (thumb_start..thumb_start + thumb_height).contains(&row);
            frame.render_widget(
                Paragraph::new("│").style(
                    Style::default()
                        .fg(if thumb { theme.text } else { theme.border })
                        .bg(theme.panel),
                ),
                Rect {
                    x: inner.x + inner.width - 1,
                    y: inner.y + row as u16,
                    width: 1,
                    height: 1,
                },
            );
        }
    }
    if inner.height >= 2 {
        frame.render_widget(
            Paragraph::new("↑↓ scroll · enter save · esc close ")
                .alignment(Alignment::Right)
                .style(Style::default().fg(theme.muted).bg(theme.panel)),
            Rect {
                y: inner.y + inner.height - 1,
                height: 1,
                ..inner
            },
        );
    }
}

fn render_notes_editor(frame: &mut Frame, app: &AppState, theme: Theme, screen: Rect) {
    if screen.width < 12 || screen.height < 5 {
        return;
    }
    let width = screen.width.saturating_sub(4).min(72);
    let height = screen.height.saturating_sub(2).min(14);
    let area = Rect {
        x: screen.x + (screen.width - width) / 2,
        y: screen.y + (screen.height - height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme.text))
        .title(Span::styled(
            " ▰ Notes ",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let mut content = app.note_buffer.clone();
    content.push('▏');
    frame.render_widget(
        Paragraph::new(content).style(Style::default().fg(theme.text).bg(theme.panel)),
        Rect {
            height: inner.height.saturating_sub(1),
            ..inner
        },
    );
    if inner.height > 1 {
        frame.render_widget(
            Paragraph::new("type notes · enter new line (max 3) · esc save ")
                .alignment(Alignment::Right)
                .style(Style::default().fg(theme.muted).bg(theme.panel)),
            Rect {
                y: inner.y + inner.height - 1,
                height: 1,
                ..inner
            },
        );
    }
}

fn render_process_detail(frame: &mut Frame, app: &AppState, theme: Theme, scroll: usize) {
    let Some(process) = app.snapshot.processes.get(app.selected_process) else {
        return;
    };
    let screen = frame.area();
    let width = screen.width.saturating_sub(4).min(100).max(1);
    let inner_width = width.saturating_sub(4) as usize;
    let mut content = vec![Line::styled(
        format!(" {}", process_executable(&process.name)),
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    )];
    content.extend(
        process_detail_lines(app, process, inner_width)
            .into_iter()
            .map(|line| Line::styled(format!(" {line}"), Style::default().fg(theme.muted))),
    );
    content.push(Line::raw(""));
    content.push(Line::styled(
        " command",
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    ));
    content.extend(
        wrap_text(&process.name, inner_width.saturating_sub(2))
            .into_iter()
            .map(|line| Line::styled(format!("  {line}"), Style::default().fg(theme.text))),
    );

    let desired_height = (content.len() as u16 + 3).min(screen.height.saturating_sub(2).max(1));
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(width) / 2,
        y: screen.y + screen.height.saturating_sub(desired_height) / 2,
        width,
        height: desired_height,
    };
    frame.render_widget(Clear, area);
    let filename = process_subtitle(&process.name)
        .unwrap_or_else(|| process_executable(&process.name).to_owned());
    let title = Line::from(vec![
        Span::styled(" ▰ ", Style::default().fg(theme.text)),
        Span::styled(
            format!("{} {filename} ", process_executable(&process.name)),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme.text))
        .title(title)
        .style(Style::default().bg(theme.panel));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let footer_rows = usize::from(inner.height > 1);
    let visible_rows = inner.height.saturating_sub(footer_rows as u16) as usize;
    let max_scroll = content.len().saturating_sub(visible_rows);
    frame.render_widget(
        Paragraph::new(Text::from(content))
            .scroll((scroll.min(max_scroll).min(u16::MAX as usize) as u16, 0))
            .style(Style::default().bg(theme.panel)),
        Rect {
            height: visible_rows as u16,
            ..inner
        },
    );
    if footer_rows > 0 {
        frame.render_widget(
            Paragraph::new("↑↓ scroll · enter / esc close ")
                .alignment(Alignment::Right)
                .style(Style::default().fg(theme.muted).bg(theme.panel)),
            Rect {
                y: inner.y + inner.height - 1,
                height: 1,
                ..inner
            },
        );
    }
}

fn render_cards(
    frame: &mut Frame,
    app: &AppState,
    theme: Theme,
    area: Rect,
    columns: usize,
    card_height: u16,
) {
    if app.snapshot.gpus.is_empty() {
        frame.render_widget(
            Paragraph::new("  No NVIDIA GPUs found.").style(Style::default().fg(theme.yellow)),
            area,
        );
        return;
    }
    for (i, (gpu, visual)) in app.snapshot.gpus.iter().zip(&app.visuals).enumerate() {
        let row = i / columns;
        let col = i % columns;
        let (x, width) = grid_column_span(area.x, area.width, columns, col);
        let y = area.y + row as u16 * card_height;
        if y >= area.y + area.height {
            break;
        }
        let rect = Rect {
            x,
            y,
            width,
            height: card_height.min(area.y + area.height - y),
        };
        let active = app
            .snapshot
            .processes
            .iter()
            .any(|process| process.gpu == gpu.index);
        render_gpu(
            frame,
            gpu,
            visual,
            theme,
            rect,
            app.elapsed.as_secs_f64(),
            active,
        );
    }
}

fn render_gpu(
    frame: &mut Frame,
    gpu: &GpuSample,
    visual: &GpuVisual,
    theme: Theme,
    area: Rect,
    elapsed: f64,
    active: bool,
) {
    if area.width < 10 || area.height < 3 {
        return;
    }
    let title_label = truncate(
        &format!("GPU {} {}", gpu.index, gpu.name),
        area.width.saturating_sub(12) as usize,
    );
    let card_text = if active { theme.text } else { theme.muted };
    let card_border = if active { theme.text } else { theme.border };
    let title = Line::from(vec![
        Span::styled(
            if active { " ▰ " } else { " ▱ " },
            Style::default().fg(card_text),
        ),
        Span::styled(
            format!("{title_label} "),
            Style::default().fg(card_text).add_modifier(Modifier::BOLD),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(card_border))
        .title(title)
        .style(Style::default().bg(theme.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let data = inner;
    let wide = inner.width >= 42;
    let bar_width = if wide {
        (data.width as usize).saturating_sub(27).clamp(6, 30)
    } else {
        (data.width as usize).saturating_sub(15).max(3)
    };
    let util = gpu
        .utilization
        .map(|v| format!("{:>3}%", v))
        .unwrap_or_else(|| " --%".into());
    let memory_pct = match (gpu.memory_used, gpu.memory_total) {
        (Some(used), Some(total)) if total > 0 => Some(used as f64 / total as f64 * 100.0),
        _ => None,
    };
    let mem = memory_pct
        .map(|v| format!("{:>3.0}%", v))
        .unwrap_or_else(|| " --%".into());
    let util_extra = if wide {
        gpu.memory_utilization.map(|v| format!(" MEM I/O {v}%"))
    } else {
        None
    };
    let mem_extra = if wide {
        match (gpu.memory_used, gpu.memory_total) {
            (Some(used), Some(total)) => Some(format!(" {:.1}/{:.1}G", gib(used), gib(total))),
            _ => None,
        }
    } else {
        None
    };
    let util_line = meter(
        "UTIL",
        &util,
        visual.shown_util,
        bar_width,
        theme.heat(visual.shown_util),
        theme,
        util_extra,
    );
    let mem_line = meter(
        if gpu.unified_memory { "MEM " } else { "VRAM" },
        &mem,
        visual.shown_memory,
        bar_width,
        theme.heat(visual.shown_memory),
        theme,
        mem_extra,
    );
    let power_percent = match (gpu.power_w, gpu.power_limit_w) {
        (Some(used), Some(limit)) if limit > 0.0 => Some(used / limit * 100.0),
        _ => None,
    };
    let power_value = power_percent
        .map(|value| format!("{:>3.0}%", value))
        .unwrap_or_else(|| " --%".into());
    let power_extra = match (gpu.power_w, gpu.power_limit_w) {
        (Some(used), Some(limit)) => Some(format!(" {used:.0}/{limit:.0}W")),
        (Some(used), _) => Some(format!(" {used:.0}W")),
        _ => Some(" --W".into()),
    };
    let power_line = meter(
        "PWR ",
        &power_value,
        power_percent.unwrap_or(0.0),
        bar_width,
        theme.heat(power_percent.unwrap_or(0.0)),
        theme,
        power_extra,
    );
    frame.render_widget(Paragraph::new(util_line), Rect { height: 1, ..data });
    if inner.height >= 2 {
        frame.render_widget(
            Paragraph::new(mem_line),
            Rect {
                y: inner.y + 1,
                height: 1,
                ..data
            },
        );
    }

    if inner.height >= 3 {
        frame.render_widget(
            Paragraph::new(power_line),
            Rect {
                y: inner.y + 2,
                height: 1,
                ..data
            },
        );
    }

    if inner.height >= 4 {
        let graph_height = data.height.saturating_sub(4);
        let graph_area = Rect {
            y: data.y + 3,
            height: graph_height,
            ..data
        };
        render_graph(frame, visual, theme, graph_area, elapsed);
        let temperature = gpu
            .temperature
            .map(|value| format!("{value}°C"))
            .unwrap_or_else(|| "--°C".into());
        let fan = gpu
            .fan_percent
            .map(|percent| format!("{percent}%"))
            .unwrap_or_else(|| "n/a".into());
        let details = format!(" TEMP {temperature} · FAN {fan}");
        frame.render_widget(
            Paragraph::new(truncate(&details, data.width as usize))
                .style(Style::default().fg(theme.muted)),
            Rect {
                y: data.y + data.height - 1,
                height: 1,
                ..data
            },
        );
    }
}

fn meter(
    label: &str,
    value: &str,
    percent: f64,
    width: usize,
    color: Color,
    theme: Theme,
    extra: Option<String>,
) -> Line<'static> {
    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * width as f64).round() as usize;
    let mut spans = vec![
        Span::styled(format!(" {label} "), Style::default().fg(theme.muted)),
        Span::styled(
            value.to_owned(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    for i in 0..width {
        let glyph = if i < filled { "━" } else { "─" };
        let dot_color = if i < filled { color } else { theme.border };
        spans.push(Span::styled(glyph, Style::default().fg(dot_color)));
    }
    if let Some(extra) = extra {
        spans.push(Span::styled(extra, Style::default().fg(theme.muted)));
    }
    Line::from(spans)
}

fn render_graph(frame: &mut Frame, visual: &GpuVisual, theme: Theme, area: Rect, elapsed: f64) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let width = area.width as usize;
    let height = area.height as usize;
    let mut dots = vec![0u8; width * height];
    let history_len = visual.history.len();
    let sample_count = history_len.min(width * 2);
    let start_x = width * 2 - sample_count;
    let max_y = height * 4 - 1;
    let mut previous: Option<(usize, usize)> = None;
    for (offset, value) in visual
        .history
        .iter()
        .skip(history_len - sample_count)
        .enumerate()
    {
        let x = start_x + offset;
        let y = ((100 - (*value).min(100)) as usize * max_y + 50) / 100;
        if let Some((px, py)) = previous {
            let low = py.min(y);
            let high = py.max(y);
            for iy in low..=high {
                set_dot(&mut dots, width, px, iy);
            }
        }
        set_dot(&mut dots, width, x, y);
        previous = Some((x, y));
    }
    let sweep = (elapsed * 12.0).rem_euclid(width as f64 + 12.0) - 6.0;
    let buffer = frame.buffer_mut();
    for row in 0..height {
        for column in 0..width {
            let bits = dots[row * width + column];
            let glyph = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
            let distance = column as f64 - sweep;
            let glow = (-distance * distance / 18.0).exp() * 0.88;
            let center = (-distance * distance / 40.0).exp();
            let shimmer = theme.blend_rgb(theme.yellow_rgb, theme.glow_rgb, center);
            let rgb = theme.blend_rgb(theme.green_rgb, shimmer, glow);
            buffer[(area.x + column as u16, area.y + row as u16)]
                .set_char(glyph)
                .set_fg(mapped_color(rgb, theme.depth));
        }
    }
}

fn set_dot(dots: &mut [u8], width: usize, x: usize, y: usize) {
    let row = y / 4;
    let col = x / 2;
    if let Some(cell) = dots.get_mut(row * width + col) {
        const LEFT: [u8; 4] = [0x01, 0x02, 0x04, 0x40];
        const RIGHT: [u8; 4] = [0x08, 0x10, 0x20, 0x80];
        *cell |= if x.is_multiple_of(2) {
            LEFT[y % 4]
        } else {
            RIGHT[y % 4]
        };
    }
}

fn render_processes(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) {
    frame.render_widget(
        section_heading("Runs", area.width, theme),
        Rect { height: 1, ..area },
    );
    let cards_area = Rect {
        y: area.y + 1,
        height: area.height.saturating_sub(1),
        ..area
    };
    let note_lines = wrapped_notes(app, cards_area.width.saturating_sub(2) as usize);
    let note_height = if note_lines.is_empty() {
        0
    } else {
        note_lines.len() as u16 + 1
    };
    let notes_y = cards_area
        .y
        .saturating_add(cards_area.height.saturating_sub(note_height));
    let runs_bottom = if note_height > 0 {
        notes_y
    } else {
        cards_area.y + cards_area.height
    };
    let runs_height = runs_bottom.saturating_sub(cards_area.y);
    if app.snapshot.processes.is_empty() {
        if runs_height > 0 {
            #[cfg(target_os = "macos")]
            let empty_label = " Per-process GPU readings are unavailable for Apple GPUs";
            #[cfg(not(target_os = "macos"))]
            let empty_label = " No active GPU processes";
            frame.render_widget(
                Paragraph::new(empty_label).style(Style::default().fg(theme.muted).bg(theme.bg)),
                Rect {
                    height: 1,
                    ..Rect {
                        height: runs_height,
                        ..cards_area
                    }
                },
            );
        }
    }
    let columns = process_columns(cards_area.width) as usize;
    let rows = process_grid_rows(app, columns);
    let row_heights = rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .filter_map(|(column, entry)| {
                    let (_, card_width) = grid_column_span(0, cards_area.width, columns, column);
                    entry.map(|(_, process)| process_card_height(app, process, card_width))
                })
                .max()
                .unwrap_or(3)
        })
        .collect::<Vec<_>>();
    let viewport_height = runs_bottom.saturating_sub(cards_area.y);
    let scrollable = row_heights
        .iter()
        .map(|height| *height as usize)
        .sum::<usize>()
        > viewport_height as usize;
    let process_width = cards_area.width.saturating_sub(u16::from(scrollable));
    let row_heights = if scrollable {
        rows.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter_map(|(column, entry)| {
                        let (_, card_width) = grid_column_span(0, process_width, columns, column);
                        entry.map(|(_, process)| process_card_height(app, process, card_width))
                    })
                    .max()
                    .unwrap_or(3)
            })
            .collect::<Vec<_>>()
    } else {
        row_heights
    };
    let selected_row = rows
        .iter()
        .position(|row| {
            row.iter()
                .flatten()
                .any(|(index, _)| *index == app.selected_process)
        })
        .unwrap_or(0);
    let mut start_row = 0;
    let mut selected_offset = row_heights
        .iter()
        .take(selected_row + 1)
        .copied()
        .sum::<u16>();
    while start_row < selected_row && selected_offset > viewport_height {
        selected_offset = selected_offset.saturating_sub(row_heights[start_row]);
        start_row += 1;
    }
    let scroll_offset = row_heights
        .iter()
        .take(start_row)
        .map(|height| *height as usize)
        .sum::<usize>();
    let mut y = cards_area.y;
    for (row_index, row) in rows.iter().enumerate().skip(start_row) {
        let remaining = runs_bottom.saturating_sub(y);
        if remaining < 3 {
            break;
        }
        let row_height = row_heights[row_index].min(remaining).max(3);
        for (column, entry) in row.iter().enumerate() {
            let Some((index, process)) = entry else {
                continue;
            };
            let (card_x, card_width) =
                grid_column_span(cards_area.x, process_width, columns, column);
            let selected = app.selected_process == *index;
            let content_color = if selected { theme.text } else { theme.muted };
            let border_color = if selected { theme.text } else { theme.border };
            let marker = if selected { "▰" } else { "▱" };
            let filename = process_subtitle(&process.name)
                .unwrap_or_else(|| process_executable(&process.name).to_owned());
            let executable = process_executable(&process.name);
            let title = Line::from(vec![
                Span::styled(format!(" {marker} "), Style::default().fg(content_color)),
                Span::styled(
                    truncate(
                        &format!("{executable} {filename} "),
                        card_width.saturating_sub(7) as usize,
                    ),
                    Style::default()
                        .fg(content_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            let inner_width = card_width.saturating_sub(3);
            let detail_lines = process_detail_lines(app, process, inner_width as usize);
            let requested_height = process_card_height(app, process, card_width);
            let card_height = requested_height.min(row_height);
            let card = Rect {
                x: card_x,
                y,
                width: card_width,
                height: card_height,
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(ratatui::symbols::border::ROUNDED)
                .title(title)
                .border_style(Style::default().fg(border_color).add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }))
                .style(Style::default().bg(theme.bg));
            let inner = block.inner(card);
            frame.render_widget(block, card);
            let visible_details = detail_lines.len().min(inner.height as usize);
            for (row, line) in detail_lines.iter().take(visible_details).enumerate() {
                let line = format!(" {line}");
                frame.render_widget(
                    Paragraph::new(line).style(Style::default().fg(if selected {
                        theme.muted
                    } else {
                        theme.border
                    })),
                    Rect {
                        y: inner.y + row as u16,
                        height: 1,
                        ..inner
                    },
                );
            }
        }
        y += row_height;
    }
    if scrollable && runs_height > 0 {
        render_scrollbar(
            frame,
            cards_area.x + cards_area.width - 1,
            cards_area.y,
            runs_height,
            row_heights.iter().map(|height| *height as usize).sum(),
            scroll_offset,
            theme,
        );
    }
    if note_height > 0 && cards_area.height >= note_height {
        frame.render_widget(
            section_heading("Notes", cards_area.width, theme),
            Rect {
                y: notes_y,
                height: 1,
                ..cards_area
            },
        );
        for (index, line) in note_lines.iter().enumerate() {
            frame.render_widget(
                Paragraph::new(format!(" {line}")).style(
                    Style::default()
                        .fg(theme.text)
                        .bg(theme.bg)
                        .add_modifier(Modifier::ITALIC),
                ),
                Rect {
                    y: notes_y + 1 + index as u16,
                    height: 1,
                    ..cards_area
                },
            );
        }
    }
}

fn wrapped_notes(app: &AppState, width: usize) -> Vec<String> {
    app.notes
        .lines()
        .flat_map(|line| wrap_text(line, width))
        .collect()
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_owned()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_width = 0;
    for word in text.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);
        let separator_width = usize::from(!line.is_empty());
        if line_width + separator_width + word_width <= width {
            if separator_width > 0 {
                line.push(' ');
                line_width += 1;
            }
            line.push_str(word);
            line_width += word_width;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            line_width = 0;
        }
        for grapheme in UnicodeSegmentation::graphemes(word, true) {
            let grapheme_width = UnicodeWidthStr::width(grapheme);
            if line_width + grapheme_width > width && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                line_width = 0;
            }
            line.push_str(grapheme);
            line_width += grapheme_width;
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

fn render_tiny(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let header_height = area.height.min(1);
    render_header(
        frame,
        app,
        theme,
        Rect {
            height: header_height,
            ..area
        },
    );
    let mut y = area.y + header_height;
    let bottom = area.y + area.height;
    let note_lines = wrapped_notes(app, area.width.saturating_sub(2) as usize);
    let notes_height = if note_lines.is_empty() {
        0
    } else {
        note_lines.len() as u16 + 1
    };
    let notes_y = bottom.saturating_sub(notes_height);
    let content_bottom = if notes_height > 0 { notes_y } else { bottom };
    for gpu in &app.snapshot.gpus {
        if y >= content_bottom {
            break;
        }
        let util = gpu
            .utilization
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--%".into());
        let vram = match (gpu.memory_used, gpu.memory_total) {
            (Some(used), Some(total)) if total > 0 => {
                format!("{:.0}%", used as f64 / total as f64 * 100.0)
            }
            _ => "--%".into(),
        };
        let temp = gpu
            .temperature
            .map(|value| format!("{value}°C"))
            .unwrap_or_else(|| "--°C".into());
        let line = format!(" GPU {}  UTIL {util}  VRAM {vram}  TEMP {temp}", gpu.index);
        frame.render_widget(
            Paragraph::new(truncate(&line, area.width as usize))
                .style(Style::default().fg(theme.text).bg(theme.bg)),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
        y += 1;
    }
    if y < content_bottom {
        frame.render_widget(
            section_heading("Runs", area.width, theme),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
        y += 1;
    }
    if app.snapshot.processes.is_empty() && y < content_bottom {
        frame.render_widget(
            Paragraph::new(" No active GPU processes")
                .style(Style::default().fg(theme.muted).bg(theme.bg)),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
        y += 1;
    }
    let visible_processes = content_bottom.saturating_sub(y) as usize;
    let process_count = app.snapshot.processes.len();
    let scrollable = process_count > visible_processes;
    let first_process = app
        .selected_process
        .saturating_sub(visible_processes.saturating_sub(1));
    for (index, process) in app
        .snapshot
        .processes
        .iter()
        .enumerate()
        .skip(first_process)
    {
        if y >= content_bottom {
            break;
        }
        let selected = index == app.selected_process;
        let marker = if selected { "▰" } else { "▱" };
        let filename = process_subtitle(&process.name)
            .unwrap_or_else(|| process_executable(&process.name).to_owned());
        let metadata = process_detail_lines(app, process, area.width as usize).join(" · ");
        let line = format!(
            " {marker} {} {filename} · {metadata}",
            process_executable(&process.name)
        );
        frame.render_widget(
            Paragraph::new(truncate(
                &line,
                area.width.saturating_sub(u16::from(scrollable)) as usize,
            ))
            .style(
                Style::default()
                    .fg(if selected { theme.text } else { theme.muted })
                    .bg(theme.bg),
            ),
            Rect {
                y,
                height: 1,
                ..area
            },
        );
        y += 1;
    }
    if scrollable && visible_processes > 0 {
        render_scrollbar(
            frame,
            area.x + area.width - 1,
            content_bottom.saturating_sub(visible_processes as u16),
            visible_processes as u16,
            process_count,
            first_process,
            theme,
        );
    }
    if notes_height > 0 && notes_y >= area.y + header_height {
        frame.render_widget(
            section_heading("Notes", area.width, theme),
            Rect {
                y: notes_y,
                height: 1,
                ..area
            },
        );
        for (index, line) in note_lines.iter().enumerate() {
            frame.render_widget(
                Paragraph::new(format!(" {line}")).style(
                    Style::default()
                        .fg(theme.text)
                        .bg(theme.bg)
                        .add_modifier(Modifier::ITALIC),
                ),
                Rect {
                    y: notes_y + 1 + index as u16,
                    height: 1,
                    ..area
                },
            );
        }
    }
}

fn render_scrollbar(
    frame: &mut Frame,
    x: u16,
    y: u16,
    height: u16,
    total: usize,
    offset: usize,
    theme: Theme,
) {
    let viewport = height as usize;
    if viewport == 0 || total <= viewport {
        return;
    }
    let thumb_height = (viewport * viewport / total).clamp(1, viewport);
    let max_offset = total - viewport;
    let thumb_start = offset.min(max_offset) * (viewport - thumb_height) / max_offset;
    for row in 0..viewport {
        let thumb = (thumb_start..thumb_start + thumb_height).contains(&row);
        frame.render_widget(
            Paragraph::new("│").style(
                Style::default()
                    .fg(if thumb { theme.text } else { theme.border })
                    .bg(theme.bg),
            ),
            Rect {
                x,
                y: y + row as u16,
                width: 1,
                height: 1,
            },
        );
    }
}

fn gib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

fn truncate(value: &str, max: usize) -> String {
    if UnicodeWidthStr::width(value) <= max {
        return value.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut width = 0;
    for grapheme in UnicodeSegmentation::graphemes(value, true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        if width + grapheme_width > max.saturating_sub(1) {
            break;
        }
        result.push_str(grapheme);
        width += grapheme_width;
    }
    result.push('…');
    result
}
