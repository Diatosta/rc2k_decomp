//! `screens::car_page_engine` (0x469F12): the car specification's page 1 (see
//! `screens/car_pages.rs`).
//!
//! In:  edi = the car's record
//! Out: every register kept; the carry of the last text drawn
//! Self-test: `car_pages::test_page`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::menu::car_description::CarDescription;
use crate::screens::draw_car_figures::draw_car_page;
use crate::text::draw_text::GameCanvas;

/// The Rust version: the page drawn for the car's record `description` (edi); the carry
/// of the last text drawn (too wide).
pub unsafe fn car_page_engine(description: *mut CarDescription) -> Result<bool, Refused> {
    unsafe { draw_car_page(1, description, &mut GameCanvas) }
}
