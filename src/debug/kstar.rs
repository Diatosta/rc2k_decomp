//! The developers' "kstar modes" (`g_maskKind`, 0x621870, cycled by a key of the unreached
//! handler at 0x43AE87; `debug::draw_pages` calls their drawers), as the debug UI's tab. In
//! RAL.EXE a mode's keys edited one thing at a time; here each is a section. IDA, 2026-10-30:
//! - 5: the car's faces by group (`g_groupTable`, `scene::car_face_hidden`); the game's own.
//! - 6: the camera's record (`sub_43B177`, the texts in `sub_427DB4`).
//! - 7: the "error list", places kept (`g_errorList`, 0x20 bytes each; the stage's file loads it).
//! - 8: a car's light places (`g_lightsData`), a flare at the light (`race::car_light_flares`).
//!   One place a pair: the left is the right's x negated (`light_pair`). The car's real lights
//!   are copies made at the load (`race::copy_lights_data`), hence "Apply".
//! - 0xC: the road's height around the car (`sub_42985B`).
//! - 0x11: the sun's point on the screen (`sub_428D75`).
//! - 0x12: the course from above (`sub_42935E`; here the checkpoints' line, not its bitmap).
//!
//! Not here: 0xB is the "Probe points" box; 3, 9, 0xA draw nothing; 4 reads lists at 0x5EE6B8
//! nothing writes; 0x10 has only 8's keys. What this tab writes is the game's memory until
//! the stage reloads, as the developers' keys did.

use crate::debug::wear_page::value_at;

use crate::race::frame_store::FRAME_WORK;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::session::Dword;
use dear_imgui_rs::{TreeNodeFlags, Ui};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// The first car's block (IDA): its model, place, heading, view, nearest checkpoint.
const CAR_KIND: usize = 0x70F410;
const CAR_PLACE: [usize; 3] = [0x70F520, 0x70F524, 0x70F528];
const CAR_HEADING: usize = 0x70FDAC;
const CAR_VIEW: usize = 0x70FA70;
const CAR_CHECKPOINT: usize = 0x70F484;

const GROUP_TABLE: usize = 0x621934;
const VIEW_NAMES: usize = 0x5EE2D5;
const ERROR_LIST: usize = 0x637660;
const LIGHTS: usize = 0x6077E8;
const CHECKPOINT_FACES: usize = 0x9E3738;

/// `off_5EE228`: the lights' names (the rest are "null"; the left ones mirror the right).
const LIGHT_NAMES: [&str; 12] = [
    "right head",
    "right spot 1",
    "right spot 2",
    "right spot 3",
    "right brake",
    "centre brake",
    "right side",
    "right reverse",
    "right fog",
    "null",
    "null",
    "null",
];

/// The camera record's figures, as `sub_43B177`: name, offset; the last is whole.
/// The record's figures the page edits, by the developers' labels.
fn camera_figures(r: &ViewRecord) -> [(&'static str, &Dword); 12] {
    [
        ("xfoc", &r.focus[0]),
        ("yfoc", &r.focus[1]),
        ("zfoc", &r.focus[2]),
        ("xoff", &r.offset[0]),
        ("yoff", &r.offset[1]),
        ("zoff", &r.offset[2]),
        ("xang", &r.angles[0]),
        ("yang", &r.angles[1]),
        ("zang", &r.angles[2]),
        ("mspx", &r.speeds[0]),
        ("mspy", &r.speeds[1]),
        ("mspz", &r.speeds[2]),
    ]
}

/// The chosen light.
static LIGHT: AtomicU32 = AtomicU32::new(0);
/// The sun's mark shown.
static SUN: AtomicBool = AtomicBool::new(false);
/// The last place kept.
static KEPT: Mutex<usize> = Mutex::new(0);

/// `v` at IDA's address `ida`, where it lives now (nothing when it cannot be written: the
/// UI does not end the game).
pub fn write_at(ida: usize, v: u32) {
    let _ = crate::moved::named(ida).and_then(|at| at.write(v, "a debug page's value"));
}

/// The dword at IDA's address, 0 when unreadable.
fn dword(ida: usize) -> u32 {
    value_at(ida).unwrap_or(0)
}

/// The fixed-point (16.16) figure with a drag; true when changed.
fn fixed(ui: &Ui, label: &str, v: &mut i32) -> bool {
    let mut shown = *v as f32 / 65536.0;
    let changed = ui
        .drag_config::<_, f32>(label)
        .speed(1.0 / 64.0)
        .build(ui, &mut shown);
    if changed {
        *v = (shown * 65536.0) as i32;
    }
    changed
}

/// A section's header, open at first only for the mode's own choice.
fn section(ui: &Ui, name: &str) -> bool {
    ui.collapsing_header(name, TreeNodeFlags::NONE)
}

/// The tab.
pub fn tab(ui: &Ui) {
    mode_choice(ui);
    if section(ui, "Car's faces by group (mode 5)") {
        groups(ui);
    }
    if section(ui, "Camera (mode 6)") {
        camera(ui);
    }
    if section(ui, "Kept places (mode 7)") {
        kept_places(ui);
    }
    if section(ui, "Car's lights (mode 8)") {
        lights(ui);
    }
    if section(ui, "Road's height ahead and behind (mode 0xC)") {
        heights(ui);
    }
    if section(ui, "Course from above (mode 0x12)") {
        course_map(ui);
    }
    if section(ui, "Sun (mode 0x11)") {
        let mut on = SUN.load(Relaxed);
        if ui.checkbox("Mark the sun's point on the screen", &mut on) {
            SUN.store(on, Relaxed);
        }
        ui.text(format!(
            "On the screen: {}; at ({}, {})",
            FRAME_WORK.sun_on_screen.get(),
            FRAME_WORK.sun_hit[0].get() as i32,
            FRAME_WORK.sun_hit[1].get() as i32
        ));
    }
}

/// The value of `g_maskKind` itself: only 5 and 8 change what the game draws.
fn mode_choice(ui: &Ui) {
    let now = FRAME_SWITCHES.mask_kind.get();
    ui.text("The game's own mode:");
    for (name, mode) in [
        ("Off", 0),
        ("5: only the groups ticked", 5),
        ("8: a flare at the light", 8),
    ] {
        ui.same_line();
        if ui.radio_button_bool(name, now == mode) {
            FRAME_SWITCHES.mask_kind.set(mode);
        }
    }
}

/// Mode 5's table: with the mode on, a car's face shows only if its group's entry is set.
fn groups(ui: &Ui) {
    ui.text_disabled("Takes effect with the game's mode at 5; all start unticked (no car)");
    if ui.button("Tick all") {
        (0..32).for_each(|g| write_at(GROUP_TABLE + 4 * g, 1));
    }
    ui.same_line();
    if ui.button("Untick all") {
        (0..32).for_each(|g| write_at(GROUP_TABLE + 4 * g, 0));
    }
    for g in 0..32 {
        if g % 8 != 0 {
            ui.same_line();
        }
        let at = GROUP_TABLE + 4 * g;
        let mut on = dword(at) != 0;
        if ui.checkbox(format!("{g:2}"), &mut on) {
            write_at(at, u32::from(on));
        }
    }
}

/// Mode 6: the record of the view the car is in.
fn camera(ui: &Ui) {
    let view = (dword(CAR_VIEW) as i32).clamp(0, 0x33) as usize;
    let mut name = Vec::new();
    for k in 0..4 {
        name.extend_from_slice(&dword(VIEW_NAMES + 16 * view + 4 * k).to_le_bytes());
    }
    let name: String = name
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| char::from(b))
        .collect();
    ui.text(format!("View {view}: {}", name.trim_end()));
    ui.text_disabled("Changes the view's record until the stage is loaded again");
    let Ok(record) = ViewVars::record_of(view) else {
        ui.text("This view's record is not readable");
        return;
    };
    for (label, cell) in camera_figures(record) {
        let mut v = f32::from_bits(cell.get());
        if ui
            .drag_config::<_, f32>(label)
            .speed(1.0 / 64.0)
            .build(ui, &mut v)
        {
            cell.set(v.to_bits());
        }
    }
    let mut fish = record.fish.get() as i32;
    if ui.drag("fish", &mut fish) {
        record.fish.set(fish as u32);
    }
}

/// Mode 7: the places kept: 0x20 bytes each: view, x, y, z, heading, .. in use at +0x1C.
fn kept_places(ui: &Ui) {
    const ENTRIES: usize = 0x100;
    let at = |n: usize, offset: usize| ERROR_LIST + 0x20 * n + offset;
    let mut kept = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if ui.button("Keep the car's place") {
        // The first free, as the original's key (0x43B0BF).
        if let Some(n) = (0..ENTRIES).find(|&n| dword(at(n, 0x1C)) == 0) {
            write_at(at(n, 0), dword(CAR_VIEW));
            for (k, place) in CAR_PLACE.iter().enumerate() {
                write_at(at(n, 4 + 4 * k), dword(*place));
            }
            write_at(at(n, 0x10), dword(CAR_HEADING));
            write_at(at(n, 0x1C), 1);
            *kept = n;
        }
    }
    ui.same_line();
    ui.text_disabled("Kept in memory only, until the stage is loaded again");
    for n in (0..ENTRIES).filter(|&n| dword(at(n, 0x1C)) != 0) {
        let _id = ui.push_id(n);
        let place = [4, 8, 12].map(|o| dword(at(n, o)) as i32 >> 16);
        ui.text(format!(
            "{n:3}{} at ({}, {}, {})",
            if n == *kept { "*" } else { " " },
            place[0],
            place[1],
            place[2]
        ));
        ui.same_line();
        if ui.button("Go") {
            // As the original's key (0x43B13E): the car put there.
            for (k, place) in CAR_PLACE.iter().enumerate() {
                write_at(*place, dword(at(n, 4 + 4 * k)));
            }
            write_at(CAR_HEADING, dword(at(n, 0x10)));
            write_at(CAR_VIEW, dword(at(n, 0)));
            *kept = n;
        }
        ui.same_line();
        if ui.button("Forget") {
            write_at(at(n, 0x1C), 0);
        }
    }
}

/// Mode 8: the car model's lights: 12 places of 3 dwords, 12 x 12 a model.
fn lights(ui: &Ui) {
    let kind = dword(CAR_KIND) as usize;
    let mut light = LIGHT.load(Relaxed) as usize % LIGHT_NAMES.len();
    ui.text(format!("Car model {kind}"));
    for (n, name) in LIGHT_NAMES.iter().enumerate().take(9) {
        if n % 3 != 0 {
            ui.same_line();
        }
        if ui.radio_button_bool(*name, light == n) {
            light = n;
            LIGHT.store(n as u32, Relaxed);
        }
    }
    ui.same_line();
    if ui.button("Apply to the car's real lights") {
        let _ = crate::race::stage_prefs::copy_lights_data();
    }
    ui.text_disabled("One place a pair: the left light is the right's x negated");
    let base = LIGHTS + (kind * 12 + light) * 12;
    for (k, label) in ["xoff", "yoff", "zoff"].iter().enumerate() {
        let at = base + 4 * k;
        let mut v = dword(at) as i32;
        if fixed(ui, label, &mut v) {
            write_at(at, v as u32);
        }
        // The original's key loop leaves the place where `car_light_flares` reads it.
        FRAME_WORK.part_place[k].set(dword(at));
    }
}

/// A checkpoint's place: the first point of its face (`sub_42985B`).
fn checkpoint_place(n: usize) -> Option<[i32; 3]> {
    let face = value_at(CHECKPOINT_FACES + 4 * n)?;
    let point =
        crate::race::course_places::face_field::<u32>(face, 4, "a checkpoint's face").ok()?;
    let place = crate::race::scene_points::point_number(point).ok()?;
    Some(place.map(|v| v as i32))
}

/// Mode 0xC: 25 checkpoints around the car's, heights less its own.
fn heights(ui: &Ui) {
    let count = RACE_VARS.checkpoint_count.get() as usize;
    let here = dword(CAR_CHECKPOINT) as usize;
    let Some(own) = checkpoint_place(here).map(|p| p[1]) else {
        ui.text("No checkpoints");
        return;
    };
    let values: Vec<f32> = (0..25usize)
        .filter_map(|k| (here + k).checked_sub(12).filter(|&n| n < count))
        .filter_map(checkpoint_place)
        .map(|p| (p[1] - own) as f32 / 65536.0)
        .collect();
    ui.text(format!("Checkpoint {here} of {count}"));
    ui.plot_lines_config("##heights", &values)
        .overlay_text("12 checkpoints behind .. 12 ahead")
        .graph_size([-1.0, 80.0 * ui.current_font_size() / 13.0])
        .build();
}

/// Mode 0x12: the checkpoints' line from above and the car on it.
fn course_map(ui: &Ui) {
    let count = RACE_VARS.checkpoint_count.get() as usize;
    let places: Vec<[i32; 3]> = (0..count)
        .step_by((count / 2000).max(1))
        .filter_map(checkpoint_place)
        .collect();
    if places.len() < 2 {
        ui.text("No checkpoints");
        return;
    }
    let range = |k: usize| {
        let values = places.iter().map(|p| f64::from(p[k]));
        (
            values.clone().fold(f64::MAX, f64::min),
            values.fold(f64::MIN, f64::max),
        )
    };
    let ((x0, x1), (z0, z1)) = (range(0), range(2));
    let [ox, oy] = ui.cursor_screen_pos();
    let width = ui.content_region_avail_width().max(1.0);
    let height = (300.0 * ui.current_font_size() / 13.0).min(width);
    ui.dummy([width, height]);
    let scale = f64::from(width.min(height) - 8.0) / (x1 - x0).max(z1 - z0).max(1.0);
    let to = |x: f64, z: f64| {
        [
            ox + 4.0 + ((x - x0) * scale) as f32,
            oy + height - 4.0 - ((z - z0) * scale) as f32,
        ]
    };
    let list = ui.get_window_draw_list();
    let line: Vec<[f32; 2]> = places
        .iter()
        .map(|p| to(f64::from(p[0]), f64::from(p[2])))
        .collect();
    list.add_polyline(line, [1.0, 1.0, 0.4, 1.0])
        .thickness(1.5)
        .build();
    let car = to(
        f64::from(dword(CAR_PLACE[0]) as i32),
        f64::from(dword(CAR_PLACE[2]) as i32),
    );
    list.add_circle(car, 4.0, [1.0, 0.2, 0.2, 1.0])
        .filled(true)
        .build();
}

/// What goes over the picture (once a UI frame).
pub fn over_picture(ui: &Ui) {
    if !SUN.load(Relaxed) || FRAME_WORK.sun_on_screen.get() == 0 {
        return;
    }
    let [w, h] = ui.io().display_size();
    let (width, height) = crate::platform::screen_size();
    let at = [
        FRAME_WORK.sun_hit[0].get() as i32 as f32 * w / width.max(1) as f32,
        FRAME_WORK.sun_hit[1].get() as i32 as f32 * h / height.max(1) as f32,
    ];
    let size = ui.current_font_size() / 2.0;
    ui.get_foreground_draw_list()
        .add_circle(at, size, [1.0, 1.0, 0.0, 1.0])
        .thickness(2.0)
        .build();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_table_is_reachable() {
        for at in [
            CAR_KIND,
            CAR_HEADING,
            CAR_VIEW,
            CAR_CHECKPOINT,
            GROUP_TABLE,
            VIEW_NAMES,
            ERROR_LIST,
            LIGHTS,
            CHECKPOINT_FACES,
        ] {
            assert!(value_at(at).is_some(), "{at:#x}");
        }
        // The view names' text, read through the same way.
        assert_eq!(dword(VIEW_NAMES).to_le_bytes(), *b"bump");
    }
}
