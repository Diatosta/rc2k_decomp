//! The time trial stage results page (`sub_48E740`; stream A, part F1, 2026-10-18).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::text::draw_text::Canvas;

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

fn w(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The stage's record table and record (`off_735512[dword_7556F8][dword_75880E]`, + 0xA4 x
/// `dword_758812`), none: 0.
fn records() -> (u32, u32) {
    unsafe {
        let table =
            ((game_addr(rva(0x735512)) as u32 + 4 * k(0x7556F8)) as *const u32).read_unaligned();
        if table == 0 {
            return (0, 0);
        }
        let base = get(table + 4 * k(0x75880E));
        (base, base.wrapping_add(k(0x758812).wrapping_mul(0xA4)))
    }
}

/// The times table: `off_7354FE[dword_7556F8 (+ 2 when dword_7556FC is 1)]` points at the
/// pointer to it.
fn times_table() -> u32 {
    unsafe {
        let mut at = game_addr(rva(0x7354FE)) as u32 + 4 * k(0x7556F8);
        if k(0x7556FC) == 1 {
            at += 8;
        }
        let p = get(at);
        if p == 0 { 0 } else { get(p) }
    }
}

/// `screens::draw_stage_results` (0x48E740): the time trial stage's results: the current
/// menu's (`dword_75558D`) items 1 .. 3 (strings by the two switches, the record's +0x38), the
/// record's +4 into `dword_A8C154`, the stage line (string 0x481 .. in `byte_A8BD54`), then up
/// to ten rows of name, result and rank with the car's and flag's sprites: in seal mode
/// (`dword_75AB30` 1) the row's seal code (`strings::format_seal_code` of +0x54 / +0x58) one
/// character at a time from x 0x20A every 0xE; else its time (fixed pitch at 0x276), the first
/// kept in `dword_75AB24` and the others' gap to it (string 0x579 and the time at 0x2EE); after
/// ten rows the fastest line (string 0x1A2; from the stage record found again by
/// `records::find_stage_record` when both switches are 0, else the record's own, its second
/// fastest with `dword_7556FC` 1). The sprites' palettes are ebx as the original leaves it.
/// Every register kept. Refuses (before anything) no current menu, no stage records or times
/// table, a stage line past its buffer, a times table whose end (-1) comes within the ten rows
/// (the original's early exit leaves two dwords on the stack and returns through a `pusha`
/// slot), and what the strings refuse.
pub unsafe fn draw_stage_results(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let menu = k(0x75558D);
        let (base, record) = records();
        if menu == 0 || base == 0 || times_table() == 0 {
            return Err(BadInput("no current menu, stage records or times table").into());
        }
        let both_off = (k(0x7556F8) ^ 1) & (k(0x7556FC) ^ 1);
        let fastest = if both_off == 1 {
            crate::records::find_stage_record::find_stage_record(
                game_addr(rva(0x735576)) as u32,
                k(0x75880E),
                k(0x758812),
            )?
        } else {
            record
        };
        let second = both_off != 1 && k(0x7556FC) == 1;
        let speed_at = if second { 0x2C } else { 0x30 };
        let mut speed = crate::strings::append_speed::append_speed(
            get(fastest + speed_at),
            true,
            crate::strings::units::units(),
        )?;
        speed.push(0);
        let stage = crate::screens::page_settings::stage_line_bytes(0x481, record)?;
        if stage.len() > 0x100 {
            return Err(BadInput("a stage line past its buffer").into());
        }
        let first = times_table().wrapping_add(get(record + 0x3C).wrapping_mul(0x618));
        if (1..=10).any(|n| get(first.wrapping_add(0x9C * n)) == u32::MAX) {
            return Err(BadInput(
                "a times table ending within the ten rows: the original returns through a wrong stack",
            )
            .into());
        }
        let set_item =
            |n: u32, v: u32| ((menu + 4 + 0x5C * n + 0x14) as *mut u32).write_unaligned(v);
        set_item(1, if k(0x7556F8) == 1 { 0x19C } else { 0x19B });
        set_item(2, if k(0x7556FC) == 1 { 0x19E } else { 0x19D });
        set_item(3, get(base + 0x38));
        w(0xA8C154, get(record + 4));
        std::ptr::copy_nonoverlapping(
            stage.as_ptr(),
            game_addr(rva(0xA8BD54)) as *mut u8,
            stage.len(),
        );
        // Converted: `rc2k_core::screens::stage_results`.
        use rc2k_core::screens::stage_results::{Entry, Fastest, stage_results};
        let rows = std::array::from_fn(|n| {
            let entry = first.wrapping_add(0x9C * n as u32);
            let f = |o: u32| get(entry + o);
            Entry {
                name: entry + 0x20,
                nation: f(8),
                car: f(0x10),
                time: f(0x14),
                seal: (f(0x54), f(0x58)),
            }
        });
        let (nation, name, car) = if second {
            (0x9C, 0x7C, 0xA0)
        } else {
            (0x74, 0x54, 0x78)
        };
        speed.pop();
        let fastest = Fastest {
            nation: get(fastest + nation),
            name: fastest + name,
            car: get(fastest + car),
            palette: if both_off == 1 { k(0x75880E) } else { both_off },
            speed,
        };
        let mut seal_code = |a: u32, b: u32| {
            crate::strings::format_seal_code::format_seal_code(a, b);
            let code = game_addr(rva(0x4E0544)) as *const u8;
            std::ffi::CStr::from_ptr(code as *const std::ffi::c_char)
                .to_bytes()
                .to_vec()
        };
        stage_results(
            &mut crate::screens::game_screen::GameScreen::new(canvas),
            k(0x75AB30) == 1,
            crate::strings::append_time::show_hours(),
            &rows,
            &mut seal_code,
            &fastest,
        )?;
        Ok(())
    }
}

/// Whether the page has what it reads (for the test): the checks `draw_stage_results` refuses
/// on, run without drawing.
pub unsafe fn stage_results_ready() -> bool {
    unsafe {
        let (base, record) = records();
        if k(0x75558D) == 0 || base == 0 || times_table() == 0 {
            return false;
        }
        if crate::screens::page_settings::stage_line_bytes(0x481, record).is_err() {
            return false;
        }
        let first = times_table().wrapping_add(get(record + 0x3C).wrapping_mul(0x618));
        if (1..=10).any(|n| get(first.wrapping_add(0x9C * n)) == u32::MAX) {
            return false;
        }
        let both_off = (k(0x7556F8) ^ 1) & (k(0x7556FC) ^ 1);
        both_off != 1
            || crate::records::find_stage_record::find_stage_record(
                game_addr(rva(0x735576)) as u32,
                k(0x75880E),
                k(0x758812),
            )
            .is_ok()
    }
}
