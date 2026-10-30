//! The debug UI (owner, 2026-10-30): one semi-transparent ImGui window over the game,
//! resizable, its pages as tabs.
//! - "Display": the display page's figures (`debug::wear_page`), named and grouped
//!   (`debug::display_rows`), its bars as bars, filtered by name; a wide window lays the
//!   groups out in columns side by side.
//! - "Profiler": the original profile page's live items (`debug::draw_profile`, 0x43530E:
//!   the frame rate, the course's memory, the processor's features, `dword_6218B8`) and the
//!   standalone's own frame timings; the original's 40 timers are listed but RAL.EXE
//!   writes none of them (the developers' timing code was taken out).
//! - "Rivals": the rivals' table the display page ended with (`g_rivalNames`,
//!   `g_rivalScaled`, `g_rivalTimes`, as they are: unlike the original page this does not
//!   refill them).
//!
//! The race's freeze as a check box.

use crate::debug::wear_page::{SURFACES, value_at};
use dear_imgui_rs::{Condition, TableColumnFlags, TableFlags, TreeNodeFlags, Ui};

/// How a figure is shown.
#[derive(Clone, Copy)]
pub enum Shown {
    Whole,
    Float,
    Hex,
    Surface,
}

/// A figure of the display page: its name, the developers' label, how it is shown, its
/// place (IDA's address), whether the page drew a bar of it, and its place on the
/// original's 800 x 600 screen.
pub struct Row {
    name: &'static str,
    label: &'static str,
    shown: Shown,
    at: usize,
    bar: bool,
    place: (u32, u32),
}

impl Row {
    /// Its place (IDA's address).
    #[cfg(test)]
    pub fn at(&self) -> usize {
        self.at
    }

    pub const fn new(
        name: &'static str,
        label: &'static str,
        shown: Shown,
        at: usize,
        bar: bool,
        place: (u32, u32),
    ) -> Self {
        Self {
            name,
            label,
            shown,
            at,
            bar,
            place,
        }
    }
}

/// A group of figures.
pub struct Group {
    pub name: &'static str,
    pub rows: &'static [Row],
}

/// The filter's text (kept across frames).
static FILTER: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// How opaque the window's background is.
const BACKGROUND: f32 = 0.6;

/// A column of groups is at least this wide (pixels at the UI's scale 1).
const COLUMN_WIDTH: f32 = 460.0;

/// The UI's frame.
pub fn build(ui: &Ui) {
    ui.window("Debug")
        .size([760.0, 560.0], Condition::FirstUseEver)
        .position([40.0, 40.0], Condition::FirstUseEver)
        .bg_alpha(BACKGROUND)
        .build(|| {
            let mut frozen = crate::debug::overlay::frozen();
            if ui.checkbox("Race frozen (Ctrl+F12)", &mut frozen) {
                crate::debug::overlay::set_frozen(frozen);
            }
            ui.same_line();
            ui.text_disabled("Ctrl+F11 hides this");
            if let Some(_bar) = ui.tab_bar("pages") {
                if let Some(_tab) = ui.tab_item("Display") {
                    display(ui);
                }
                if let Some(_tab) = ui.tab_item("Profiler") {
                    profiler(ui);
                }
                if let Some(_tab) = ui.tab_item("Rivals") {
                    rivals(ui);
                }
            }
        });
}

/// A figure's text.
fn figure(value: Option<u32>, shown: Shown) -> String {
    match (value, shown) {
        (None, _) => "?".to_owned(),
        (Some(v), Shown::Whole) => format!("{}", v as i32),
        (Some(v), Shown::Float) => format!("{:.3}", f32::from_bits(v)),
        (Some(v), Shown::Hex) => format!("{v:08x}"),
        (Some(v), Shown::Surface) => {
            String::from_utf8_lossy(SURFACES[(v & 0xF) as usize]).into_owned()
        }
    }
}

/// Whether a row passes the filter (`wanted` in lower case).
fn wanted_row(row: &Row, wanted: &str) -> bool {
    wanted.is_empty()
        || row.name.to_lowercase().contains(wanted)
        || row.label.to_lowercase().contains(wanted)
}

/// The "Display" tab: the groups in as many columns as the width holds.
fn display(ui: &Ui) {
    let wanted = {
        let mut filter = FILTER.lock().unwrap_or_else(|e| e.into_inner());
        ui.input_text("Filter", &mut filter).build();
        filter.to_lowercase()
    };
    let groups: Vec<(&Group, usize)> = crate::debug::display_rows::GROUPS
        .iter()
        .map(|g| (g, g.rows.iter().filter(|r| wanted_row(r, &wanted)).count()))
        .filter(|&(_, n)| n > 0)
        .collect();
    let scale = ui.current_font_size() / 13.0;
    let columns = ((ui.content_region_avail_width() / (COLUMN_WIDTH * scale)) as usize)
        .clamp(1, 8)
        .min(groups.len().max(1));
    // Each group into the column with the fewest rows so far, in order.
    let mut placed: Vec<Vec<&Group>> = vec![Vec::new(); columns];
    let mut heights = vec![0usize; columns];
    for (group, rows) in groups {
        let shortest = (0..columns).min_by_key(|&c| heights[c]).unwrap_or(0);
        placed[shortest].push(group);
        heights[shortest] += rows + 2;
    }
    // The window scrolls; the columns share its width.
    let Some(_layout) = ui.begin_table_with_flags("layout", columns, TableFlags::NONE) else {
        return;
    };
    ui.table_next_row();
    for column in placed {
        ui.table_next_column();
        for group in column {
            if ui.collapsing_header(group.name, TreeNodeFlags::DEFAULT_OPEN) {
                group_table(ui, group, &wanted);
            }
        }
    }
}

/// A group's figures as a table.
fn group_table(ui: &Ui, group: &Group, wanted: &str) {
    let flags = TableFlags::BORDERS | TableFlags::ROW_BG | TableFlags::RESIZABLE;
    let Some(_table) = ui.begin_table_with_flags(group.name, 4, flags) else {
        return;
    };
    ui.table_setup_column("Name", TableColumnFlags::NONE, None);
    ui.table_setup_column("Value", TableColumnFlags::NONE, None);
    ui.table_setup_column("Bar", TableColumnFlags::NONE, None);
    ui.table_setup_column("Original", TableColumnFlags::NONE, None);
    ui.table_headers_row();
    for row in group.rows.iter().filter(|r| wanted_row(r, wanted)) {
        let value = value_at(row.at);
        ui.table_next_row();
        ui.table_next_column();
        ui.text(row.name);
        ui.table_next_column();
        ui.text(figure(value, row.shown));
        ui.table_next_column();
        if row.bar {
            let v = value.map_or(0.0, f32::from_bits);
            let _id = ui.push_id(row.at);
            ui.progress_bar(v / 100.0)
                .size([-1.0, 0.0])
                .overlay_text("")
                .build();
        }
        ui.table_next_column();
        ui.text_disabled(row.label);
        if ui.is_item_hovered() {
            ui.tooltip_text(format!(
                "IDA {:#08x}; at ({}, {}) on the original's page",
                row.at, row.place.0, row.place.1
            ));
        }
    }
}

/// The profiler's frame history (milliseconds a frame), kept across frames.
struct Timings {
    frames: std::collections::VecDeque<f32>,
    last: Option<(std::time::Instant, [u64; 5])>,
    /// The last frame's parts (ms): rasteriser, noting the triangles, showing; triangles;
    /// screens of pixels.
    parts: [f32; 5],
}

static TIMINGS: std::sync::Mutex<Timings> = std::sync::Mutex::new(Timings {
    frames: std::collections::VecDeque::new(),
    last: None,
    parts: [0.0; 5],
});

/// Frames in the history.
const HISTORY: usize = 240;

/// The original profile page's timers (`debug::draw_profile`'s labels), none written by
/// RAL.EXE.
const ORIGINAL_TIMERS: [&str; 40] = [
    "rdmap", "bitsy", "uplod", "scene", "overl", "addsk", "prepm", "tyrec", "goto2", "lockw",
    "debug", "xchng", "muddy", "systm", "drv3d", "setst", "clrps", "reflc", "readz", "sorts",
    "rainw", "revrb", "abmie", "camer", "mark", "cntrl", "chblb", "getys", "getyc", "cdata",
    "carvc", "getcp", "ctrl1", "ctrl2", "ctrl3", "ctrl4", "ctrl5", "ctrl6", "ctrl7", "ctrl8",
];

/// The "Profiler" tab.
fn profiler(ui: &Ui) {
    use crate::race::course_lists_store::COURSE_LISTS;
    use crate::race::switches_store::FRAME_SWITCHES;
    use crate::session::MACHINE;
    let mut t = TIMINGS.lock().unwrap_or_else(|e| e.into_inner());
    let now = std::time::Instant::now();
    let counts = crate::platform::frame_counters();
    if let Some((then, before)) = t.last {
        let ms = now.duration_since(then).as_secs_f32() * 1000.0;
        t.frames.push_back(ms);
        if t.frames.len() > HISTORY {
            t.frames.pop_front();
        }
        let d = |n: usize| counts[n].wrapping_sub(before[n]) as f32;
        let screen = crate::platform::screen_pixels().max(1) as f32;
        t.parts = [
            d(0) / 1000.0,
            d(1) / 1000.0,
            d(2) / 1000.0,
            d(3),
            d(4) / screen,
        ];
    }
    t.last = Some((now, counts));
    let fps = f32::from_bits(FRAME_SWITCHES.frame_rate.bits());
    ui.text(format!("Frame rate (the game's): {fps:.1}"));
    let history: Vec<f32> = t.frames.iter().copied().collect();
    let average = history.iter().sum::<f32>() / history.len().max(1) as f32;
    let worst = history.iter().copied().fold(0.0f32, f32::max);
    ui.plot_lines_config("##frames", &history)
        .overlay_text(format!(
            "{average:.1} ms a frame on average, {worst:.1} the worst"
        ))
        .scale_min(0.0)
        .scale_max((worst * 1.2).max(20.0))
        .graph_size([-1.0, 90.0 * ui.current_font_size() / 13.0])
        .build();
    if let Some(_table) =
        ui.begin_table_with_flags("profile", 2, TableFlags::BORDERS | TableFlags::ROW_BG)
    {
        let row = |name: &str, value: String| {
            ui.table_next_row();
            ui.table_next_column();
            ui.text(name);
            ui.table_next_column();
            ui.text(value);
        };
        row("Rasteriser (last frame)", format!("{:.2} ms", t.parts[0]));
        row("Noting the triangles", format!("{:.2} ms", t.parts[1]));
        row("Showing the frame", format!("{:.2} ms", t.parts[2]));
        row("Triangle fans", format!("{:.0}", t.parts[3]));
        row("Pixels drawn", format!("{:.2} screens", t.parts[4]));
        row(
            "Course memory (mem)",
            format!("{} KiB", COURSE_LISTS.course_bytes_total.get() >> 10),
        );
        let features = [
            ("MMX", MACHINE.has_mmx.get()),
            ("3DNow!", MACHINE.has_3dnow.get()),
            ("SSE", MACHINE.has_sse.get()),
        ];
        let list: Vec<&str> = features
            .iter()
            .filter(|(_, on)| *on != 0)
            .map(|(name, _)| *name)
            .collect();
        row(
            "Processor features (the game's)",
            if list.is_empty() {
                "none".to_owned()
            } else {
                list.join(", ")
            },
        );
        row(
            "dword_6218B8 (on / off)",
            figure(value_at(0x6218B8), Shown::Whole),
        );
    }
    if ui.collapsing_header("The original's timers (never written)", TreeNodeFlags::NONE) {
        ui.text_wrapped(
            "RAL.EXE's profile page printed these 40 timers (dword_5EA8C8 ..), but no code of \
             the shipped game writes them: the developers' timing calls were taken out.",
        );
        ui.text_disabled(ORIGINAL_TIMERS.join(" "));
    }
}

/// A rival's time (hundredths of a second) as minutes, seconds and hundredths.
fn time(t: u32) -> String {
    format!("{}:{:02}.{:02}", t / 6000, t / 100 % 60, t % 100)
}

/// The "Rivals" tab.
fn rivals(ui: &Ui) {
    use crate::physics::split_tables_store::SPLIT_TABLES;
    let flags = TableFlags::BORDERS | TableFlags::ROW_BG | TableFlags::SCROLL_Y;
    let Some(_table) = ui.begin_table_with_flags("rivals", 4, flags) else {
        return;
    };
    ui.table_setup_scroll_freeze(0, 1);
    ui.table_setup_column("#", TableColumnFlags::NONE, None);
    ui.table_setup_column("Name", TableColumnFlags::NONE, None);
    ui.table_setup_column("Time at the split", TableColumnFlags::NONE, None);
    ui.table_setup_column("Time", TableColumnFlags::NONE, None);
    ui.table_headers_row();
    for n in 0..rc2k_core::physics::splits::RIVALS {
        ui.table_next_row();
        ui.table_next_column();
        ui.text(format!("{n:02}"));
        ui.table_next_column();
        let mut name = Vec::new();
        let _ = crate::strings::append_string_by_id::append_string_by_id(
            SPLIT_TABLES.rival_names[n].get(),
            &mut |run| name.extend_from_slice(run),
        );
        ui.text(name.iter().map(|&b| char::from(b)).collect::<String>());
        ui.table_next_column();
        ui.text(time(SPLIT_TABLES.rival_scaled[n].get()));
        ui.table_next_column();
        ui.text(time(SPLIT_TABLES.rival_times[n].get()));
    }
}
