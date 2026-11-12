//! The "Players" tab: the developers' two data dumps (no references to either).
//! 0x471412 wrote each player's record to the game's log under the labels below (the
//! record's fields, named by them); 0x449A59 a car block's state, of "car1" and "car2", which
//! are both the first block (a slip: `notes/bugs.md`; here every car). The button
//! writes the same to the log.

use crate::owners::Owners;
use crate::physics::car_store::Car;
use crate::session::Dword;
use dear_imgui_rs::{TableFlags, Ui};

/// 0x471412's labels and offsets, in its order.
const PLAYER_ROWS: [(&str, usize); 51] = [
    ("zxplayer", 0x0),
    ("country", 0x4),
    ("carnumber", 0x8),
    ("viewpoint", 0xC),
    ("viewpoint2", 0x10),
    ("CarSideNum", 0x28),
    ("CurrentPoints", 0x80),
    ("TotalPoints", 0x84),
    ("CurrentTime", 0x88),
    ("TotalTime", 0x8C),
    ("Group", 0x90),
    ("TopSpeed", 0x94),
    ("CR1Score", 0x98),
    ("CR2Score", 0x9C),
    ("CR3Score", 0xA0),
    ("CR4Score", 0xA4),
    ("CR5Score", 0xA8),
    ("CR6Score", 0xAC),
    ("CRScoreTotal", 0xB0),
    ("R1Score", 0xB4),
    ("R2Score", 0xB8),
    ("R3Score", 0xBC),
    ("R4Score", 0xC0),
    ("R5Score", 0xC4),
    ("R6Score", 0xC8),
    ("RScoreTotal", 0xCC),
    ("Active", 0xD0),
    ("Engine", 0xD4),
    ("Bodywork", 0xD8),
    ("Exhaust", 0xDC),
    ("Electrics", 0xE0),
    ("Turbo", 0xE4),
    ("Lights", 0xE8),
    ("Cooling", 0xEC),
    ("Steering", 0xF0),
    ("Clutch", 0xF4),
    ("Suspension", 0xF8),
    ("Differential", 0xFC),
    ("Brakes", 0x100),
    ("Gearbox", 0x104),
    ("TyresWear", 0x108),
    ("TyresWearType", 0x10C),
    ("BrakeBalance", 0x110),
    ("SuspHeight", 0x114),
    ("SuspStiffness", 0x118),
    ("SteerSense", 0x11C),
    ("GearboxType", 0x120),
    ("LastChampPosC", 0x144),
    ("LastChampPosA", 0x148),
    ("LastLBoardPosC", 0x14C),
    ("LastLBoardPosA", 0x150),
];

/// A car block's cell.
type CarField = fn(&Car) -> &Dword;

/// 0x449AA3's labels and fields (it shows `racefinished` twice).
const CAR_ROWS: [(&str, CarField); 7] = [
    ("driver", |c| &c.driver_kind),
    ("racefinished", |c| &c.finish_flag),
    ("currsegment", |c| &c.splits_passed),
    ("nextcheckpoint", |c| &c.splits_recorded),
    ("timetaken", |c| &c.stage_time),
    ("outoftime", |c| &c.go_stand_timer),
    ("gameoverflag", |c| &c.stop_state),
];

fn figure(word: u32) -> String {
    format!("{}", word as i32)
}

fn player_figure(o: &Owners<'static>, player: usize, offset: usize) -> String {
    figure(o.menus.play.players.players[player].get().word(offset))
}

fn car_figure(o: &Owners<'static>, car: usize, field: CarField) -> String {
    figure(field(&o.race.car_blocks().cars[car]).get())
}

/// The player's name, to its NUL.
fn name(o: &Owners<'static>, player: usize) -> String {
    let record = o.menus.play.players.players[player].get();
    let bytes: Vec<u8> = record
        .name_copy
        .iter()
        .copied()
        .take_while(|&b| b != 0)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Both dumps to the log.
fn write_log(o: &Owners<'static>) {
    use crate::framework::log;
    for player in 0..4 {
        log(&format!("DATA DUMP OF PLAYER {}", player + 1));
        for (label, offset) in PLAYER_ROWS {
            log(&format!("{label}: {}", player_figure(o, player, offset)));
        }
        log(&format!("NameStore: {}", name(o, player)));
    }
    log(&format!(
        "shaungmode: {}",
        figure(o.race.shared.vars.run_kind.get())
    ));
    for car in 0..10 {
        log(&format!("car {}", car + 1));
        for (label, field) in CAR_ROWS {
            log(&format!("{label}: {}", car_figure(o, car, field)));
        }
    }
}

/// A table of a label and a figure per column.
fn table(
    ui: &Ui,
    id: &str,
    columns: usize,
    label: &str,
    rows: &mut dyn Iterator<Item = (String, Vec<String>)>,
) {
    let flags = TableFlags::BORDERS | TableFlags::ROW_BG;
    let Some(_table) = ui.begin_table_with_flags(id, columns + 1, flags) else {
        return;
    };
    ui.table_next_row();
    ui.table_next_column();
    ui.text_disabled(label);
    for n in 0..columns {
        ui.table_next_column();
        ui.text_disabled(format!("{}", n + 1));
    }
    for (name, values) in rows {
        ui.table_next_row();
        ui.table_next_column();
        ui.text(name);
        for value in values {
            ui.table_next_column();
            ui.text(value);
        }
    }
}

/// The tab.
pub fn tab(o: &Owners<'static>, ui: &Ui) {
    if ui.button("Write both dumps to the log") {
        write_log(o);
    }
    ui.same_line();
    ui.text(format!(
        "shaungmode (run kind): {}",
        figure(o.race.shared.vars.run_kind.get())
    ));
    if ui.collapsing_header(
        "Cars (the race dump)",
        dear_imgui_rs::TreeNodeFlags::DEFAULT_OPEN,
    ) {
        table(
            ui,
            "cars",
            10,
            "car",
            &mut CAR_ROWS.iter().map(|&(label, field)| {
                (
                    label.to_owned(),
                    (0..10).map(|c| car_figure(o, c, field)).collect(),
                )
            }),
        );
    }
    if ui.collapsing_header(
        "Players (the data dump)",
        dear_imgui_rs::TreeNodeFlags::DEFAULT_OPEN,
    ) {
        table(
            ui,
            "players",
            4,
            "player",
            &mut std::iter::once(("NameStore".to_owned(), (0..4).map(|p| name(o, p)).collect()))
                .chain(PLAYER_ROWS.iter().map(|&(label, offset)| {
                    (
                        label.to_owned(),
                        (0..4).map(|p| player_figure(o, p, offset)).collect(),
                    )
                })),
        );
    }
}
