//! The "Effects" tab: the effects' developers' page (`debug::effects_page_1`, 0x4D1711, no
//! references to it; IDA 2026-10-30), its labels kept. Some figures lie through
//! `g_marksSettings` (0x767A28, now a set / not-set flag), the rest at their own places. Labels with `?` were
//! the developers' own, for the names in their source.

use crate::debug::wear_page::value_at;
use dear_imgui_rs::{TreeNodeFlags, Ui};

/// `g_marksSettings`.
const SETTINGS: usize = 0x767A28;

/// The windows and cracks lie from here on (`lea ebp, [ebp+29Ch]`, 0x4D238F).
const WORK: u32 = 0x29C;

#[derive(Clone, Copy)]
enum At {
    /// IDA's address.
    Place(usize),
    /// An offset in the settings `g_marksSettings` stands for (the race's record).
    Through(u32),
}

/// A figure: the developers' label, its place, whether a float.
type Figure = (&'static str, At, bool);

use At::{Place, Through};

/// The window's width for two columns.
const TWO_COLUMNS: f32 = 900.0;

const GROUPS: &[(&str, &[Figure])] = &[
    (
        "Weather and splashes",
        &[
            ("?rain.hang", Place(0x81623A), false),
            ("?rain.total", Place(0x814710), false),
            ("?snow3d.drawn", Place(0x8D219C), false),
            ("splash amount", Place(0x9356C8), false),
            ("fworks amount", Place(0x93A5EC), false),
            ("fworks run", Place(0x93A5CC), false),
            ("flag", Place(0x91FF32), false),
            ("First object in list", Place(0x8DD059), false),
            ("fsizefp", Place(0x624088), true),
            ("maxzspacefp", Place(0x6241C8), true),
        ],
    ),
    (
        "The car",
        &[
            ("change_gear", Through(0x26FC), false),
            ("hand_brake", Through(0x2700), false),
            ("backfire", Through(0x8C), false),
            ("car angle", Place(0x8F3975), true),
            ("car direction of travel", Place(0x8F397D), true),
            ("car x", Through(0x26AC), true),
            ("car y", Through(0x26B0), true),
            ("car z", Through(0x26B4), true),
            ("?wheels.vect1.x", Place(0x93543A), true),
            ("?wheels.vect1.y", Place(0x93543E), true),
            ("?wheels.vect1.z", Place(0x935442), true),
            ("?wheels.vect4.x", Place(0x93545E), true),
            ("?wheels.vect4.y", Place(0x935462), true),
            ("?wheels.vect4.z", Place(0x935466), true),
        ],
    ),
    (
        "Clocks",
        &[
            ("?world.time_abs", Place(0x764FE8), false),
            ("?world.time", Place(0x764FD0), false),
            ("clock0", Place(0x765118), true),
            ("clock1", Place(0x76511C), true),
            ("clock2", Place(0x765120), true),
            ("clock3", Place(0x765124), true),
            ("clock4", Place(0x765128), true),
            ("clock5", Place(0x76512C), true),
            ("clock6", Place(0x765130), true),
            ("clock7", Place(0x765134), true),
        ],
    ),
    (
        "Windows",
        &[
            ("window 0", Through(WORK + 0x1C), false),
            ("window 1", Through(WORK + 0x20), false),
            ("window 2", Through(WORK + 0x24), false),
            ("window 3", Through(WORK + 0x28), false),
            ("window 4", Through(WORK + 0x2C), false),
            ("window 5", Through(WORK + 0x30), false),
        ],
    ),
];

/// The three sets of six cracks' offsets (the labels repeat in the original).
const CRACK_SETS: [u32; 3] = [0x1B8, 0x1FC, 0x240];
/// `g_marksAnyMeasured` ("?crack.flag").
const CRACK_FLAG: usize = 0x93C8BC;

fn read(at: At) -> Option<u32> {
    match at {
        Place(ida) => value_at(ida),
        Through(offset) => {
            if value_at(SETTINGS)? == 0 {
                return None;
            }
            crate::race::marks_settings_store::MARKS_SETTINGS
                .at(offset)
                .read::<u32>("the effects' page")
                .ok()
        }
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
pub fn tab(ui: &Ui) {
    // 0x4D2B90: its warning, when the two counts differ.
    let (drawn, run) = (value_at(0x91FFA2), value_at(0x91FFBA));
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
    // Two columns when there is room (owner, 2026-10-30).
    if ui.content_region_avail_width() >= TWO_COLUMNS
        && let Some(_layout) = ui.begin_table("effects", 2)
    {
        ui.table_next_row();
        ui.table_next_column();
        group(ui, &GROUPS[0]);
        group(ui, &GROUPS[2]);
        ui.table_next_column();
        group(ui, &GROUPS[1]);
        group(ui, &GROUPS[3]);
        cracks(ui);
    } else {
        for g in GROUPS {
            group(ui, g);
        }
        cracks(ui);
    }
}

/// A group of figures, under its header.
fn group(ui: &Ui, (name, figures): &(&str, &[Figure])) {
    if ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN)
        && let Some(_table) = ui.begin_table(name, 2)
    {
        for &(label, at, float) in *figures {
            ui.table_next_row();
            ui.table_next_column();
            ui.text_disabled(label);
            ui.table_next_column();
            ui.text(shown(read(at), float));
        }
    }
}

/// The three sets of cracks.
fn cracks(ui: &Ui) {
    if ui.collapsing_header("Cracks", TreeNodeFlags::DEFAULT_OPEN) {
        ui.text(format!(
            "?crack.flag: {}",
            shown(value_at(CRACK_FLAG), false)
        ));
        for (set, base) in CRACK_SETS.iter().enumerate() {
            let figures: Vec<String> = (0..6)
                .map(|n| shown(read(Through(WORK + base + 4 * n)), false))
                .collect();
            ui.text(format!("Set {}: {}", set + 1, figures.join("  ")));
        }
    }
}
