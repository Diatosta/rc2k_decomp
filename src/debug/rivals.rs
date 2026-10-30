//! The rivals' table (`debug::draw_rivals`, 0x435F43, IDA 2026-10-30), the display page's
//! last item: `physics::fill_rival_table`, then for each of the 90 rivals a row 6 lower,
//! at x 0x258, built in the shared text buffer as the original builds it: its number's two
//! digits and a space, its name (`strings::append_string_by_id`, which writes no NUL: a
//! shorter name leaves what the buffer held, and a NUL there ends the row), a space at
//! +0xC, the scaled time (`g_rivalScaled`) at +0xD, a space, the time (`g_rivalTimes`) at
//! +0x13, a NUL at +0x18; 24 characters of it printed narrow (`debug::draw::print_narrow`).

use crate::errors::Refused;
use crate::physics::split_tables_store::SPLIT_TABLES;
use rc2k_core::debug::numbers::rival_time;
use rc2k_core::physics::splits::RIVALS;

/// `debug::draw_rivals` (see the header).
pub fn draw_rivals() -> Result<(), Refused> {
    crate::physics::fill_rival_table::fill_rival_table()?;
    let text = crate::strings::text_from_ids::text_at();
    for n in 0..RIVALS {
        text.write_bytes(
            &[b'0' + (n / 10) as u8, b'0' + (n % 10) as u8, b' '],
            "a rival's row",
        )?;
        let mut name = Vec::new();
        crate::strings::append_string_by_id::append_string_by_id(
            SPLIT_TABLES.rival_names[n].get(),
            &mut |run| name.extend_from_slice(run),
        )?;
        text.offset(3).write_bytes(&name, "a rival's name")?;
        text.offset(0xC).write_bytes(b" ", "a rival's row")?;
        text.offset(0x12).write_bytes(b" ", "a rival's row")?;
        text.offset(0x18).write_bytes(&[0], "a rival's row")?;
        text.offset(0xD).write_bytes(
            &rival_time(SPLIT_TABLES.rival_scaled[n].get()),
            "a rival's time",
        )?;
        text.offset(0x13).write_bytes(
            &rival_time(SPLIT_TABLES.rival_times[n].get()),
            "a rival's time",
        )?;
        let row = text.read_bytes(0x18, "a rival's row")?;
        crate::debug::draw::print_narrow(0x258, n as u32 * 6, &row, 0x18)?;
    }
    Ok(())
}
