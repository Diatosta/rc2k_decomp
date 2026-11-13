//! The "Effects" tab: the effects' developers' page (`debug::effects_page_1`), its labels kept.
//! Some figures lie through the settings (now a set / not-set flag), the rest at their own
//! places. Labels with `?` were the developers' own, for the names in their source.

use crate::effects::setup::effects_settings;
use crate::owners::Owners;
use crate::race::effects_settings_store::EffectsSettings;
use dear_imgui_rs::{TreeNodeFlags, Ui};

/// Where a figure is read.
#[derive(Clone, Copy)]
enum At {
    /// A variable of the effects' or the scene's state.
    Cell(fn(&Owners<'static>) -> Figure),
    /// A figure of the effects' settings (the race's record); the windows and cracks lie in its
    /// first car's record.
    Through(fn(&EffectsSettings) -> Figure),
}

/// A figure's value: a count or flag as the game keeps it (shown signed), or a float.
#[derive(Clone, Copy, PartialEq)]
enum Figure {
    Whole(u32),
    Real(f32),
}

impl From<u32> for Figure {
    fn from(v: u32) -> Self {
        Figure::Whole(v)
    }
}

impl From<f32> for Figure {
    fn from(v: f32) -> Self {
        Figure::Real(v)
    }
}

/// A row: the developers' label and its place.
type Row = (&'static str, At);

use At::{Cell, Through};

/// The window's width for two columns.
const TWO_COLUMNS: f32 = 900.0;

const GROUPS: &[(&str, &[Row])] = &[
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
                        .into()
                }),
            ),
            (
                "?rain.total",
                Cell(|o| o.effects.windscreen_drops.state.live.get().into()),
            ),
            (
                "?snow3d.drawn",
                Cell(|o| o.effects.weather.snow.drawn.get().into()),
            ),
            (
                "splash amount",
                Cell(|o| o.effects.ring.work.count.get().into()),
            ),
            (
                "fworks amount",
                Cell(|o| o.effects.ring.pool.used.get().into()),
            ),
            (
                "fworks run",
                Cell(|o| o.effects.ring.pool.fireworks_run.get().into()),
            ),
            (
                "fsizefp",
                Cell(|o| o.scene.drawing.scene.get().view_scale.get().into()),
            ),
            (
                "maxzspacefp",
                Cell(|o| o.scene.drawing.scene.get().farthest.get().into()),
            ),
        ],
    ),
    (
        "The car",
        &[
            (
                "change_gear",
                Through(|s| s.spline_gear_changed.get().into()),
            ),
            ("hand_brake", Through(|s| s.spline_handbrake.get().into())),
            ("backfire", Through(|s| s.cars[0].gear_notice.get().into())),
            (
                "car angle",
                Cell(|o| o.effects.frame.view_vars.heading.get().into()),
            ),
            (
                "car direction of travel",
                Cell(|o| o.effects.frame.view_vars.facing.get().into()),
            ),
            ("car x", Through(|s| s.listener_car_place[0].get().into())),
            ("car y", Through(|s| s.listener_car_place[1].get().into())),
            ("car z", Through(|s| s.listener_car_place[2].get().into())),
            (
                "?wheels.vect1.x",
                Cell(|o| o.effects.kinds.emitter.side_b[0].get().into()),
            ),
            (
                "?wheels.vect1.y",
                Cell(|o| o.effects.kinds.emitter.side_b[1].get().into()),
            ),
            (
                "?wheels.vect1.z",
                Cell(|o| o.effects.kinds.emitter.side_b[2].get().into()),
            ),
            (
                "?wheels.vect4.x",
                Cell(|o| o.effects.kinds.emitter.side_a[0].get().into()),
            ),
            (
                "?wheels.vect4.y",
                Cell(|o| o.effects.kinds.emitter.side_a[1].get().into()),
            ),
            (
                "?wheels.vect4.z",
                Cell(|o| o.effects.kinds.emitter.side_a[2].get().into()),
            ),
        ],
    ),
    (
        "Clocks",
        &[
            (
                "?world.time_abs",
                Cell(|o| o.effects.setup.clock.now.get().into()),
            ),
            (
                "?world.time",
                Cell(|o| o.effects.setup.clock.frame_ticks.get().into()),
            ),
            (
                "clock0",
                Cell(|o| o.effects.setup.frame.empty_share.get().into()),
            ),
            (
                "clock1",
                Cell(|o| o.effects.setup.frame.ring_share.get().into()),
            ),
            (
                "clock2",
                Cell(|o| o.effects.setup.frame.whole_share.get().into()),
            ),
            (
                "clock3",
                Cell(|o| o.effects.setup.frame.quads_share.get().into()),
            ),
            (
                "clock4",
                Cell(|o| o.effects.setup.frame.step_share.get().into()),
            ),
            (
                "clock5",
                Cell(|o| o.effects.setup.frame.clock5.get().into()),
            ),
            (
                "clock6",
                Cell(|o| o.effects.setup.frame.clock6.get().into()),
            ),
            (
                "clock7",
                Cell(|o| o.effects.setup.frame.draw_share.get().into()),
            ),
        ],
    ),
    (
        "Windows",
        &[
            ("window 0", Through(|s| s.cars[0].face_sums[0].get().into())),
            ("window 1", Through(|s| s.cars[0].face_sums[1].get().into())),
            ("window 2", Through(|s| s.cars[0].face_sums[2].get().into())),
            ("window 3", Through(|s| s.cars[0].face_sums[3].get().into())),
            ("window 4", Through(|s| s.cars[0].face_sums[4].get().into())),
            ("window 5", Through(|s| s.cars[0].face_sums[5].get().into())),
        ],
    ),
];

/// The three sets of six cracks: the first car's line sums, shares and point counts.
const CRACK_SETS: [fn(&EffectsSettings, usize) -> u32; 3] = [
    |s, n| s.cars[0].kind_sums[n].get(),
    |s, n| s.cars[0].kind_shares[n].get(),
    |s, n| s.cars[0].kind_counts[n].get(),
];

fn read(o: &Owners<'static>, at: At) -> Option<Figure> {
    match at {
        Cell(read) => Some(read(o)),
        Through(read) => effects_settings(o.effects).ok().map(read),
    }
}

fn shown(value: Option<Figure>) -> String {
    match value {
        None => "?".to_owned(),
        Some(Figure::Real(v)) => format!("{v:.3}"),
        Some(Figure::Whole(v)) => format!("{}", v as i32),
    }
}

/// The tab.
pub fn tab(o: &Owners<'static>, ui: &Ui) {
    // Its warning, when the two counts differ.
    let frame = &o.effects.frame.draw_frame;
    let (drawn, run) = (
        Some(frame.drawn.get().into()),
        Some(frame.cleared.get().into()),
    );
    if drawn != run {
        ui.text_colored(
            [1.0, 0.4, 0.3, 1.0],
            format!(
                "draw amount not same as process amount: {} / {}",
                shown(drawn),
                shown(run)
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
fn group(ui: &Ui, o: &Owners<'static>, (name, figures): &(&str, &[Row])) {
    if ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN)
        && let Some(_table) = ui.begin_table(name, 2)
    {
        for &(label, at) in *figures {
            ui.table_next_row();
            ui.table_next_column();
            ui.text_disabled(label);
            ui.table_next_column();
            ui.text(shown(read(o, at)));
        }
    }
}

/// The three sets of cracks.
fn cracks(ui: &Ui, o: &Owners<'static>) {
    if ui.collapsing_header("Cracks", TreeNodeFlags::DEFAULT_OPEN) {
        ui.text(format!(
            "?crack.flag: {}",
            shown(Some(o.effects.paths.lines_work.any.get().into()))
        ));
        for (set, figure) in CRACK_SETS.iter().enumerate() {
            let figures: Vec<String> = (0..6)
                .map(|n| {
                    shown(
                        effects_settings(o.effects)
                            .ok()
                            .map(|s| figure(s, n).into()),
                    )
                })
                .collect();
            ui.text(format!("Set {}: {}", set + 1, figures.join("  ")));
        }
    }
}
