//! The debug UI (owner, 2026-10-30): one semi-transparent ImGui window over the game,
//! resizable, its pages as tabs.
//! - "Display": the display page's figures (`debug::wear_page`), named and grouped
//!   (`debug::display_rows`), its bars as bars, filtered by name; a wide window lays the
//!   groups out in columns side by side.
//! - "Profiler": the original profile page's live items (`debug::draw_profile`, 0x43530E:
//!   the frame rate, the course's memory, the processor's features, `dword_6218B8`) and our
//!   own timings (`debug::timing`, `platform::frame_counters`): RAL.EXE writes none of its
//!   page's 40 timers (the developers' timing code was taken out).
//! - "Watch list": the variables of the developers' watch pages (`debug::watch_rows`), laid
//!   out as "Display"; "Dumps": the league tables those pages dumped in hex.
//! - "Rivals": the rivals' table the display page ended with (`g_rivalNames`,
//!   `g_rivalScaled`, `g_rivalTimes`, as they are: unlike the original page this does not
//!   refill them).
//!
//! The race's freeze as a check box.

use crate::debug::timing::STAGES;
use crate::debug::wear_page::{SURFACES, value_at};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::owner_physics::Physics;
use dear_imgui_rs::{Condition, TableColumnFlags, TableColumnWidth, TableFlags, TreeNodeFlags, Ui};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

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

/// The filters' texts (kept across frames): one a tab of groups.
static FILTER: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static WATCH_FILTER: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// How opaque the window's background is.
const BACKGROUND: f32 = 0.6;

/// A column of groups is at least this wide (pixels at the UI's scale 1).
const COLUMN_WIDTH: f32 = 460.0;

/// Whether the probe points are drawn.
static PROBES: AtomicBool = AtomicBool::new(false);

/// The view matrix, the scale, the centre.
type NotedView = ([f32; 16], f32, [f32; 2]);

/// The first view as its scene was drawn: the view matrix, the scale, the centre. Later
/// drawing (the mirror) changes them, so they are noted here (`note_view`).
static VIEW: std::sync::Mutex<Option<NotedView>> = std::sync::Mutex::new(None);

/// The first view's scene has been drawn.
pub fn note_view(frame: &Frame) {
    use crate::scene::draw_store::DRAW;
    let view = (
        std::array::from_fn(|n| frame.matrices.view_matrix[n].get()),
        DRAW.view_scale.get(),
        [
            f32::from_bits(DRAW.view_centre[0].bits()),
            f32::from_bits(DRAW.view_centre[1].bits()),
        ],
    );
    *VIEW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(view);
}

/// In the spirit of `sub_4295AD` (0x4295AD), the developers' mode 0xB, which drew the probe
/// lists as one pixel each: the physics' probe points as last worked out
/// (`physics::probe_ground`: 0x6D1E14 .. in the world, whole numbers), through the view
/// noted, each with a line down to the ground found under it. Green the first four (the
/// wheels, a reading), red a point at or under the ground, white the rest; yellow the
/// ground. Above the window, and behind the eye skipped.
fn draw_probes(physics: &Physics, ui: &Ui) {
    use crate::physics::car_store::CAR;
    let wk = &physics.contact;
    let Some((m, scale, centre)) = *VIEW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    else {
        return;
    };
    let [w, h] = ui.io().display_size();
    let (width, height) = crate::platform::screen_size();
    let (sx, sy) = (w / width.max(1) as f32, h / height.max(1) as f32);
    let place = |[x, y, z]: [i32; 3]| {
        let [x, y, z] = [f64::from(x), f64::from(y), f64::from(z)];
        let turned = |n: usize| {
            f64::from(m[n]) * x
                + f64::from(m[n + 4]) * y
                + f64::from(m[n + 8]) * z
                + f64::from(m[n + 12])
        };
        let (tx, ty, tz) = (turned(0), turned(1), turned(2));
        (tz >= 4096.0).then(|| {
            let s = f64::from(scale) / tz;
            [
                (f64::from(centre[0]) + tx * s) as f32 * sx,
                (f64::from(centre[1]) - ty * s) as f32 * sy,
            ]
        })
    };
    let list = ui.get_foreground_draw_list();
    let size = ui.current_font_size() / 4.0;
    for n in 0..CAR.probe_count.get().min(0x40) as usize {
        let (x, y, z) = (
            wk.probe_x[n].get() as i32,
            wk.probe_y[n].get() as i32,
            wk.probe_z[n].get() as i32,
        );
        let Some(p) = place([x, y, z]) else {
            continue;
        };
        let found = wk.probe_found[n].get() != 0;
        if found && let Some(g) = place([x, wk.probe_ground_height[n].get() as i32, z]) {
            list.add_line(p, g, [1.0, 1.0, 0.0, 1.0]).build();
        }
        let colour = if found && (wk.probe_height[n].get() as i32) <= 0 {
            [1.0, 0.2, 0.2, 1.0]
        } else if n < 4 {
            [0.2, 1.0, 0.2, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };
        list.add_circle(p, size, colour).filled(true).build();
        list.add_text(
            [p[0] + size * 1.5, p[1] - size * 2.0],
            colour,
            format!("{n}"),
        );
    }
}

/// The window's place when last drawn, and the screen it was on.
#[derive(Clone, Copy)]
struct Layout {
    display: [f32; 2],
    position: [f32; 2],
    size: [f32; 2],
}

static LAYOUT: std::sync::Mutex<Option<Layout>> = std::sync::Mutex::new(None);

/// A tab's contents, scrolled under the tabs (which stay; owner, 2026-10-30).
fn body(ui: &Ui, f: impl FnOnce()) {
    ui.child_window("body").build(ui, f);
}

/// The UI's frame.
pub fn build(race: &Race, ui: &Ui) {
    if PROBES.load(Relaxed) {
        draw_probes(&race.physics, ui);
    }
    crate::debug::kstar::over_picture(&race.frame, ui);
    // The window keeps its share of the screen when the mode changes (the menus' 640 x 480,
    // the race's own; owner, 2026-10-30).
    let screen = ui.io().display_size();
    let mut layout = LAYOUT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (position, size, condition) = match *layout {
        Some(l) if l.display != screen => {
            let (x, y) = (screen[0] / l.display[0], screen[1] / l.display[1]);
            (
                [l.position[0] * x, l.position[1] * y],
                [l.size[0] * x, l.size[1] * y],
                Condition::Always,
            )
        }
        _ => ([40.0, 40.0], [760.0, 560.0], Condition::FirstUseEver),
    };
    ui.window("Debug")
        .size(size, condition)
        .position(position, condition)
        .bg_alpha(BACKGROUND)
        .build(|| {
            *layout = Some(Layout {
                display: screen,
                position: ui.window_pos(),
                size: ui.window_size(),
            });
            let mut frozen = crate::debug::overlay::frozen();
            if ui.checkbox("Race frozen (Ctrl+F12)", &mut frozen) {
                crate::debug::overlay::set_frozen(frozen);
            }
            ui.same_line();
            let mut probes = PROBES.load(Relaxed);
            if ui.checkbox("Probe points", &mut probes) {
                PROBES.store(probes, Relaxed);
            }
            if ui.is_item_hovered() {
                ui.tooltip_text(
                    "Where the physics asks for the ground (the last car simulated): green \
                     the first four (the wheels, a reading), red at or under the ground, \
                     white the rest; the yellow line goes down to the ground found",
                );
            }
            ui.same_line();
            ui.text_disabled("Ctrl+F11 hides this");
            if let Some(_bar) = ui.tab_bar("pages") {
                if let Some(_tab) = ui.tab_item("Display") {
                    body(ui, || {
                        display(ui, crate::debug::display_rows::GROUPS, &FILTER);
                    });
                }
                if let Some(_tab) = ui.tab_item("Watch list") {
                    body(ui, || {
                        display(ui, crate::debug::watch_rows::GROUPS, &WATCH_FILTER);
                    });
                }
                if let Some(_tab) = ui.tab_item("Dumps") {
                    body(ui, || dumps(ui));
                }
                if let Some(_tab) = ui.tab_item("Profiler") {
                    body(ui, || profiler(&race.course, ui));
                }
                if let Some(_tab) = ui.tab_item("Kstar modes") {
                    body(ui, || crate::debug::kstar::tab(&race.frame, ui));
                }
                if let Some(_tab) = ui.tab_item("Switches") {
                    body(ui, || crate::debug::switches::tab(ui));
                }
                if let Some(_tab) = ui.tab_item("Rivals") {
                    body(ui, || rivals(&race.physics, ui));
                }
                if let Some(_tab) = ui.tab_item("Effects") {
                    body(ui, || crate::debug::effects_page::tab(ui));
                }
                if let Some(_tab) = ui.tab_item("Dyno") {
                    body(ui, || crate::debug::dyno::tab(&race.course, ui));
                }
                if let Some(_tab) = ui.tab_item("Players") {
                    body(ui, || crate::debug::players::tab(ui));
                }
                if let Some(_tab) = ui.tab_item("Screens") {
                    body(ui, || screens(ui));
                }
                if let Some(_tab) = ui.tab_item("Developers") {
                    body(ui, || crate::debug::dev_front_end::tab(ui));
                }
            }
        });
}

/// The screens no route reaches.
fn screens(ui: &Ui) {
    ui.text("Section best times (shown at the next menu's frame):");
    for section in 0..6 {
        let label = format!("Section {}", section + 1);
        if crate::debug::screens::section_has_times(section) {
            if ui.button(&label) {
                crate::debug::screens::ask_section_times(section as u32);
            }
        } else {
            ui.text_disabled(format!("{label}: no table"));
        }
        ui.same_line();
    }
    ui.new_line();
    if ui.button("Car specifications (0x4AA1A3)") {
        crate::debug::screens::ask_cspecs();
    }
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

/// A tab of groups, in as many columns as the width holds.
fn display(ui: &Ui, all: &'static [Group], filter: &std::sync::Mutex<String>) {
    let wanted = {
        let mut filter = filter
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ui.input_text("Filter", &mut filter).build();
        filter.to_lowercase()
    };
    let groups: Vec<(&Group, usize)> = all
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
    // The name wide, the figure narrow (owner, 2026-10-30).
    let column = |name: &str, weight: f32| {
        ui.table_setup_column(
            name,
            TableColumnFlags::NONE,
            Some(TableColumnWidth::Stretch(weight)),
        );
    };
    column("Name", 3.0);
    column("Value", 1.2);
    column("Bar", 1.5);
    column("Original", 1.2);
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
            let hex = value.map_or("?".to_owned(), |v| format!("{v:08x}"));
            ui.tooltip_text(if row.place == (0, 0) {
                format!("IDA {:#08x}; {hex}", row.at)
            } else {
                format!(
                    "IDA {:#08x}; {hex}; at ({}, {}) on the original's page",
                    row.at, row.place.0, row.place.1
                )
            });
        }
    }
}

/// How much of a new reading goes into the shown one (they jump from frame to frame).
const SMOOTHING: f32 = 0.1;

/// The graph's parts of a frame, with their colours. The rasteriser and noting the
/// triangles are inside the scene's time (the scene's end flushes its fans).
const PARTS: [(&str, [f32; 4]); 8] = [
    ("Input", [0.6, 0.6, 1.0, 1.0]),
    ("Physics", [0.3, 1.0, 0.3, 1.0]),
    ("Scene", [1.0, 0.6, 0.2, 1.0]),
    ("Sound commands", [1.0, 0.4, 1.0, 1.0]),
    ("Rasteriser", [1.0, 0.25, 0.25, 1.0]),
    ("Noting", [1.0, 1.0, 0.3, 1.0]),
    ("Showing", [0.3, 1.0, 1.0, 1.0]),
    ("The rest", [0.7, 0.7, 0.7, 1.0]),
];

/// `value` at the end of `history`, the last `HISTORY` kept.
fn push(history: &mut VecDeque<f32>, value: f32) {
    history.push_back(value);
    if history.len() > HISTORY {
        history.pop_front();
    }
}

/// A reading of the running counters.
#[derive(Clone, Copy)]
struct Counts {
    at: std::time::Instant,
    frame: [u64; 5],
    stages: ([u64; STAGES], [u64; STAGES]),
    ticks: u32,
}

/// The profiler's readings, kept across frames.
struct Timings {
    /// Milliseconds a frame, the last `HISTORY`.
    frames: VecDeque<f32>,
    /// Each part's, the same frames.
    parts: [VecDeque<f32>; PARTS.len()],
    /// The screens of pixels drawn and the triangle fans, the same frames.
    pixels: VecDeque<f32>,
    fans: VecDeque<f32>,
    /// Whether the graph shows each part (off: the frame's total alone).
    each: bool,
    /// Smoothed: the frame less what is timed.
    rest: f32,
    last: Option<Counts>,
    /// Smoothed: `frame_counters`' five per frame (the three times in ms), the stages' ms a
    /// run, the physics ticks a frame.
    frame: [f32; 5],
    stages: [f32; STAGES],
    ticks: f32,
}

static TIMINGS: std::sync::Mutex<Timings> = std::sync::Mutex::new(Timings {
    frames: VecDeque::new(),
    parts: [const { VecDeque::new() }; PARTS.len()],
    pixels: VecDeque::new(),
    fans: VecDeque::new(),
    each: false,
    rest: 0.0,
    last: None,
    frame: [0.0; 5],
    stages: [0.0; STAGES],
    ticks: 0.0,
});

/// Frames in the history.
const HISTORY: usize = 240;

/// The readings taken, once a UI frame while the tab is shown (a gap, from another tab, is
/// not a frame's time).
fn take_readings(t: &mut Timings) {
    let now = Counts {
        at: std::time::Instant::now(),
        frame: crate::platform::frame_counters(),
        stages: crate::debug::timing::counts(),
        ticks: crate::debug::timing::ticks(),
    };
    if let Some(then) = t
        .last
        .filter(|l| now.at.duration_since(l.at).as_millis() < 500)
    {
        let ms = now.at.duration_since(then.at).as_secs_f32() * 1000.0;
        push(&mut t.frames, ms);
        let ease = |shown: &mut f32, new: f32| *shown += (new - *shown) * SMOOTHING;
        let screen = crate::platform::screen_pixels().max(1) as f32;
        for (n, shown) in t.frame.iter_mut().enumerate() {
            let d = now.frame[n].wrapping_sub(then.frame[n]) as f32;
            ease(
                shown,
                match n {
                    0..=2 => d / 1000.0,
                    4 => d / screen,
                    _ => d,
                },
            );
        }
        for (n, shown) in t.stages.iter_mut().enumerate() {
            let runs = now.stages.1[n].wrapping_sub(then.stages.1[n]);
            if runs != 0 {
                let nanos = now.stages.0[n].wrapping_sub(then.stages.0[n]);
                ease(shown, nanos as f32 / runs as f32 / 1e6);
            }
        }
        ease(&mut t.ticks, now.ticks.wrapping_sub(then.ticks) as f32);
        let stage = |n: usize| now.stages.0[n].wrapping_sub(then.stages.0[n]) as f32 / 1e6;
        let counted = |n: usize| now.frame[n].wrapping_sub(then.frame[n]) as f32;
        let timed = [
            stage(0),
            stage(1),
            stage(2),
            stage(3),
            counted(0) / 1000.0,
            counted(1) / 1000.0,
            counted(2) / 1000.0,
        ];
        // The rasteriser (4) and noting (5) are inside the scene's time.
        let inside: f32 = timed
            .iter()
            .enumerate()
            .filter(|&(n, _)| n != 4 && n != 5)
            .map(|(_, v)| v)
            .sum();
        let rest = (ms - inside).max(0.0);
        ease(&mut t.rest, rest);
        for (n, value) in timed.iter().chain([&rest]).enumerate() {
            push(&mut t.parts[n], *value);
        }
        push(&mut t.pixels, counted(4) / screen);
        push(&mut t.fans, counted(3));
    }
    t.last = Some(now);
}

/// The "Profiler" tab.
fn profiler(course: &Course, ui: &Ui) {
    use crate::audio::store::AUDIO;
    use crate::debug::timing::Stage;
    use crate::race::switches_store::FRAME_SWITCHES;
    use crate::race::view_store::RACE_VIEW;
    use crate::session::MACHINE;
    let mut t = TIMINGS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    take_readings(&mut t);
    let fps = f32::from_bits(FRAME_SWITCHES.frame_rate.bits());
    ui.text(format!("Frame rate (the game's, over 16 frames): {fps:.1}"));
    ui.same_line();
    ui.checkbox("Each part in the graph", &mut t.each);
    let history: Vec<f32> = t.frames.iter().copied().collect();
    let average = history.iter().sum::<f32>() / history.len().max(1) as f32;
    let worst = history.iter().copied().fold(0.0f32, f32::max);
    let top = (worst * 1.2).max(20.0);
    let line = 90.0 * ui.current_font_size() / 13.0;
    let summary = format!("{average:.1} ms a frame on average, {worst:.1} the worst");
    if t.each {
        parts_graph(ui, &t, &history, top, line * 2.0, &summary);
        let small = |label: &str, values: &VecDeque<f32>, text: String| {
            let values: Vec<f32> = values.iter().copied().collect();
            ui.plot_lines_config(label, &values)
                .overlay_text(text)
                .scale_min(0.0)
                .graph_size([-1.0, line / 2.0])
                .build();
        };
        small(
            "##pixels",
            &t.pixels,
            format!("Pixels drawn: {:.2} screens", t.frame[4]),
        );
        small(
            "##fans",
            &t.fans,
            format!("Triangle fans: {:.0}", t.frame[3]),
        );
    } else {
        ui.plot_lines_config("##frames", &history)
            .overlay_text(summary)
            .scale_min(0.0)
            .scale_max(top)
            .graph_size([-1.0, line])
            .build();
    }
    let section = |name: &str, rows: &[(&str, String)]| {
        if !ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN) {
            return;
        }
        let flags = TableFlags::BORDERS | TableFlags::ROW_BG;
        if let Some(_table) = ui.begin_table_with_flags(name, 2, flags) {
            for (label, value) in rows {
                ui.table_next_row();
                ui.table_next_column();
                ui.text(label);
                ui.table_next_column();
                ui.text(value);
            }
        }
    };
    // Under a millisecond in microseconds: input and the sound commands take a few.
    let ms = |v: f32| {
        if v < 1.0 {
            format!("{:.1} \u{b5}s", v * 1000.0)
        } else {
            format!("{v:.2} ms")
        }
    };
    section(
        "The frame",
        &[
            ("Input", ms(t.stages[Stage::Input as usize])),
            ("Physics (all ticks)", ms(t.stages[Stage::Physics as usize])),
            (
                "Scene (all its 3D work)",
                ms(t.stages[Stage::Scene as usize]),
            ),
            ("  of which the rasteriser", ms(t.frame[0])),
            ("  of which noting the triangles", ms(t.frame[1])),
            ("Sound commands", ms(t.stages[Stage::SoundTick as usize])),
            ("Showing the frame", ms(t.frame[2])),
            ("The rest (not timed, and the wait)", ms(t.rest)),
        ],
    );
    section(
        "Physics",
        &[
            ("Ticks a frame", format!("{:.1}", t.ticks)),
            (
                "Cars in the race",
                format!("{}", RACE_VIEW.race_cars.bits()),
            ),
        ],
    );
    section(
        "Drawing",
        &[
            ("Triangle fans a frame", format!("{:.0}", t.frame[3])),
            ("Pixels drawn", format!("{:.2} screens", t.frame[4])),
        ],
    );
    let (playing, stopping) = crate::debug::timing::voices();
    section(
        "Sound (its own thread)",
        &[
            (
                "A turn's work (every 20 ms)",
                ms(t.stages[Stage::AudioTurn as usize]),
            ),
            ("Voices playing", format!("{playing}")),
            ("Voices stopping", format!("{stopping}")),
            (
                "Voices made (for the race's sounds)",
                format!("{}", AUDIO.voice_count.get()),
            ),
            ("Channels in use", format!("{}", AUDIO.channels_used.get())),
            ("Mode's bits", format!("{:08x}", AUDIO.sound_mode.get())),
        ],
    );
    // The original profile page's items. `dword_6218B8` is the developers' switch: 1 in the
    // image, changed only by a key of their handler at 0x43B3D6, which nothing calls; that
    // handler is also all that masks the processor's features into `g_useSse`, `g_use3dNow`
    // and `dword_621A60`, so "Game uses" stays at none.
    let names = |flags: [(&'static str, Option<u32>); 3]| {
        let on: Vec<&str> = flags
            .iter()
            .filter(|(_, v)| v.is_some_and(|v| v != 0))
            .map(|(name, _)| *name)
            .collect();
        if on.is_empty() {
            "none".to_owned()
        } else {
            on.join(", ")
        }
    };
    section(
        "Machine",
        &[
            (
                "Course memory",
                format!("{} KiB", course.lists.course_bytes_total.get() >> 10),
            ),
            (
                "Processor has",
                names([
                    ("MMX", Some(MACHINE.has_mmx.get())),
                    ("3DNow!", Some(MACHINE.has_3dnow.get())),
                    ("SSE", Some(MACHINE.has_sse.get())),
                ]),
            ),
            (
                "Game uses",
                names([
                    ("MMX", value_at(0x621A60)),
                    ("3DNow!", value_at(0x621A5C)),
                    ("SSE", value_at(0x621A58)),
                ]),
            ),
            (
                "Developers' extensions switch",
                figure(value_at(0x6218B8), Shown::Whole),
            ),
        ],
    );
}

/// The frame's total (white) and each part, over one another, the colours named below.
fn parts_graph(ui: &Ui, t: &Timings, total: &[f32], top: f32, height: f32, summary: &str) {
    let [x, y] = ui.cursor_screen_pos();
    let width = ui.content_region_avail_width().max(1.0);
    ui.dummy([width, height]);
    let list = ui.get_window_draw_list();
    list.add_rect([x, y], [x + width, y + height], [0.1, 0.1, 0.1, 0.7])
        .filled(true)
        .build();
    let points = |values: Vec<f32>| -> Vec<[f32; 2]> {
        let step = width / (HISTORY - 1) as f32;
        let from = HISTORY - values.len();
        values
            .iter()
            .enumerate()
            .map(|(n, v)| {
                [
                    x + (from + n) as f32 * step,
                    y + height - (v / top).min(1.0) * height,
                ]
            })
            .collect()
    };
    for (n, (_, colour)) in PARTS.iter().enumerate() {
        list.add_polyline(points(t.parts[n].iter().copied().collect()), *colour)
            .thickness(1.0)
            .build();
    }
    list.add_polyline(points(total.to_vec()), [1.0, 1.0, 1.0, 1.0])
        .thickness(2.0)
        .build();
    list.add_text([x + 4.0, y + 2.0], [1.0, 1.0, 1.0, 1.0], summary);
    drop(list);
    ui.text("Total");
    for (name, colour) in PARTS {
        ui.same_line();
        ui.text_colored(colour, name);
    }
}

/// The league tables the watch pages dumped (`debug::draw_league_dump`, 0x427A79: 50 rows of
/// 8 bytes each, `off_4278DB` ..; `g_leagueRows` was dumped twice).
const DUMPS: [(&str, &str, usize, usize, usize); 4] = [
    (
        "League: the ten records' names (g_leagueNames)",
        "`race::copy_league_table` fills them; `physics::compare_split_times` reads them.",
        0x60410C,
        1,
        10,
    ),
    (
        "League: kept times, sixteen rows of ten (g_leagueRows)",
        "A row a split, a column a record; hundredths of a second.",
        0x604134,
        10,
        16,
    ),
    (
        "Car 1's times at the splits (unk_75B02C)",
        "Noted at each split; kept in the player's +0x20C after the race; hundredths of a second.",
        0x75B02C,
        1,
        16,
    ),
    (
        "Car 2's times at the splits (unk_75B09C)",
        "As car 1's.",
        0x75B09C,
        1,
        16,
    ),
];

/// A row of a dump as text.
fn dump_row(at: usize, columns: usize, row: usize) -> String {
    let figures: Vec<String> = (0..columns)
        .map(|column| {
            value_at(at + 4 * (row * columns + column))
                .map_or("?".to_owned(), |v| format!("{}", v as i32))
        })
        .collect();
    format!("{row:2}: {}", figures.join("  "))
}

/// The "Dumps" tab: what the developers' page showed as bytes, read (2026-10-30).
fn dumps(ui: &Ui) {
    if ui.button("Write the four to the log") {
        for (name, _, at, columns, rows) in DUMPS {
            crate::framework::log(name);
            for row in 0..rows {
                crate::framework::log(&dump_row(at, columns, row));
            }
        }
    }
    for (name, about, at, columns, rows) in DUMPS {
        if !ui.collapsing_header(name, TreeNodeFlags::NONE) {
            continue;
        }
        ui.text_disabled(about);
        for row in 0..rows {
            ui.text(dump_row(at, columns, row));
        }
    }
}

/// A rival's time (hundredths of a second) as minutes, seconds and hundredths.
fn time(t: u32) -> String {
    format!("{}:{:02}.{:02}", t / 6000, t / 100 % 60, t % 100)
}

/// The "Rivals" tab.
fn rivals(physics: &Physics, ui: &Ui) {
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
            physics.split_tables.rival_names[n].get(),
            &mut |run| name.extend_from_slice(run),
        );
        ui.text(name.iter().map(|&b| char::from(b)).collect::<String>());
        ui.table_next_column();
        ui.text(time(physics.split_tables.rival_scaled[n].get()));
        ui.table_next_column();
        ui.text(time(physics.split_tables.rival_times[n].get()));
    }
}
