//! The itinerary pages (`sub_472C8C` and its sisters; stream A, part F1, 2026-10-18): a
//! section of the record table (`g_recordTable`, records 0xA4 bytes; +0 the tag: 0 a stage,
//! 1 a service, 3 / 4 the end; +0x10 its time of day as hhmm) listed row by row from y 0xE8
//! (`seconds`), the rows counted in `dword_7559B0`, the stages in `dword_7559B4`, the
//! services in `dword_755A0C`.

use crate::errors::{FatalError, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::screens::best_scores::{record_at, section_there};
use crate::screens::{font_at, line_buffer};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

fn w(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}

const INVALID_STAGE: FatalError = FatalError {
    code: 0x8070_D91A,
    message: "Invalid Stage ID",
};
const Y: usize = 0x7559AC; // seconds
const ROW: usize = 0x7559B0;
const STAGE: usize = 0x7559B4;
const SERVICE: usize = 0x755A0C;

unsafe fn set_line(bytes: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
}

/// A row's time of day ("09:05", `strings::append_clock_padded` of the minutes) drawn in fixed
/// columns ending at x 0x2C2.
unsafe fn clock(record: u32, font: usize, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let hhmm = ((record + 0x10) as *const u32).read_unaligned();
        let mut line = crate::strings::append_clock_padded::append_clock_padded(
            crate::math::hhmm_to_minutes::hhmm_to_minutes(hhmm),
        );
        line.push(0);
        set_line(&line);
        crate::text::draw_clock_fixed_pitch::draw_clock_fixed_pitch_at(
            font_at(font),
            line_buffer(),
            0x2C2,
            k(Y) as i32,
            canvas,
        )?;
        Ok(())
    }
}

/// `screens::draw_itinerary` (0x472C8C): the section's heading (event mode 0x13
/// `dword_75A28A[section]`, else its first record's +0x38 string) at (0x2EB, 0x77), the column
/// heads (strings 0x482, 0x47E, 0x480, 0x47F at y 0xD1), then from record 1: a service its
/// name (string 0x478 + its number, kept in `dword_755A08`) and time; a stage (the row
/// `index` in the highlight fonts) its track-pack entry's name, its length (hundredths of a
/// mile, `strings::append_distance_2dp` and the unit's string 0x56F / 0x570; none for 0), its
/// number, and its time; tags 3 and 4 end the list, any other is fatal error 0x8070D91A
/// "Invalid Stage ID". Every register kept. Refuses (before anything) a section past the
/// table.
pub unsafe fn draw_itinerary(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        if !section_there() {
            return Err(crate::errors::BadInput("a section past the record table").into());
        }
        let plain = 0x51E584;
        let first = record_at(0);
        let id = if k(0x7556D0) == 0x13 {
            ((game_addr(rva(0x75A28A)) as u32 + 4 * k(0x75595C)) as *const u32).read_unaligned()
        } else {
            ((first + 0x38) as *const u32).read_unaligned()
        };
        draw_string_by_id_at(id, font_at(0x51E9A4), 0x2EB, 0x77, canvas)?;
        for (id, x) in [
            (0x482u32, 0xBA),
            (0x47E, 0x10A),
            (0x480, 0x1F1),
            (0x47F, 0x257),
        ] {
            draw_string_by_id_at(id, font_at(plain), x, 0xD1, canvas)?;
        }
        w(Y, 0xE8);
        w(ROW, 1);
        w(STAGE, 1);
        w(SERVICE, 0);
        let mut record = record_at(1);
        loop {
            let y = k(Y) as i32;
            match (record as *const u32).read_unaligned() {
                1 => {
                    let id = 0x478 + k(SERVICE);
                    w(0x755A08, id);
                    draw_string_by_id_at(id, font_at(plain), 0x10A, y, canvas)?;
                    w(SERVICE, k(SERVICE).wrapping_add(1));
                    clock(record, 0x51E564, canvas)?;
                }
                3 | 4 => return Ok(()),
                0 => {
                    let entry = crate::records::trakpak_entry::record_entry(record as *const u8)?;
                    let marked = k(0x755958) == k(ROW);
                    let font = if marked { 0x51E524 } else { plain };
                    draw_string_by_id_at(
                        (entry as *const u32).read_unaligned(),
                        font_at(font),
                        0x10A,
                        y,
                        canvas,
                    )?;
                    let length = ((entry + 0xC) as *const u32).read_unaligned();
                    // No length: two NULs (the original's `append_nul` on both ways).
                    let mut line = if length == 0 { vec![0u8] } else { Vec::new() };
                    if length != 0 {
                        let metric = crate::strings::units::units() != 0;
                        line.extend(crate::strings::append_distance_2dp::append_distance_2dp(
                            length, metric,
                        )?);
                        crate::strings::append_string_by_id::push_string(
                            &mut line,
                            if metric { 0x570 } else { 0x56F },
                        );
                    }
                    line.push(0);
                    set_line(&line);
                    draw_text_at(font_at(font), line_buffer(), 0x1F1, y, canvas)?;
                    let mut n = crate::strings::append_u32::append_u32(k(STAGE));
                    n.push(0);
                    set_line(&n);
                    draw_text_at(font_at(font), line_buffer(), 0xBA, y, canvas)?;
                    w(STAGE, k(STAGE).wrapping_add(1));
                    clock(record, if marked { 0x51E504 } else { 0x51E564 }, canvas)?;
                }
                _ => return Err(INVALID_STAGE.into()),
            }
            w(Y, k(Y).wrapping_add(0x19));
            w(ROW, k(ROW).wrapping_add(1));
            record = record.wrapping_add(0xA4);
            if (record as *const u32).read_unaligned() == u32::MAX {
                return Ok(());
            }
        }
    }
}
