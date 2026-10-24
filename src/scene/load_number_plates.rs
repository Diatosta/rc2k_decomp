//! `scene::load_number_plates` (0x4255AA): the two number plates' textures.
//!
//! In:  nothing
//! Out: kind 0xC1 has no first texture: ebx = -1, the rest preserved; else eax, ecx =
//!      the second plate's two picture names, edx, esi its two numbers, edi = 0xAD,
//!      ebp = 0x94, ebx = the texture
//!
//! Unless kind 0xC1's first texture (`dword_5D0DEC`) is none, twice, for the player's
//! car (`dword_70F410`, or with `dword_52A910` set the block of the car
//! `dword_52A920`, its +0x24) and then for `dword_7103F0`:
//! - `scene::known_kind_plate` of it gives the plate's number;
//! - under 0x12 the pictures `files\b\tx1822.pcx` and `tx1823.pcx`, else `tx1824.pcx`
//!   and `tx1825.pcx` (eax, ecx);
//! - edx, esi = `dword_5DCD9C[number]`, `dword_5DCE2C[number]`, edi = 0xAD, ebp = 0x68
//!   for the first and 0x94 for the second, ebx = the texture;
//! - `scene::upload_plate` (`pusha` .. `popa`).
//!
//! One caller, `scene::reload_system_textures`.
//! Refuses a number of 0x24 or more (the tables' room) and a car without its block.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose upload notes its registers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::scene::known_kind_plate::known_kind_plate;
use crate::scene::upload_plate::{Plate, upload_plate};

// ---- game data ----

const TEXTURE_RVA: usize = rva(0x5D0DEC); // dword_5D0DEC: kind 0xC1's first texture
const PLAYER_CAR_RVA: usize = rva(0x70F410); // dword_70F410
const OTHER_CAR_RVA: usize = rva(0x7103F0); // dword_7103F0
const ONE_CAR_RVA: usize = rva(0x52A910); // dword_52A910
const WHICH_CAR_RVA: usize = rva(0x52A920); // dword_52A920
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const FIRSTS_RVA: usize = rva(0x5DCD9C); // dword_5DCD9C
const SECONDS_RVA: usize = rva(0x5DCE2C); // dword_5DCE2C
// scene::upload_plate
/// `files\b\tx1822.pcx` .. `tx1825.pcx`.
const NAMES: [usize; 4] = [rva(0x60AEB7), rva(0x60AECA), rva(0x60AEDD), rva(0x60AEF0)];

// ---- Rust version ----

/// The Rust version. The registers the original leaves (the last plate's, or ebx -1 when
/// kind 0xC1 has no texture) are read by no caller (taint at its one site, in
/// `reload_system_textures`: 0 read points).
// Converted: `rc2k_core::scene::plates`
pub unsafe fn load_number_plates() -> Result<(), Refused> {
    use rc2k_core::scene::plates;
    unsafe {
        let texture: u32 = game_read(TEXTURE_RVA);
        if texture == u32::MAX {
            return Ok(());
        }
        let player = if game_read::<u32>(ONE_CAR_RVA) != 0 {
            let car = plates::one_car(game_read(WHICH_CAR_RVA))?;
            let block: u32 = game_read(BLOCKS_RVA + car * 4);
            if block == 0 {
                return Err(BadInput("a car without its block").into());
            }
            ((block + 0x24) as *const u32).read_unaligned()
        } else {
            game_read(PLAYER_CAR_RVA)
        };
        let numbers = [
            known_kind_plate(player),
            known_kind_plate(game_read(OTHER_CAR_RVA)),
        ];
        plates::check_numbers(numbers)?;
        for (number, target_y) in numbers.into_iter().zip(plates::ROWS) {
            let names = plates::pictures(number);
            upload_plate(&Plate {
                sheet: game_addr(NAMES[names]) as u32,
                alpha_sheet: game_addr(NAMES[names + 1]) as u32,
                texture,
                sheet_x: game_read(FIRSTS_RVA + number as usize * 4),
                sheet_y: game_read(SECONDS_RVA + number as usize * 4),
                target_x: plates::COLUMN,
                target_y,
            })?;
        }
        Ok(())
    }
}
