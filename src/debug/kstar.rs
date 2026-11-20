//! The developers' "kstar modes" as the debug UI's tab; each mode is a section: 5 the car's faces
//! by group, 6 the camera's record, 7 the "error list" (places kept; the stage's file loads it), 8
//! a car's light places with a flare at the light (one place a pair, the left the right's x
//! negated; the car's real lights are copies made at the load, hence "Apply"), 0xC the road's
//! height around the car, 0x11 the sun's point on the screen, 0x12 the course from above (the
//! checkpoints' line). Not here: 0xB is the "Probe points" box; 3, 9, 0xA draw nothing; 4 reads
//! lists nothing writes; 0x10 has only 8's keys. What this tab writes is the game's memory until
//! the stage reloads, as the developers' keys did.

use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::{ViewRecord, ViewVars};
use crate::scene::owner::Scene;
use dear_imgui_rs::{TreeNodeFlags, Ui};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// The first car's block: its model, place, heading, view, nearest checkpoint.
fn first_car(race: &Race) -> &Car {
    &race.car_blocks().cars[0]
}

/// The kept places: 0x100 entries of 0x20 bytes.
const ERROR_ENTRIES: usize = 0x100;
const GROUP_COUNT: usize = 32;
/// The lights' table: 0x22 models of 12 lights of 3 dwords.
const LIGHTS_LEN: usize = 0x1320;
const VIEW_NAMES: [&str; 52] = [
    "bumper",
    "bonnet",
    "driver",
    "drivermid",
    "chase",
    "medchase",
    "farchase",
    "fixedreplay",
    "zoomreplay",
    "fixedreplayroad",
    "zoomreplayroad",
    "overhead1",
    "overhead2",
    "overhead3",
    "overhead4",
    "overhead5",
    "frontlow",
    "frontmed",
    "leftlow",
    "leftmid",
    "rightlow",
    "rightmid",
    "frontleft34",
    "frontright34",
    "backleft34",
    "backright34",
    "rearbumper",
    "roofrear",
    "flwheel",
    "frwheel",
    "blwheel",
    "brwheel",
    "intowindscreen",
    "intoleftwindow",
    "intorightwindow",
    "random",
    "randomdynamic",
    "randomfixed",
    "fronttoback",
    "backtofront",
    "rotfromback",
    "rotfromfront",
    "freproadud",
    "freproadudz",
    "freproaddu",
    "freproadduz",
    "freptrackud",
    "freptrackudz",
    "freptrackdu",
    "freptrackduz",
    "widezoomin",
    "widezoomout",
];

/// The lights' names (the rest are "null"; the left ones mirror the right).
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

/// The camera record's figures the page edits, by the developers' labels: name, offset; the
/// last is whole.
fn camera_figures(r: &ViewRecord) -> [(&'static str, &PlainDword); 12] {
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

/// Group `g`'s entry in the car's face table, to write in the scene's draw switches.
fn set_group(scene: &Scene, g: usize, value: u32) {
    scene.draw_switches_mut().group_table[g] = value;
}

/// The error list's dword `offset` bytes into entry `n`.
fn kept(race: &Race, n: usize, offset: usize) -> u32 {
    race.frame.error_list().list.get_u32(0x20 * n + offset)
}

fn set_kept(race: &Race, n: usize, offset: usize, v: u32) {
    race.frame.error_list().list.set_u32(0x20 * n + offset, v);
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

pub fn tab(race: &Race, scene: &Scene, ui: &Ui) {
    mode_choice(&race.shared, ui);
    if section(ui, "Car's faces by group (mode 5)") {
        groups(scene, ui);
    }
    if section(ui, "Camera (mode 6)") {
        camera(race, ui);
    }
    if section(ui, "Kept places (mode 7)") {
        kept_places(race, ui);
    }
    if section(ui, "Car's lights (mode 8)") {
        lights(race, ui);
    }
    if section(ui, "Road's height ahead and behind (mode 0xC)") {
        heights(race, ui);
    }
    if section(ui, "Course from above (mode 0x12)") {
        course_map(race, ui);
    }
    if section(ui, "Sun (mode 0x11)") {
        let mut on = SUN.load(Relaxed);
        if ui.checkbox("Mark the sun's point on the screen", &mut on) {
            SUN.store(on, Relaxed);
        }
        ui.text(format!(
            "On the screen: {}; at ({}, {})",
            scene.scratch.main_view(0).polygon.sun.on_screen.get(),
            scene.scratch.main_view(0).polygon.sun.hit[0].get() as i32,
            scene.scratch.main_view(0).polygon.sun.hit[1].get() as i32
        ));
    }
}

/// The mode value itself: only 5 and 8 change what the game draws.
fn mode_choice(shared: &RaceShared, ui: &Ui) {
    let now = shared.switches.get().mask_kind.get();
    ui.text("The game's own mode:");
    for (name, mode) in [
        ("Off", 0),
        ("5: only the groups ticked", 5),
        ("8: a flare at the light", 8),
    ] {
        ui.same_line();
        if ui.radio_button_bool(name, now == mode) {
            shared.switches.get().mask_kind.set(mode);
        }
    }
}

/// Mode 5's table: with the mode on, a car's face shows only if its group's entry is set.
fn groups(scene: &Scene, ui: &Ui) {
    ui.text_disabled("Takes effect with the game's mode at 5; all start unticked (no car)");
    if ui.button("Tick all") {
        (0..GROUP_COUNT).for_each(|g| set_group(scene, g, 1));
    }
    ui.same_line();
    if ui.button("Untick all") {
        (0..GROUP_COUNT).for_each(|g| set_group(scene, g, 0));
    }
    for g in 0..GROUP_COUNT {
        if g % 8 != 0 {
            ui.same_line();
        }
        let mut on = scene.draw_switches().group_table[g] != 0;
        if ui.checkbox(format!("{g:2}"), &mut on) {
            set_group(scene, g, u32::from(on));
        }
    }
}

/// Mode 6: the record of the view the car is in.
fn camera(race: &Race, ui: &Ui) {
    let view = (first_car(race).camera_view.get() as i32).clamp(0, 0x33) as usize;
    ui.text(format!("View {view}: {}", VIEW_NAMES[view]));
    ui.text_disabled("Changes the view's record until the stage is loaded again");
    let Ok(record) = ViewVars::record_of(race.frame.view_and_records.get(), view) else {
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

/// Mode 7: the places kept: each with its view, x, y, z, heading, .. and an in-use flag.
fn kept_places(race: &Race, ui: &Ui) {
    let mut last = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if ui.button("Keep the car's place") {
        // The first free.
        if let Some(n) = (0..ERROR_ENTRIES).find(|&n| kept(race, n, 0x1C) == 0) {
            let car = first_car(race);
            set_kept(race, n, 0, car.camera_view.get());
            for (k, place) in car.position.iter().enumerate() {
                set_kept(race, n, 4 + 4 * k, place.get());
            }
            set_kept(race, n, 0x10, car.car_heading.get());
            set_kept(race, n, 0x1C, 1);
            *last = n;
        }
    }
    ui.same_line();
    ui.text_disabled("Kept in memory only, until the stage is loaded again");
    for n in (0..ERROR_ENTRIES).filter(|&n| kept(race, n, 0x1C) != 0) {
        let _id = ui.push_id(n);
        let place = [4, 8, 12].map(|o| kept(race, n, o) as i32 >> 16);
        ui.text(format!(
            "{n:3}{} at ({}, {}, {})",
            if n == *last { "*" } else { " " },
            place[0],
            place[1],
            place[2]
        ));
        ui.same_line();
        if ui.button("Go") {
            // The car put there.
            let car = first_car(race);
            for (k, place) in car.position.iter().enumerate() {
                place.set(kept(race, n, 4 + 4 * k));
            }
            car.car_heading.set(kept(race, n, 0x10));
            car.camera_view.set(kept(race, n, 0));
            *last = n;
        }
        ui.same_line();
        if ui.button("Forget") {
            set_kept(race, n, 0x1C, 0);
        }
    }
}

/// Mode 8: the car model's lights: 12 places of 3 dwords, 12 x 12 a model.
fn lights(race: &Race, ui: &Ui) {
    let frame = &race.frame;
    let kind = first_car(race).car_kind.get() as usize;
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
        let _ = crate::race::stage_prefs::copy_lights_data(race);
    }
    ui.text_disabled("One place a pair: the left light is the right's x negated");
    // The model's table is 0x22 models long; a model past it shows nothing.
    let table = &race.course.lights_data().lights;
    let base = (kind * 12 + light) * 12;
    for (k, label) in ["xoff", "yoff", "zoff"].iter().enumerate() {
        let at = base + 4 * k;
        let readable = at + 4 <= LIGHTS_LEN;
        let word = if readable { table.get_u32(at) } else { 0 };
        let mut v = word as i32;
        if fixed(ui, label, &mut v) && readable {
            table.set_u32(at, v as u32);
        }
        // The place where `car_light_flares` reads it.
        frame.frame_work.get().part_place[k].set(if readable { table.get_u32(at) } else { 0 });
    }
}

/// A checkpoint's place: the first point of its face.
fn checkpoint_place(race: &Race, n: usize) -> Option<[i32; 3]> {
    let face = race.checkpoints().checkpoint_faces.get(n)?.get();
    let point = crate::race::course_places::face_field::<u32>(race.course.reads(), face, 4).ok()?;
    let place = crate::race::scene_points::point_number(
        race.course.reads(),
        &crate::race::scene_points::no_own(),
        point,
    )
    .ok()?;
    Some(place.map(|v| v as i32))
}

/// Mode 0xC: 25 checkpoints around the car's, heights less its own.
fn heights(race: &Race, ui: &Ui) {
    let count = race.shared.vars.get().checkpoint_count.get() as usize;
    let here = first_car(race).checkpoint.get() as usize;
    let Some(own) = checkpoint_place(race, here).map(|p| p[1]) else {
        ui.text("No checkpoints");
        return;
    };
    let values: Vec<f32> = (0..25usize)
        .filter_map(|k| (here + k).checked_sub(12).filter(|&n| n < count))
        .filter_map(|n| checkpoint_place(race, n))
        .map(|p| (p[1] - own) as f32 / 65536.0)
        .collect();
    ui.text(format!("Checkpoint {here} of {count}"));
    ui.plot_lines_config("##heights", &values)
        .overlay_text("12 checkpoints behind .. 12 ahead")
        .graph_size([-1.0, 80.0 * ui.current_font_size() / 13.0])
        .build();
}

/// Mode 0x12: the checkpoints' line from above and the car on it.
fn course_map(race: &Race, ui: &Ui) {
    let count = race.shared.vars.get().checkpoint_count.get() as usize;
    let places: Vec<[i32; 3]> = (0..count)
        .step_by((count / 2000).max(1))
        .filter_map(|n| checkpoint_place(race, n))
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
        f64::from(first_car(race).position[0].get() as i32),
        f64::from(first_car(race).position[2].get() as i32),
    );
    list.add_circle(car, 4.0, [1.0, 0.2, 0.2, 1.0])
        .filled(true)
        .build();
}

/// What goes over the picture (once a UI frame).
pub(super) fn over_picture(
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
    ui: &Ui,
) {
    if !SUN.load(Relaxed) || scene.scratch.main_view(0).polygon.sun.on_screen.get() == 0 {
        return;
    }
    let [w, h] = ui.io().display_size();
    let (width, height) = crate::shell::frame::screen_size(system);
    let at = [
        scene.scratch.main_view(0).polygon.sun.hit[0].get() as i32 as f32 * w / width.max(1) as f32,
        scene.scratch.main_view(0).polygon.sun.hit[1].get() as i32 as f32 * h
            / height.max(1) as f32,
    ];
    let size = ui.current_font_size() / 2.0;
    ui.get_foreground_draw_list()
        .add_circle(at, size, [1.0, 1.0, 0.0, 1.0])
        .thickness(2.0)
        .build();
}
