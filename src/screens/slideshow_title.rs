//! `screens::slideshow_title` (0x469A57): the slideshow's title.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! String 0x33D and a NUL into `byte_A8BF54`. In two draw lists (the slideshow).
//! Refuses a title longer than 0x100 bytes.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::screens::car_caption::put_string;

const TITLE_RVA: usize = rva(0xA8BF54); // byte_A8BF54
const TITLE_LEN: usize = 0x100;
const TITLE: u32 = 0x33D;

// ---- Rust version ----

pub unsafe fn slideshow_title() -> Result<(), Refused> {
    unsafe { put_string(TITLE, TITLE_RVA, TITLE_LEN) }
}
