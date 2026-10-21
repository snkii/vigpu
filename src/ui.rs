// ViGPU — Seonuk Kim, Human Interface Laboratory, Seoul National University (2026-10-12)
// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{env, sync::OnceLock};
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

// Shared layout geometry. The border is followed by three blank cells; the
// scrollbar occupies the middle cell of that right gutter.
pub const PAGE_GUTTER_X: u16 = 2;
const PAGE_GUTTER_Y: u16 = 1;
const BORDER_PADDING_X: u16 = 3;
const BORDER_INSET_X: u16 = BORDER_PADDING_X + 1;
const CARD_PADDING_X: u16 = 1;
const MODAL_BACKDROP_GUTTER_X: u16 = 1;
const SECTION_GAP: u16 = 0;

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

static TRANSPARENT: OnceLock<bool> = OnceLock::new();

pub fn set_transparent(enabled: bool) {
    let _ = TRANSPARENT.set(enabled);
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
                [235, 235, 230],
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
        let transparent = TRANSPARENT.get().copied().unwrap_or(true);
        Self {
            bg: if transparent { Color::Reset } else { color(0) },
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
        x: screen.x + PAGE_GUTTER_X.min(screen.width),
        y: screen.y + PAGE_GUTTER_Y.min(screen.height),
        width: screen.width.saturating_sub(PAGE_GUTTER_X * 2),
        height: screen.height.saturating_sub(PAGE_GUTTER_Y * 2),
    };
    let gpu_scroll = gpu_scroll_for_selection(app, area);
    if area.width < 36 || area.height < (8 + app.snapshot.gpus.len() as u16 * 4).min(20) {
        render_tiny(frame, app, theme, area, gpu_scroll);
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
    let total_gpu_rows = app.snapshot.gpus.len().div_ceil(columns).max(1);
    let ideal_card = if body.height >= 24 { 7 } else { 6 };
    let section_gap = SECTION_GAP;
    // Preserve document order and let the viewport move through it. Do not
    // reserve space for Runs while GPU rows still occupy that part of the page.
    let minimum_card_height = if body.height >= 6 { 5 } else { 3 };
    let gpu_capacity = body.height.saturating_sub(1);
    let max_visible_gpu_rows =
        total_gpu_rows.min((gpu_capacity / minimum_card_height).max(1) as usize);
    let first_gpu_row = gpu_scroll.min(total_gpu_rows);
    let visible_gpu_rows = total_gpu_rows
        .saturating_sub(first_gpu_row)
        .min(max_visible_gpu_rows);
    let overview_height = u16::from(gpu_scroll == 0);
    let card_height = (gpu_capacity / max_visible_gpu_rows as u16)
        .min(ideal_card)
        .max(minimum_card_height);
    let cards_height = visible_gpu_rows as u16 * card_height;
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
    let process_y = body.y + overview_height + cards_height + section_gap;
    let process_area = Rect {
        y: process_y,
        height: body.y + body.height - process_y,
        ..body
    };
    let runs_scroll = if process_area.height >= 3 {
        render_processes(frame, app, theme, process_area)
    } else {
        ProcessScroll::default()
    };
    render_cards(
        frame,
        app,
        theme,
        cards_area,
        columns,
        card_height,
        gpu_scroll,
    );
    let runs_content = process_content_height(app, body.width).max(runs_scroll.content)
        + usize::from(app.snapshot.processes.is_empty());
    let total_content =
        1 + total_gpu_rows * card_height as usize + section_gap as usize + 1 + runs_content;
    let page_scrollable = total_content > body.height as usize;
    if page_scrollable {
        let content_offset = if gpu_scroll == 0 {
            0
        } else {
            1 + gpu_scroll.min(total_gpu_rows) * card_height as usize + runs_scroll.offset
        };
        render_scrollbar(
            frame,
            page_scrollbar_x(body),
            body.y,
            body.height,
            total_content,
            content_offset,
            theme,
            theme.bg,
        );
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

/// Keep keyboard navigation attached to process cards. The GPU section only
/// moves out of view when needed to reveal the selected process row, and is
/// restored automatically as selection moves back toward the top.
fn gpu_scroll_for_selection(app: &AppState, area: Rect) -> usize {
    if app.snapshot.processes.is_empty() {
        return gpu_scroll_for_gpu_selection(app, area);
    }
    if !app.process_navigation_active {
        return 0;
    }
    if area.width < 36 || area.height < (8 + app.snapshot.gpus.len() as u16 * 4).min(20) {
        let header = area.height.min(1);
        let y = area.y + header;
        let bottom = area.y + area.height;
        let content_bottom = bottom;
        let gpu_slots = content_bottom.saturating_sub(y) as usize;
        let total_gpus = app.snapshot.gpus.len();
        let rows = process_grid_rows(app, process_columns(area.width) as usize);
        let notes_space = if rows.last().is_some_and(|row| {
            row.iter()
                .flatten()
                .any(|(index, _)| *index == app.selected_process)
        }) {
            wrapped_notes(app, area.width.saturating_sub(2) as usize).len()
                + usize::from(!app.notes.trim().is_empty())
        } else {
            0
        };
        let needed_runs = 1 + app.selected_process.saturating_add(1) + notes_space;
        return (0..=total_gpus)
            .find(|first| {
                let visible = total_gpus.saturating_sub(*first).min(gpu_slots);
                content_bottom.saturating_sub(y + visible as u16) as usize >= needed_runs
            })
            .unwrap_or(total_gpus);
    }
    let header_height = 1.min(area.height);
    let body_height = area.height.saturating_sub(header_height);
    let columns = process_columns(area.width) as usize;
    let total_gpu_rows = app.snapshot.gpus.len().div_ceil(columns).max(1);
    let minimum_card_height = if body_height >= 6 { 5 } else { 3 };
    let gpu_capacity = body_height.saturating_sub(1);
    let max_visible_gpu_rows =
        total_gpu_rows.min((gpu_capacity / minimum_card_height).max(1) as usize);
    let ideal_card = if body_height >= 24 { 7 } else { 6 };
    let card_height = (gpu_capacity / max_visible_gpu_rows as u16)
        .min(ideal_card)
        .max(minimum_card_height);

    let rows = process_grid_rows(app, columns);
    let selected_row = rows
        .iter()
        .position(|row| {
            row.iter()
                .flatten()
                .any(|(index, _)| *index == app.selected_process)
        })
        .unwrap_or(0);
    let process_width = area.width.saturating_sub(1);
    let required_rows = rows
        .iter()
        .take(selected_row + 1)
        .enumerate()
        .map(|(row_index, row)| {
            row.iter()
                .enumerate()
                .filter_map(|(column, entry)| {
                    let (_, width) = grid_column_span(0, process_width, columns, column);
                    entry.map(|(_, process)| process_card_height(app, process, width))
                })
                .max()
                .unwrap_or(if row_index == selected_row { 3 } else { 0 })
        })
        .map(usize::from)
        .sum::<usize>();
    let notes_space = if selected_row + 1 == rows.len() && !app.notes.trim().is_empty() {
        wrapped_notes(app, area.width.saturating_sub(2) as usize).len() + 1
    } else {
        0
    };
    let needed_runs = 1 + required_rows + notes_space;

    (0..=total_gpu_rows)
        .find(|scroll| {
            let visible_gpu_rows = total_gpu_rows
                .saturating_sub(*scroll)
                .min(max_visible_gpu_rows);
            let overview_height = usize::from(*scroll == 0);
            let process_height = (body_height as usize).saturating_sub(
                overview_height + visible_gpu_rows * card_height as usize + SECTION_GAP as usize,
            );
            process_height >= needed_runs
        })
        .unwrap_or(total_gpu_rows)
}

fn gpu_scroll_for_gpu_selection(app: &AppState, area: Rect) -> usize {
    if !app.gpu_navigation_active || app.snapshot.gpus.is_empty() {
        return 0;
    }

    if area.width < 36 || area.height < (8 + app.snapshot.gpus.len() as u16 * 4).min(20) {
        let selected_row = app.selected_gpu;
        let total_rows = app.snapshot.gpus.len();
        let gpu_slots = area.height.saturating_sub(1) as usize;
        let notes = wrapped_notes(app, area.width.saturating_sub(2) as usize);
        let trailing = 2 + usize::from(!notes.is_empty()) * (1 + notes.len());
        let slots_before_trailing = gpu_slots.saturating_sub(trailing).max(1);
        let first_gpu = app
            .snapshot
            .gpus
            .len()
            .saturating_sub(slots_before_trailing);
        return if selected_row + 1 == total_rows {
            selected_row
                .saturating_sub(gpu_slots.saturating_sub(1))
                .max(first_gpu)
                .min(selected_row)
        } else {
            selected_row.saturating_sub(gpu_slots.saturating_sub(1))
        };
    }

    let columns = process_columns(area.width) as usize;
    let selected_row = app.selected_gpu / columns.max(1);
    let total_rows = app.snapshot.gpus.len().div_ceil(columns.max(1));
    let body_height = area.height.saturating_sub(1);
    let minimum_card_height = if body_height >= 6 { 5 } else { 3 };
    let gpu_capacity = body_height.saturating_sub(1);
    let max_visible_rows = total_rows.min((gpu_capacity / minimum_card_height).max(1) as usize);
    let card_height = (gpu_capacity / max_visible_rows as u16)
        .min(if body_height >= 24 { 7 } else { 6 })
        .max(minimum_card_height);
    let regular_first = selected_row.saturating_sub(max_visible_rows.saturating_sub(1));
    if selected_row + 1 < total_rows {
        return regular_first;
    }

    let notes = wrapped_notes(app, area.width.saturating_sub(2) as usize);
    let trailing_lines =
        SECTION_GAP as usize + 2 + usize::from(!notes.is_empty()) * (1 + notes.len());
    let rows_before_trailing = (body_height as usize)
        .saturating_sub(trailing_lines)
        .checked_div(card_height as usize)
        .unwrap_or(0)
        .max(1);
    regular_first
        .max(total_rows.saturating_sub(rows_before_trailing))
        .min(selected_row)
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
    } else if width >= 120 {
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
    app.snapshot
        .processes
        .iter()
        .enumerate()
        .collect::<Vec<_>>()
        .chunks(columns)
        .map(|entries| {
            let mut row = vec![None; columns];
            for (column, entry) in entries.iter().enumerate() {
                row[column] = Some(*entry);
            }
            row
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

pub fn gpu_neighbor(
    app: &AppState,
    selected: usize,
    columns: usize,
    delta_column: isize,
    delta_row: isize,
) -> Option<usize> {
    let columns = columns.max(1);
    let row = selected / columns;
    let column = selected % columns;
    if selected >= app.snapshot.gpus.len() {
        return None;
    }
    let next = if delta_column != 0 {
        row.checked_mul(columns)?
            .checked_add(column.checked_add_signed(delta_column)?)?
    } else {
        row.checked_add_signed(delta_row)?
            .checked_mul(columns)?
            .checked_add(column)?
    };
    (next < app.snapshot.gpus.len()).then_some(next)
}

fn process_card_height(app: &AppState, process: &crate::gpu::GpuProcess, width: u16) -> u16 {
    let inner_width = card_content_width(width) as usize;
    let detail_lines = process_detail_lines(app, process, inner_width).len();
    (detail_lines + 2) as u16
}

fn process_content_height(app: &AppState, width: u16) -> usize {
    let columns = process_columns(width) as usize;
    process_grid_rows(app, columns)
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .filter_map(|(column, entry)| {
                    let (_, card_width) = grid_column_span(0, width, columns, column);
                    entry.map(|(_, process)| process_card_height(app, process, card_width))
                })
                .max()
                .unwrap_or(0) as usize
        })
        .sum()
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
            format!("GPU {}", process.gpu),
            format!("PID {}", process.pid),
            format!("USER {}", process.user),
        ],
        vec![cpu, vram],
    ];
    for group in groups {
        let mut line = String::new();
        for field in group {
            let separator = if line.is_empty() { "" } else { " · " };
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
        _ => "<n> notes · <t> themes · <q> exit".to_owned(),
    };
    let footer = Rect {
        x: screen.x + PAGE_GUTTER_X.min(screen.width),
        y: screen.y + screen.height - 1,
        width: screen.width.saturating_sub(PAGE_GUTTER_X * 2),
        height: 1,
    };
    frame.render_widget(
        Paragraph::new("hi! (▰ᴗ▱)").style(Style::default().fg(theme.muted).bg(theme.bg)),
        Rect {
            width: footer.width.min(12),
            ..footer
        },
    );
    frame.render_widget(
        Paragraph::new(truncate(&hint, footer.width.saturating_sub(13) as usize))
            .alignment(Alignment::Right)
            .style(Style::default().fg(theme.muted).bg(theme.bg)),
        Rect {
            x: footer.x.saturating_add(13.min(footer.width)),
            width: footer.width.saturating_sub(13),
            ..footer
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

fn modal_inner(area: Rect) -> Rect {
    area.inner(Margin::new(BORDER_INSET_X, 1))
}

fn card_content(block_inner: Rect) -> Rect {
    block_inner.inner(Margin::new(CARD_PADDING_X, 0))
}

fn card_content_width(width: u16) -> u16 {
    width.saturating_sub(CARD_PADDING_X.saturating_add(1).saturating_mul(2))
}

fn scrollbar_gutter_x(area: Rect) -> u16 {
    area.x
        .saturating_add(area.width.saturating_sub(BORDER_PADDING_X))
}

fn page_scrollbar_x(area: Rect) -> u16 {
    area.x.saturating_add(area.width)
}

fn render_modal_backdrop(frame: &mut Frame, area: Rect, screen: Rect, theme: Theme) {
    let left = area.x.saturating_sub(MODAL_BACKDROP_GUTTER_X).max(screen.x);
    let right = area
        .x
        .saturating_add(area.width)
        .saturating_add(MODAL_BACKDROP_GUTTER_X)
        .min(screen.x.saturating_add(screen.width));
    let backdrop = Rect {
        x: left,
        y: area.y,
        width: right.saturating_sub(left),
        height: area.height,
    };
    frame.render_widget(Clear, backdrop);
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.panel)),
        backdrop,
    );
}

fn wordmark_line(theme: Theme, elapsed: std::time::Duration) -> Line<'static> {
    let phase = (elapsed.as_secs_f64() * 2.4) % 7.5 - 1.25;
    // Alternate the right eye between open and closed every 500 ms.
    let blink = (elapsed.as_millis() / 500) % 2 == 1;
    let mut spans = vec![
        Span::styled(
            "▰",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if blink { "▰" } else { "▱" },
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
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
        let area = Rect {
            y: screen.y + screen.height.saturating_sub(1),
            height: 1,
            ..screen
        };
        frame.render_widget(
            Paragraph::new(format!(" theme: {name} · <enter> save · <esc> close"))
                .style(Style::default().fg(theme.text).bg(theme.panel)),
            area,
        );
        return;
    }
    let width = screen.width.saturating_sub(PAGE_GUTTER_X * 2).min(44);
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
    render_modal_backdrop(frame, area, screen, theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme.text))
        .title(Span::styled(
            " ▰ Themes ",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel));
    let inner = modal_inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let visible = inner.height.saturating_sub(1) as usize;
    let start = selected
        .saturating_sub(visible.saturating_sub(1) / 2)
        .min(ThemeName::ALL.len().saturating_sub(visible));
    let list_area = inner;
    for (row, index) in (start..ThemeName::ALL.len()).take(visible).enumerate() {
        let name = ThemeName::ALL[index];
        let preview = Theme::from_name(name);
        let active = index == selected;
        let label_width = list_area
            .width
            .saturating_sub(if list_area.width >= 32 { 5 } else { 0 })
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
                    x: scrollbar_gutter_x(area),
                    y: inner.y + row as u16,
                    width: 1,
                    height: 1,
                },
            );
        }
    }
    if inner.height >= 2 {
        frame.render_widget(
            Paragraph::new("<enter> save · <esc> close")
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
    let width = screen.width.saturating_sub(PAGE_GUTTER_X * 2).min(72);
    let height = screen.height.saturating_sub(2).min(14);
    let area = Rect {
        x: screen.x + (screen.width - width) / 2,
        y: screen.y + (screen.height - height) / 2,
        width,
        height,
    };
    render_modal_backdrop(frame, area, screen, theme);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme.text))
        .title(Span::styled(
            " ▰ Notes ",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(theme.panel));
    let inner = modal_inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let content_width = inner.width as usize;
    let note_rows = wrapped_note_editor(&app.note_buffer, content_width);
    let visible_rows = inner.height.saturating_sub(1) as usize;
    let first_row = note_rows.len().saturating_sub(visible_rows);
    let mut rendered = Vec::with_capacity(visible_rows);
    for (text, show_newline) in note_rows.iter().skip(first_row) {
        let mut spans = vec![Span::styled(text.clone(), Style::default().fg(theme.text))];
        if *show_newline && content_width >= 2 {
            spans.push(Span::styled("/n", Style::default().fg(theme.muted)));
        }
        rendered.push(Line::from(spans));
    }
    frame.render_widget(
        Paragraph::new(rendered).style(Style::default().bg(theme.panel)),
        Rect {
            height: inner.height.saturating_sub(1),
            ..inner
        },
    );
    if visible_rows > 0 {
        if let Some((last_row, _)) = note_rows.last() {
            let cursor_y = inner
                .y
                .saturating_add(note_rows.len().saturating_sub(1).saturating_sub(first_row) as u16);
            let cursor_x = inner
                .x
                .saturating_add(UnicodeWidthStr::width(last_row.as_str()) as u16)
                .min(inner.x.saturating_add(inner.width.saturating_sub(1)));
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
    if note_rows.len() > visible_rows {
        render_scrollbar(
            frame,
            scrollbar_gutter_x(area),
            inner.y,
            visible_rows as u16,
            note_rows.len(),
            first_row,
            theme,
            theme.panel,
        );
    }
    if inner.height > 1 {
        frame.render_widget(
            Paragraph::new("<enter> new line (max 3) · <esc> save")
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
    let width = screen
        .width
        .saturating_sub(PAGE_GUTTER_X * 2)
        .min(100)
        .max(1);
    // Keep one of the three right-side padding cells blank before the scrollbar.
    let inner_width = modal_content_width(width) as usize;
    let make_content = |content_width: usize| {
        let mut content = process_detail_lines(app, process, content_width)
            .into_iter()
            .map(|line| Line::styled(format!(" {line}"), Style::default().fg(theme.muted)))
            .collect::<Vec<_>>();
        content.push(Line::raw(""));
        content.push(Line::styled(
            "[Command]",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ));
        content.extend(
            wrap_text(&process.name, content_width)
                .into_iter()
                .map(|line| Line::styled(line, Style::default().fg(theme.text))),
        );
        content
    };
    let content = make_content(inner_width);

    let desired_height = (content.len() as u16 + 2).min(screen.height.saturating_sub(2).max(1));
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(width) / 2,
        y: screen.y + screen.height.saturating_sub(desired_height) / 2,
        width,
        height: desired_height,
    };
    render_modal_backdrop(frame, area, screen, theme);
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
    let inner = modal_inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }
    let visible_rows = inner.height as usize;
    let has_scrollbar = content.len() > visible_rows;
    let content_rows = content.len();
    let max_scroll = content.len().saturating_sub(visible_rows);
    frame.render_widget(
        Paragraph::new(Text::from(content))
            .scroll((scroll.min(max_scroll).min(u16::MAX as usize) as u16, 0))
            .style(Style::default().bg(theme.panel)),
        Rect {
            width: inner.width,
            height: visible_rows as u16,
            ..inner
        },
    );
    if has_scrollbar {
        render_scrollbar(
            frame,
            scrollbar_gutter_x(area),
            inner.y,
            visible_rows as u16,
            content_rows,
            scroll.min(max_scroll),
            theme,
            theme.panel,
        );
    }
}

fn modal_content_width(width: u16) -> u16 {
    width.saturating_sub(BORDER_INSET_X.saturating_mul(2))
}

fn wrapped_note_editor(text: &str, width: usize) -> Vec<(String, bool)> {
    if width == 0 {
        return vec![(String::new(), false)];
    }
    let mut output = Vec::new();
    let logical_lines: Vec<&str> = text.split('\n').take(3).collect();
    for (line_index, logical) in logical_lines.iter().enumerate() {
        let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(*logical, true).collect();
        let has_newline = line_index + 1 < logical_lines.len();
        let suffix_width = if has_newline { 2 } else { 1 };
        let mut start = 0;
        let mut logical_rows = Vec::new();
        while start < graphemes.len() {
            let remaining_width = graphemes[start..]
                .iter()
                .map(|grapheme| UnicodeWidthStr::width(*grapheme))
                .sum::<usize>();
            let final_row = remaining_width <= width.saturating_sub(suffix_width);
            let limit = if final_row {
                width.saturating_sub(suffix_width)
            } else {
                width
            };
            let mut row = String::new();
            let mut row_width = 0;
            while start < graphemes.len() {
                let grapheme_width = UnicodeWidthStr::width(graphemes[start]);
                if row_width + grapheme_width > limit && !row.is_empty() {
                    break;
                }
                row.push_str(graphemes[start]);
                row_width += grapheme_width;
                start += 1;
                if row_width >= limit && limit > 0 {
                    break;
                }
            }
            logical_rows.push(row);
        }
        if logical_rows.is_empty() {
            logical_rows.push(String::new());
        }
        let last_width =
            UnicodeWidthStr::width(logical_rows.last().map(String::as_str).unwrap_or(""));
        if last_width + suffix_width > width {
            logical_rows.push(String::new());
        }
        let last_row = logical_rows.len().saturating_sub(1);
        output.extend(
            logical_rows
                .into_iter()
                .enumerate()
                .map(|(row_index, row)| (row, has_newline && row_index == last_row)),
        );
    }
    if !logical_lines.is_empty()
        && output
            .last()
            .is_some_and(|(row, _)| UnicodeWidthStr::width(row.as_str()) >= width)
    {
        output.push((String::new(), false));
    }
    if output.is_empty() {
        output.push((String::new(), false));
    }
    output
}

fn render_cards(
    frame: &mut Frame,
    app: &AppState,
    theme: Theme,
    area: Rect,
    columns: usize,
    card_height: u16,
    scroll: usize,
) {
    if app.snapshot.gpus.is_empty() {
        frame.render_widget(
            Paragraph::new("  No NVIDIA GPUs found.").style(Style::default().fg(theme.yellow)),
            area,
        );
        return;
    }
    let total_rows = app.snapshot.gpus.len().div_ceil(columns);
    let rows_visible = area.height.div_ceil(card_height) as usize;
    let first_row = scroll.min(total_rows);
    let last_row = first_row + rows_visible;
    for (i, (gpu, visual)) in app.snapshot.gpus.iter().zip(&app.visuals).enumerate() {
        let row = i / columns;
        if row < first_row || row >= last_row {
            continue;
        }
        let col = i % columns;
        let (x, width) = grid_column_span(area.x, area.width, columns, col);
        let y = area.y + (row - first_row) as u16 * card_height;
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
            .any(|process| process.gpu == gpu.index)
            || (app.snapshot.processes.is_empty()
                && app.gpu_navigation_active
                && app
                    .snapshot
                    .gpus
                    .get(app.selected_gpu)
                    .is_some_and(|selected| selected.index == gpu.index));
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
        &format!("GPU {}: {}", gpu.index, gpu.name),
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
    let border_inner = block.inner(area);
    let data = card_content(border_inner);
    frame.render_widget(block, area);
    if data.height == 0 {
        return;
    }

    let wide = data.width >= 42;
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
    let power_extra = match (gpu.power_w, gpu.power_limit_w) {
        (Some(used), Some(limit)) => Some(format!(" PWR {used:.0}/{limit:.0}W")),
        (Some(used), _) => Some(format!(" PWR {used:.0}W")),
        _ => Some(" PWR --W".into()),
    };
    let mem_extra = if wide {
        match (gpu.memory_used, gpu.memory_total) {
            (Some(used), Some(total)) => Some(format!(" {:.1}/{:.1}G", gib(used), gib(total))),
            _ => None,
        }
    } else {
        None
    };
    let bar_width = if wide {
        let longest_extra = power_extra
            .as_deref()
            .map(str::len)
            .unwrap_or(0)
            .max(mem_extra.as_deref().map(str::len).unwrap_or(0));
        (data.width as usize)
            .saturating_sub(11 + longest_extra)
            .max(6)
    } else {
        (data.width as usize).saturating_sub(15).max(3)
    };
    let util_extra = wide.then_some(power_extra).flatten();
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
    frame.render_widget(Paragraph::new(util_line), Rect { height: 1, ..data });
    if data.height >= 2 {
        frame.render_widget(
            Paragraph::new(mem_line),
            Rect {
                y: data.y + 1,
                height: 1,
                ..data
            },
        );
    }

    if data.height >= 3 {
        let graph_height = data.height.saturating_sub(3).min(2);
        let graph_area = Rect {
            y: data.y + 2,
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
        let details = format!("TEMP {temperature} · FAN {fan}");
        frame.render_widget(
            Paragraph::new(truncate(&details, data.width as usize))
                .style(Style::default().fg(theme.muted)),
            Rect {
                y: data.y + 2 + graph_height,
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
        Span::styled(format!("{label} "), Style::default().fg(theme.muted)),
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

#[derive(Clone, Copy, Default)]
struct ProcessScroll {
    content: usize,
    offset: usize,
}

fn render_processes(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect) -> ProcessScroll {
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
    let runs_bottom = cards_area.y.saturating_add(cards_area.height);
    if app.snapshot.processes.is_empty() && cards_area.height > 0 {
        #[cfg(target_os = "macos")]
        let empty_label = " Per-process GPU readings are unavailable for Apple GPUs";
        #[cfg(not(target_os = "macos"))]
        let empty_label = " No active GPU processes";
        frame.render_widget(
            Paragraph::new(empty_label).style(Style::default().fg(theme.muted).bg(theme.bg)),
            Rect {
                height: 1,
                ..Rect {
                    height: cards_area.height,
                    ..cards_area
                }
            },
        );
    }
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
    let note_section_height = if note_lines.is_empty() {
        0
    } else {
        note_lines.len() as u16 + 1
    };
    let mut selected_offset = row_heights
        .iter()
        .take(selected_row + 1)
        .copied()
        .sum::<u16>();
    if selected_row + 1 == rows.len() {
        selected_offset = selected_offset.saturating_add(note_section_height);
    }
    while start_row < selected_row && selected_offset > viewport_height {
        selected_offset = selected_offset.saturating_sub(row_heights[start_row]);
        start_row += 1;
    }
    let scroll_offset = row_heights
        .iter()
        .take(start_row)
        .map(|height| *height as usize)
        .sum::<usize>();
    let mut y = cards_area
        .y
        .saturating_add(u16::from(app.snapshot.processes.is_empty()));
    let mut reached_final_run = start_row >= rows.len();
    for (row_index, row) in rows.iter().enumerate().skip(start_row) {
        let remaining = runs_bottom.saturating_sub(y);
        if remaining < 3 {
            reached_final_run = false;
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
            let inner_width = card_content_width(card_width);
            let detail_lines = process_detail_lines(app, process, inner_width as usize);
            let requested_height = process_card_height(app, process, card_width);
            let clipped = requested_height > remaining
                || (requested_height == remaining && row_index + 1 < rows.len());
            let card_height = requested_height.min(row_height);
            let card = Rect {
                x: card_x,
                y,
                width: card_width,
                height: card_height,
            };
            let block = Block::default()
                .borders(if clipped {
                    Borders::LEFT | Borders::RIGHT | Borders::TOP
                } else {
                    Borders::ALL
                })
                .border_set(ratatui::symbols::border::ROUNDED)
                .title(title)
                .border_style(Style::default().fg(border_color).add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }))
                .style(Style::default().bg(theme.bg));
            let inner = card_content(block.inner(card));
            frame.render_widget(block, card);
            let visible_details = detail_lines.len().min(inner.height as usize);
            for (row, line) in detail_lines.iter().take(visible_details).enumerate() {
                frame.render_widget(
                    Paragraph::new(line.clone()).style(Style::default().fg(if selected {
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
        reached_final_run = row_index + 1 == rows.len();
        y += row_height;
    }
    // Only render Notes after the final Run row is on screen. Otherwise, the
    // remaining Run rows still occupy this document flow, even when clipped.
    let notes_y = y;
    if reached_final_run && !note_lines.is_empty() && notes_y < runs_bottom {
        frame.render_widget(
            section_heading("Notes", cards_area.width, theme),
            Rect {
                y: notes_y,
                height: 1,
                ..cards_area
            },
        );
        for (index, line) in note_lines.iter().enumerate() {
            let line_y = notes_y + 1 + index as u16;
            if line_y >= runs_bottom {
                break;
            }
            frame.render_widget(
                Paragraph::new(format!(" {line}")).style(
                    Style::default()
                        .fg(theme.text)
                        .bg(theme.bg)
                        .add_modifier(Modifier::ITALIC),
                ),
                Rect {
                    y: line_y,
                    height: 1,
                    ..cards_area
                },
            );
        }
    }
    ProcessScroll {
        content: row_heights
            .iter()
            .map(|height| *height as usize)
            .sum::<usize>()
            + usize::from(!note_lines.is_empty())
            + note_lines.len(),
        offset: scroll_offset,
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

fn render_tiny(frame: &mut Frame, app: &AppState, theme: Theme, area: Rect, gpu_scroll: usize) {
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
    let content_bottom = bottom;
    let total_gpus = app.snapshot.gpus.len();
    let gpu_slots = content_bottom.saturating_sub(y) as usize;
    let gpu_first = gpu_scroll.min(total_gpus);
    let visible_gpus = total_gpus.saturating_sub(gpu_first).min(gpu_slots);
    let gpu_scrollable = gpu_first < total_gpus;
    let gpu_start_y = y;
    for (index, gpu) in app
        .snapshot
        .gpus
        .iter()
        .enumerate()
        .skip(gpu_first)
        .take(visible_gpus)
    {
        let selected = app.gpu_navigation_active && index == app.selected_gpu;
        let marker = if selected { "▰" } else { "▱" };
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
        let line = format!(
            " {marker} GPU {}  UTIL {util}  VRAM {vram}  TEMP {temp}",
            gpu.index
        );
        let gpu_area = Rect {
            y,
            height: 1,
            ..area
        };
        frame.render_widget(
            Paragraph::new(truncate(
                &line,
                area.width.saturating_sub(u16::from(gpu_scrollable)) as usize,
            ))
            .style(
                Style::default()
                    .fg(if selected { theme.text } else { theme.muted })
                    .bg(theme.bg),
            ),
            gpu_area,
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
    let process_count = app.snapshot.processes.len();
    let note_section_height = if note_lines.is_empty() {
        0
    } else {
        note_lines.len() as u16 + 1
    };
    let selected_is_last = app.selected_process + 1 >= process_count && process_count > 0;
    let visible_processes = content_bottom
        .saturating_sub(y)
        .saturating_sub(if selected_is_last {
            note_section_height
        } else {
            0
        }) as usize;
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
    let notes_y = y;
    let notes_visible = !note_lines.is_empty() && notes_y < bottom;
    if notes_visible {
        frame.render_widget(
            section_heading("Notes", area.width, theme),
            Rect {
                y: notes_y,
                height: 1,
                ..area
            },
        );
        for (index, line) in note_lines.iter().enumerate() {
            let line_y = notes_y + 1 + index as u16;
            if line_y >= bottom {
                break;
            }
            frame.render_widget(
                Paragraph::new(format!(" {line}")).style(
                    Style::default()
                        .fg(theme.text)
                        .bg(theme.bg)
                        .add_modifier(Modifier::ITALIC),
                ),
                Rect {
                    y: line_y,
                    height: 1,
                    ..area
                },
            );
        }
    }
    if (gpu_scrollable || scrollable || notes_visible) && gpu_slots + visible_processes > 0 {
        render_scrollbar(
            frame,
            page_scrollbar_x(area),
            gpu_start_y,
            (gpu_slots + 1 + visible_processes)
                .min(content_bottom.saturating_sub(gpu_start_y) as usize) as u16,
            total_gpus + 1 + process_count + usize::from(!note_lines.is_empty()) + note_lines.len(),
            gpu_first + 1 + first_process,
            theme,
            theme.bg,
        );
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
    background: Color,
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
                    .bg(background),
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
