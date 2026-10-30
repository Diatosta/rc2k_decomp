//! The debug UI (owner, 2026-10-30): one ImGui window over the game, resizable, its pages
//! as tabs. "Display": the display page's figures (`debug::wear_page`) as a table, the
//! original's labels and order, its bars as bars, filtered by name; "Rivals": the rivals'
//! table the display page ended with (`g_rivalNames`, `g_rivalScaled`, `g_rivalTimes`, as
//! they are: unlike the original page this does not refill them). The race's freeze as a
//! check box.

use crate::debug::wear_page::{Item, SURFACES, value_at};
use dear_imgui_rs::{Condition, TableColumnFlags, TableFlags, Ui};
use std::sync::LazyLock;

/// How a figure is shown.
#[derive(Clone, Copy)]
enum Shown {
    Whole,
    Float,
    Hex,
    Surface,
}

/// A row of the display page: the original's label before it (or none), how it is shown,
/// its place (IDA's address) and whether a bar of it was drawn.
struct Row {
    /// The figure's place on the original's screen (800 x 600): its columns are the groups.
    place: (u32, u32),
    label: String,
    shown: Shown,
    at: usize,
    bar: bool,
}

/// The display page's rows, in the original's order.
static ROWS: LazyLock<Vec<Row>> = LazyLock::new(|| {
    let mut rows = Vec::new();
    let (mut label, mut bar) = (String::new(), None);
    for &item in crate::debug::wear_items::ITEMS.iter() {
        let (shown, at, place) = match item {
            Item::Bar(_, _, at) => {
                bar = Some(at);
                continue;
            }
            Item::Label(_, _, text) => {
                label = String::from_utf8_lossy(text).into_owned();
                continue;
            }
            Item::Int(x, y, at) => (Shown::Whole, at, (x, y)),
            Item::Fixed(x, y, at) => (Shown::Float, at, (x, y)),
            Item::Hex(x, y, at) => (Shown::Hex, at, (x, y)),
            Item::Surface(x, y, at) => (Shown::Surface, at, (x, y)),
        };
        rows.push(Row {
            place,
            label: std::mem::take(&mut label),
            shown,
            at,
            bar: bar.take() == Some(at),
        });
    }
    rows
});

/// The filter's text (kept across frames).
static FILTER: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// The UI's frame.
pub fn build(ui: &Ui) {
    ui.window("Debug")
        .size([720.0, 520.0], Condition::FirstUseEver)
        .position([40.0, 40.0], Condition::FirstUseEver)
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
                if let Some(_tab) = ui.tab_item("Rivals") {
                    rivals(ui);
                }
            }
        });
}

/// The "Display" tab.
fn display(ui: &Ui) {
    let mut filter = FILTER.lock().unwrap_or_else(|e| e.into_inner());
    ui.input_text("Filter", &mut filter).build();
    let wanted = filter.to_lowercase();
    let flags =
        TableFlags::BORDERS | TableFlags::ROW_BG | TableFlags::RESIZABLE | TableFlags::SCROLL_Y;
    let Some(_table) = ui.begin_table_with_flags("display", 4, flags) else {
        return;
    };
    ui.table_setup_scroll_freeze(0, 1);
    ui.table_setup_column("Name", TableColumnFlags::NONE, None);
    ui.table_setup_column("Value", TableColumnFlags::NONE, None);
    ui.table_setup_column("Bar", TableColumnFlags::NONE, None);
    ui.table_setup_column("Place", TableColumnFlags::NONE, None);
    ui.table_headers_row();
    let mut group = None;
    for row in ROWS.iter() {
        if !wanted.is_empty() && !row.label.to_lowercase().contains(&wanted) {
            continue;
        }
        // A group's heading where the original's column changes.
        if group != Some(row.place.0) {
            group = Some(row.place.0);
            ui.table_next_row();
            ui.table_next_column();
            ui.text_disabled(format!("column at x {}", row.place.0));
        }
        let value = value_at(row.at);
        ui.table_next_row();
        ui.table_next_column();
        ui.text(&row.label);
        ui.table_next_column();
        ui.text(match (value, row.shown) {
            (None, _) => "?".to_owned(),
            (Some(v), Shown::Whole) => format!("{}", v as i32),
            (Some(v), Shown::Float) => format!("{:.3}", f32::from_bits(v)),
            (Some(v), Shown::Hex) => format!("{v:08x}"),
            (Some(v), Shown::Surface) => {
                String::from_utf8_lossy(SURFACES[(v & 0xF) as usize]).into_owned()
            }
        });
        ui.table_next_column();
        if row.bar {
            let v = value.map_or(0.0, f32::from_bits);
            let _id = ui.push_id(row.at as i32);
            ui.progress_bar(v / 100.0)
                .size([-1.0, 0.0])
                .overlay_text("")
                .build();
        }
        ui.table_next_column();
        ui.text_disabled(format!(
            "{:#08x} ({}, {})",
            row.at, row.place.0, row.place.1
        ));
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
