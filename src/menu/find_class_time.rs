//! `menu::find_class_time` (0x47B4EA): the first entry (0x9C bytes each, from edi) whose
//! class (+0x1C) is eax, or the end mark's (-1) when none is: its +0x18 with ebx 1, else
//! its +0x14.
//!
//! In:  eax = the class, ebx = 1 for +0x18, edi = the list
//! Out: eax = the dword; the rest kept (edi pushed and popped)
//!
//! Caller: `screens::draw_class_times` (the first time of the class). Refuses a list without its
//! end in 0x400 entries.

use crate::errors::BadInput;
use crate::memory::blocks::Place;

pub const ENTRY_LEN: u32 = 0x9C;
const LIMIT: u32 = 0x400;

fn get(a: Place) -> Result<u32, crate::errors::BadInput> {
    a.read("a ranked entry's value")
}

/// The entry the original stops at.
pub fn class_entry(list: Place, class: u32) -> Result<Place, BadInput> {
    // Converted: `rc2k_core::menu::standings::search` (the end mark's entry when none).
    use rc2k_core::menu::standings::{Found, search};
    let at = |n: u32| list.add(n.wrapping_mul(ENTRY_LEN));
    let mut last = 0;
    let fault = crate::menu::event_flow::Fault::default();
    let found = search(
        |n| {
            last = n;
            fault.get_at(at(n))
        },
        |n| fault.get_at(at(n).add(0x1C)) == class,
        LIMIT,
    );
    fault.check()?;
    match found {
        Found::At(n) => Ok(at(n)),
        Found::End => Ok(at(last)),
        Found::NoEnd => Err(BadInput("a list without its end in 0x400 entries")),
    }
}

/// The Rust version (see the header).
pub fn find_class_time(list: Place, class: u32, second: bool) -> Result<u32, BadInput> {
    let e = class_entry(list, class)?;
    get(e.add(if second { 0x18 } else { 0x14 }))
}
