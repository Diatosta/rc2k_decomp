//! The "Effects" tab: the effects' developers' page (`debug::effects_page_1`, 0x4D1711, no
//! references to it), its labels kept. Some figures lie through
//! `g_marksSettings` (0x767A28, now a set / not-set flag), the rest at their own places. Labels with `?` were
//! the developers' own, for the names in their source.

use crate::effects::setup::effects_settings;
use crate::owners::Owners;
use crate::race::effects_settings_store::EffectsSettings;
use dear_imgui_rs::{TreeNodeFlags, Ui};

/// Where a figure is read.
#[derive(Clone, Copy)]
enum At {
    /// A variable of the effects' or the scene's state.
    Cell(fn(&Owners<'static>) -> u32),
    /// A figure of the settings `g_marksSettings` stands for (the race's record); the windows
    /// and cracks lie in its first car's record.
    Through(fn(&EffectsSettings) -> u32),
}

/// A figure: the developers' label, its place, whether a float.
type Figure = (&'static str, At, bool);

use At::{Cell, Through};

/// The window's width for two columns.
const TWO_COLUMNS: f32 = 900.0;

const GROUPS: &[(&str, &[Figure])] = &[
    (
        "Weather and splashes",
        &[
            (
                "?rain.hang",
                Cell(|o| {
                    o.effects
                        .windscreen_drops
                        .hex
                        .windscreen_flakes_cleared
                        .get()
                }),
                false,
            ),
            (
                "?rain.total",
                Cell(|o| o.effects.windscreen_drops.state.live.get()),
                false,
            ),
            (
                "?snow3d.drawn",
                Cell(|o| o.effects.weather.snow.drawn.get()),
                false,
            ),
            (
                "splash amount",
                Cell(|o| o.effects.ring.work.count.get()),
                false,
            ),
            (
                "fworks amount",
                Cell(|o| o.effects.ring.pool.used.get()),
                false,
            ),
            (
                "fworks run",
                Cell(|o| o.effects.ring.pool._gap_93a5cc.get_u32(0)),
                false,
            ),
            (
                "fsizefp",
                Cell(|o| o.scene.drawing.scene.get().view_scale.bits()),
                true,
            ),
            (
                "maxzspacefp",
                Cell(|o| o.scene.drawing.scene.get().farthest.bits()),
                true,
            ),
        ],
    ),
    (
        "The car",
        &[
            (
                "change_gear",
                Through(|s| s.spline_gear_changed.get()),
                false,
            ),
            ("hand_brake", Through(|s| s.spline_handbrake.get()), false),
            ("backfire", Through(|s| s.cars[0].gear_notice.get()), false),
            (
                "car angle",
                Cell(|o| o.effects.frame.view_vars.heading.bits()),
                true,
            ),
            (
                "car direction of travel",
                Cell(|o| o.effects.frame.view_vars.facing.bits()),
                true,
            ),
            ("car x", Through(|s| s.listener_car_place[0].bits()), true),
            ("car y", Through(|s| s.listener_car_place[1].bits()), true),
            ("car z", Through(|s| s.listener_car_place[2].bits()), true),
            (
                "?wheels.vect1.x",
                Cell(|o| o.effects.kinds.emitter.side_b[0].bits()),
                true,
            ),
            (
                "?wheels.vect1.y",
                Cell(|o| o.effects.kinds.emitter.side_b[1].bits()),
                true,
            ),
            (
                "?wheels.vect1.z",
                Cell(|o| o.effects.kinds.emitter.side_b[2].bits()),
                true,
            ),
            (
                "?wheels.vect4.x",
                Cell(|o| o.effects.kinds.emitter.side_a[0].bits()),
                true,
            ),
            (
                "?wheels.vect4.y",
                Cell(|o| o.effects.kinds.emitter.side_a[1].bits()),
                true,
            ),
            (
                "?wheels.vect4.z",
                Cell(|o| o.effects.kinds.emitter.side_a[2].bits()),
                true,
            ),
        ],
    ),
    (
        "Clocks",
        &[
            (
                "?world.time_abs",
                Cell(|o| o.effects.setup.clock.now.get()),
                false,
            ),
            (
                "?world.time",
                Cell(|o| o.effects.setup.clock.frame_ticks.get()),
                false,
            ),
            (
                "clock0",
                Cell(|o| o.effects.setup.frame.empty_share.bits()),
                true,
            ),
            (
                "clock1",
                Cell(|o| o.effects.setup.frame.ring_share.bits()),
                true,
            ),
            (
                "clock2",
                Cell(|o| o.effects.setup.frame.whole_share.bits()),
                true,
            ),
            (
                "clock3",
                Cell(|o| o.effects.setup.frame.quads_share.bits()),
                true,
            ),
            (
                "clock4",
                Cell(|o| o.effects.setup.frame.step_share.bits()),
                true,
            ),
            (
                "clock5",
                Cell(|o| o.effects.setup.frame._gap_76512c.get_u32(0)),
                true,
            ),
            (
                "clock6",
                Cell(|o| o.effects.setup.frame._gap_76512c.get_u32(4)),
                true,
            ),
            (
                "clock7",
                Cell(|o| o.effects.setup.frame.draw_share.bits()),
                true,
            ),
        ],
    ),
    (
        "Windows",
        &[
            ("window 0", Through(|s| s.cars[0].face_sums[0].get()), false),
            ("window 1", Through(|s| s.cars[0].face_sums[1].get()), false),
            ("window 2", Through(|s| s.cars[0].face_sums[2].get()), false),
            ("window 3", Through(|s| s.cars[0].face_sums[3].get()), false),
            ("window 4", Through(|s| s.cars[0].face_sums[4].get()), false),
            ("window 5", Through(|s| s.cars[0].face_sums[5].get()), false),
        ],
    ),
];

/// The three sets of six cracks: the first car's line sums, shares and point counts.
const CRACK_SETS: [fn(&EffectsSettings, usize) -> u32; 3] = [
    |s, n| s.cars[0].kind_sums[n].get(),
    |s, n| s.cars[0].kind_shares[n].get(),
    |s, n| s.cars[0].kind_counts[n].get(),
];

fn read(o: &Owners<'static>, at: At) -> Option<u32> {
    match at {
        Cell(read) => Some(read(o)),
        Through(read) => effects_settings().ok().map(read),
    }
}

fn shown(value: Option<u32>, float: bool) -> String {
    match value {
        None => "?".to_owned(),
        Some(v) if float => format!("{:.3}", f32::from_bits(v)),
        Some(v) => format!("{}", v as i32),
    }
}

/// The tab.
pub fn tab(o: &Owners<'static>, ui: &Ui) {
    // 0x4D2B90: its warning, when the two counts differ.
    let frame = &o.effects.frame.draw_frame;
    let (drawn, run) = (
        Some(frame._gap_91ffa2.get_u32(0)),
        Some(frame.cleared.get()),
    );
    if drawn != run {
        ui.text_colored(
            [1.0, 0.4, 0.3, 1.0],
            format!(
                "draw amount not same as process amount: {} / {}",
                shown(drawn, false),
                shown(run, false)
            ),
        );
    }
    // Two columns when there is room.
    if ui.content_region_avail_width() >= TWO_COLUMNS
        && let Some(_layout) = ui.begin_table("effects", 2)
    {
        ui.table_next_row();
        ui.table_next_column();
        group(ui, o, &GROUPS[0]);
        group(ui, o, &GROUPS[2]);
        ui.table_next_column();
        group(ui, o, &GROUPS[1]);
        group(ui, o, &GROUPS[3]);
        cracks(ui, o);
    } else {
        for g in GROUPS {
            group(ui, o, g);
        }
        cracks(ui, o);
    }
}

/// A group of figures, under its header.
fn group(ui: &Ui, o: &Owners<'static>, (name, figures): &(&str, &[Figure])) {
    if ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN)
        && let Some(_table) = ui.begin_table(name, 2)
    {
        for &(label, at, float) in *figures {
            ui.table_next_row();
            ui.table_next_column();
            ui.text_disabled(label);
            ui.table_next_column();
            ui.text(shown(read(o, at), float));
        }
    }
}

/// The three sets of cracks.
fn cracks(ui: &Ui, o: &Owners<'static>) {
    if ui.collapsing_header("Cracks", TreeNodeFlags::DEFAULT_OPEN) {
        ui.text(format!(
            "?crack.flag: {}",
            shown(Some(o.effects.paths.lines_work.any.get()), false)
        ));
        for (set, figure) in CRACK_SETS.iter().enumerate() {
            let figures: Vec<String> = (0..6)
                .map(|n| shown(effects_settings().ok().map(|s| figure(s, n)), false))
                .collect();
            ui.text(format!("Set {}: {}", set + 1, figures.join("  ")));
        }
    }
}
