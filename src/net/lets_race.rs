//! `net::lets_race` (0x4640AA): the network race driven: `dword_63DB84` and the eight stored
//! cars' +0x74 (`g_carBlockStore.out`) 0, the HUD set (`net::set_hud_options`), the race
//! (`drive::play_race`), the HUD kept (`net::keep_hud_options`); then this machine's car
//! (`g_carBlocks[dword_63DBA0]`): eax its +0x88 (the race's time, it seems), ebx 0, or -1
//! when its +0xB0 is -1.
//!
//! In:  nothing
//! Out: the time (eax) and whether the car did not finish (ebx: 0 or -1); the original
//!      leaves esi the car's block and other registers as the race leaves them (read by
//!      its one caller, `net::lets_rally`, only as these two)
//! Caller: `net::lets_rally`.

use crate::errors::Refused;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;

/// The Rust version: this machine's car's time (+0x88) and whether it did not finish
/// (+0xB0 is -1: `u32::MAX`, else 0).
pub fn lets_race() -> Result<(u32, u32), Refused> {
    RACE_VIEW.after_credits.set(0);
    // The eight stored cars' +0xB0 (out).
    for car in &CAR_BLOCKS.cars[..8] {
        car.stop_state.set(0);
    }
    crate::net::set_hud_options::set_hud_options();
    unsafe { crate::drive::play_race::play_race() }?;
    crate::net::keep_hud_options::keep_hud_options();
    let block = RACE_WORK.car_block(RACE_VIEW.local_car.get() as usize);
    let get = |offset: u32| unsafe { ((block + offset) as *const u32).read_unaligned() };
    // Converted: `rc2k_core::net::record::not_finished`.
    Ok((get(0x88), rc2k_core::net::record::not_finished(get(0xB0))))
}
