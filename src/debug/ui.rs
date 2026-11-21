//! The debug UI: one semi-transparent ImGui window over the game, resizable, its pages as tabs:
//! "Display" (the display page's figures, named and grouped, bars as bars, filtered by name; a wide
//! window lays the groups out in columns), "Profiler" (the developers' profile page's live items
//! and our own timings), "Watch list" (the developers' watch pages' variables) and "Dumps" (the
//! league tables those pages dumped in hex), "Rivals" (the rivals' table the display page ended
//! with, as it is), and the race's freeze as a check box.

use crate::debug::timing::STAGES;
use crate::debug::wear_page::SURFACES;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::race::owner_physics::Physics;
use dear_imgui_rs::{Condition, TableColumnFlags, TableColumnWidth, TableFlags, TreeNodeFlags, Ui};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

#[derive(Clone, Copy)]
pub enum Shown {
    Whole,
    Float,
    Hex,
    Surface,
}

/// A figure of the display page: its name, the developers' label, how it is shown, the cell
/// read, whether the page drew a bar of it, and its place on the page's 800 x 600 screen.
pub struct Row {
    name: &'static str,
    label: &'static str,
    shown: Shown,
    cell: for<'a> fn(&Owners<'a>) -> u32,
    bar: bool,
    place: (u32, u32),
}

impl Row {
    pub const fn typed(
        name: &'static str,
        label: &'static str,
        shown: Shown,
        cell: for<'a> fn(&Owners<'a>) -> u32,
        bar: bool,
        place: (u32, u32),
    ) -> Self {
        Self {
            name,
            label,
            shown,
            cell,
            bar,
            place,
        }
    }

    fn value(&self, o: &Owners<'static>) -> u32 {
        (self.cell)(o)
    }
}

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

pub(super) fn note_view(frame: &Frame, scene: &crate::scene::owner::Scene) {
    let view = (
        std::array::from_fn(|n| frame.matrices.get().view_matrix[n].get()),
        scene
            .scratch
            .main_view(0)
            .polygon
            .window
            .borrow()
            .view_scale,
        [
            scene
                .scratch
                .main_view(0)
                .polygon
                .window
                .borrow()
                .view_centre[0],
            scene
                .scratch
                .main_view(0)
                .polygon
                .window
                .borrow()
                .view_centre[1],
        ],
    );
    *VIEW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(view);
    crate::debug::frame_figures::note_first_view(scene);
}

/// After the developers' mode 0xB, which drew the probe lists as one pixel each: the physics'
/// probe points as last worked out (`physics::probe_ground`, in the world, whole numbers),
/// through the view noted, each with a line down to the ground found under it. Green the
/// first four (the wheels, a reading), red a point at or under the ground, white the rest;
/// yellow the ground. Above the window, and behind the eye skipped.
fn draw_probes(race: &Race, system: &crate::system::owner::System, ui: &Ui) {
    let physics = &race.physics;
    let wk = physics.probes().borrow();
    let Some((m, scale, centre)) = *VIEW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    else {
        return;
    };
    let [w, h] = ui.io().display_size();
    let (width, height) = crate::shell::frame::screen_size(system);
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
    for n in 0..race.shared.car().probe_count.get().min(0x40) as usize {
        let (x, y, z) = (
            wk.probe_x[n] as i32,
            wk.probe_y[n] as i32,
            wk.probe_z[n] as i32,
        );
        let Some(p) = place([x, y, z]) else {
            continue;
        };
        let found = wk.probe_found[n] != 0;
        if found && let Some(g) = place([x, wk.probe_ground_height[n] as i32, z]) {
            list.add_line(p, g, [1.0, 1.0, 0.0, 1.0]).build();
        }
        let colour = if found && (wk.probe_height[n] as i32) <= 0 {
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

/// A tab's contents, scrolled under the tabs (which stay).
fn body(ui: &Ui, f: impl FnOnce()) {
    ui.child_window("body").build(ui, f);
}

pub fn build(o: &Owners<'static>, ui: &Ui) {
    let race = o.race;
    crate::debug::frame_figures::take();
    if PROBES.load(Relaxed) {
        draw_probes(race, o.system, ui);
    }
    crate::debug::kstar::over_picture(o.system, o.scene, ui);
    // The window keeps its share of the screen when the mode changes (the menus' 640 x 480,
    // the race's own).
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
                     the first four (the wheels: their points feed `wheel_ground_*` and `wheel_lifted`), red at or under the ground, \
                     white the rest; the yellow line goes down to the ground found",
                );
            }
            ui.same_line();
            ui.text_disabled("Ctrl+F11 hides this");
            if let Some(_bar) = ui.tab_bar("pages") {
                if let Some(_tab) = ui.tab_item("Display") {
                    body(ui, || {
                        display(ui, o, crate::debug::display_rows::GROUPS, &FILTER);
                    });
                }
                if let Some(_tab) = ui.tab_item("Watch list") {
                    body(ui, || {
                        display(ui, o, crate::debug::watch_rows::GROUPS, &WATCH_FILTER);
                    });
                }
                if let Some(_tab) = ui.tab_item("Dumps") {
                    body(ui, || dumps(race, ui));
                }
                if let Some(_tab) = ui.tab_item("Profiler") {
                    body(ui, || profiler(o, ui));
                }
                if let Some(_tab) = ui.tab_item("GPU") {
                    body(ui, || crate::debug::gpu_cuts::tab(race, ui));
                }
                if let Some(_tab) = ui.tab_item("Kstar modes") {
                    body(ui, || crate::debug::kstar::tab(race, o.scene, ui));
                }
                if let Some(_tab) = ui.tab_item("Switches") {
                    body(ui, || crate::debug::switches::tab(race, ui));
                }
                if let Some(_tab) = ui.tab_item("Rivals") {
                    body(ui, || rivals(crate::strings::append_string_by_id::StringOwners::of(*o), &race.physics, ui));
                }
                if let Some(_tab) = ui.tab_item("Effects") {
                    body(ui, || crate::debug::effects_page::tab(o.race.effects.get(), o.scene, ui));
                }
                if let Some(_tab) = ui.tab_item("Dyno") {
                    body(ui, || crate::debug::dyno::tab(race, o.scene, ui));
                }
                if let Some(_tab) = ui.tab_item("Players") {
                    body(ui, || crate::debug::players::tab(o.menus, race, ui));
                }
                if let Some(_tab) = ui.tab_item("Screens") {
                    body(ui, || screens(o.saves, ui));
                }
                if let Some(_tab) = ui.tab_item("Developers") {
                    body(ui, || crate::debug::dev_front_end::tab(race, o.scene, ui));
                }
            }
        });
}

/// The screens no route reaches.
fn screens(saves: &crate::saves::owner::Saves, ui: &Ui) {
    ui.text("Section best times (shown at the next menu's frame):");
    for section in 0..6 {
        let label = format!("Section {}", section + 1);
        if crate::debug::screens::section_has_times(saves, section) {
            if ui.button(&label) {
                crate::debug::screens::ask_section_times(section as u32);
            }
        } else {
            ui.text_disabled(format!("{label}: no table"));
        }
        ui.same_line();
    }
    ui.new_line();
    if ui.button("Car specifications") {
        crate::debug::screens::ask_cspecs();
    }
}

fn figure(value: u32, shown: Shown) -> String {
    let v = value;
    match shown {
        Shown::Whole => format!("{}", v as i32),
        Shown::Float => format!("{:.3}", f32::from_bits(v)),
        Shown::Hex => format!("{v:08x}"),
        Shown::Surface => String::from_utf8_lossy(SURFACES[(v & 0xF) as usize]).into_owned(),
    }
}

/// Whether a row passes the filter (`wanted` in lower case).
fn wanted_row(row: &Row, wanted: &str) -> bool {
    wanted.is_empty()
        || row.name.to_lowercase().contains(wanted)
        || row.label.to_lowercase().contains(wanted)
}

/// A tab of groups, in as many columns as the width holds.
fn display(ui: &Ui, o: &Owners<'static>, all: &'static [Group], filter: &std::sync::Mutex<String>) {
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
                group_table(ui, o, group, &wanted);
            }
        }
    }
}

fn group_table(ui: &Ui, o: &Owners<'static>, group: &Group, wanted: &str) {
    let flags = TableFlags::BORDERS | TableFlags::ROW_BG | TableFlags::RESIZABLE;
    let Some(_table) = ui.begin_table_with_flags(group.name, 4, flags) else {
        return;
    };
    // The name wide, the figure narrow.
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
    for (number, row) in group
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| wanted_row(r, wanted))
    {
        let value = row.value(o);
        ui.table_next_row();
        ui.table_next_column();
        ui.text(row.name);
        ui.table_next_column();
        ui.text(figure(value, row.shown));
        ui.table_next_column();
        if row.bar {
            let v = f32::from_bits(value);
            let _id = ui.push_id(number);
            ui.progress_bar(v / 100.0)
                .size([-1.0, 0.0])
                .overlay_text("")
                .build();
        }
        ui.table_next_column();
        ui.text_disabled(row.label);
        if ui.is_item_hovered() {
            let hex = format!("{value:08x}");
            ui.tooltip_text(if row.place == (0, 0) {
                hex
            } else {
                format!(
                    "{hex}; at ({}, {}) on the developers' page",
                    row.place.0, row.place.1
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
fn take_readings(system: &crate::system::owner::System, t: &mut Timings) {
    let now = Counts {
        at: std::time::Instant::now(),
        frame: crate::shell::frame::frame_counters(),
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
        let screen = crate::shell::frame::screen_pixels(system).max(1) as f32;
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

fn profiler(o: &Owners<'static>, ui: &Ui) {
    let race = o.race;
    let course = &race.course;
    let state = &o.audio.state;
    let shared = &race.shared;
    use crate::debug::timing::Stage;
    let mut t = TIMINGS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    take_readings(o.system, &mut t);
    let fps = shared.switches.get().frame_rate.get();
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
            (
                "  of which the course's faces",
                ms(t.stages[Stage::CourseEntries as usize]),
            ),
            (
                "  of which the depth-sorted entries",
                ms(t.stages[Stage::DepthEntries as usize]),
            ),
            (
                "  of which the mirror",
                ms(t.stages[Stage::Mirror as usize]),
            ),
            (
                "  of which the effects",
                ms(t.stages[Stage::Effects as usize]),
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
                format!("{}", shared.view.get().race_cars.get()),
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
                format!("{}", state.voice_count.get()),
            ),
            ("Channels in use", format!("{}", state.channels_used.get())),
            ("Mode's bits", format!("{:08x}", state.sound_mode.get())),
        ],
    );
    section(
        "Machine",
        &[(
            "Course memory",
            format!("{} KiB", course.lists.get().course_bytes_total.get() >> 10),
        )],
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

/// A dump's dword by its number.
type DumpWord = fn(&Race, usize) -> u32;

/// The league tables the watch pages dumped (`debug::draw_league_dump`: 50 rows of 8 bytes
/// each; the league rows were dumped twice).
const DUMPS: [(&str, &str, DumpWord, usize, usize); 4] = [
    (
        "League: the ten records' names (g_leagueNames)",
        "`race::copy_league_table` fills them; `physics::compare_split_times` reads them.",
        |race, n| race.course.league().names[n],
        1,
        10,
    ),
    (
        "League: kept times, sixteen rows of ten (g_leagueRows)",
        "A row a split, a column a record; hundredths of a second.",
        |race, n| race.course.league().rows[n],
        10,
        16,
    ),
    (
        "Car 1's times at the splits",
        "Noted at each split; kept in the player's +0x20C after the race; hundredths of a second.",
        |race, n| race.shared.state.results.results[0].data.get_u32(4 * n),
        1,
        16,
    ),
    (
        "Car 2's times at the splits",
        "As car 1's.",
        |race, n| race.shared.state.results.results[1].data.get_u32(4 * n),
        1,
        16,
    ),
];

fn dump_row(race: &Race, word: DumpWord, columns: usize, row: usize) -> String {
    let figures: Vec<String> = (0..columns)
        .map(|column| format!("{}", word(race, row * columns + column) as i32))
        .collect();
    format!("{row:2}: {}", figures.join("  "))
}

/// The "Dumps" tab: what the developers' page showed as bytes, read.
fn dumps(race: &Race, ui: &Ui) {
    if ui.button("Write the four to the log") {
        for (name, _, word, columns, rows) in DUMPS {
            crate::program_log::log(name);
            for row in 0..rows {
                crate::program_log::log(&dump_row(race, word, columns, row));
            }
        }
    }
    for (name, about, word, columns, rows) in DUMPS {
        if !ui.collapsing_header(name, TreeNodeFlags::NONE) {
            continue;
        }
        ui.text_disabled(about);
        for row in 0..rows {
            ui.text(dump_row(race, word, columns, row));
        }
    }
}

/// A rival's time (hundredths of a second) as minutes, seconds and hundredths.
fn time(t: u32) -> String {
    format!("{}:{:02}.{:02}", t / 6000, t / 100 % 60, t % 100)
}

fn rivals(so: crate::strings::append_string_by_id::StringOwners<'_>, physics: &Physics, ui: &Ui) {
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
            so,
            physics.split_tables().rival_names[n].to_file(),
            &mut |run| name.extend_from_slice(run),
        );
        ui.text(name.iter().map(|&b| char::from(b)).collect::<String>());
        ui.table_next_column();
        ui.text(time(physics.split_tables().rival_scaled[n]));
        ui.table_next_column();
        ui.text(time(physics.split_tables().rival_times[n]));
    }
}
