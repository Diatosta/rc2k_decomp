//! `scene::set_object_light` (0x45143A): the light that falls on the object as a whole,
//! to be added to each of its vertices.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx and esi changed where it gathers the lamps;
//!      its caller goes on without them, and the Rust version keeps them
//!
//! `dword_635068` (what `scene::vertex_light` adds) = 0, then by `g_lightKind`:
//! - 5: plain white;
//! - 1: `g_lightStrength` = 0; the lamps gathered; `g_lightColour` plus
//!   `dword_635054`;
//! - 2: the same, with `g_lightStrength` = 0 only while the key `byte_4F6B74` is down;
//! - 3: the same without touching the strength, and with `dword_639968` =
//!   `dword_60627C` while the lamps are gathered (a car is not lit by its own), -1
//!   afterwards;
//! - anything else: nothing more.
//!
//! "The lamps gathered" (`scene::gather_lights`, 0x451425, inlined): the object's
//! place `dword_63501C` .. to `g_litPlace`, `g_lightColour` = 0,
//! `scene::add_car_lights`, `scene::add_point_lights`. "Plus" is byte by byte
//! without overflow, as in `scene::vertex_light`.
//! One caller, `sub_45045D`.
//! Shadow mode: the Rust version, its writes put back, the original, compared.
//! Self-test: a redirected copy whose three calls of the gathering note what they
//! find and leave a colour.
//!
//! Converted 2026-10-21: the kind's way through `rc2k_core::scene::lamps::object_light`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::rva;
use crate::input::store::DeviceSlot;
use crate::scene::add_car_light::add_car_light;
use crate::scene::add_car_lights::{DRAWN_CAR_RVA, add_car_lights};
use crate::scene::add_point_lights::{LIGHT_COLOUR_RVA, LIT_PLACE_RVA, add_point_lights};
use crate::scene::vertex_light::plus;
use crate::session::MACHINE;
use crate::store::{game_read, game_write};

// ---- game data ----

const ADDED_RVA: usize = rva(0x635068); // dword_635068
const KIND_RVA: usize = rva(0x62DAF4); // g_lightKind
const STRENGTH_RVA: usize = rva(0x635090); // g_lightStrength
const KEY_RVA: DeviceSlot = DeviceSlot::at(0x4F6B74); // byte_4F6B74
const PLACE_RVA: usize = rva(0x63501C); // dword_63501C, dword_635020, dword_635024
const BASE_RVA: usize = rva(0x635054); // dword_635054
const THIS_CAR_RVA: usize = rva(0x60627C); // dword_60627C

// ---- Rust version ----

/// `gather`: what stands for `scene::add_car_lights` and `scene::add_point_lights`.
// Converted: `rc2k_core::scene::lamps::object_light`
pub unsafe fn set_object_light(
    gather: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        use rc2k_core::scene::lamps::{ObjectLight, WHITE, object_light};
        game_write(ADDED_RVA, 0u32);
        let (clear_strength, own_car) =
            match object_light(game_read(KIND_RVA), || game_read::<u8>(KEY_RVA) != 0) {
                ObjectLight::White => {
                    game_write(ADDED_RVA, WHITE);
                    return Ok(());
                }
                ObjectLight::Gathered {
                    clear_strength,
                    own_car,
                } => (clear_strength, own_car),
                ObjectLight::Nothing => return Ok(()),
            };
        if clear_strength {
            game_write(STRENGTH_RVA, 0u32);
        }
        game_write(LIT_PLACE_RVA, game_read::<[u32; 3]>(PLACE_RVA));
        if own_car {
            game_write(DRAWN_CAR_RVA, game_read::<u32>(THIS_CAR_RVA));
        }
        game_write(LIGHT_COLOUR_RVA, 0u32);
        gather()?;
        if own_car {
            game_write(DRAWN_CAR_RVA, u32::MAX);
        }
        let mmx = MACHINE.has_mmx.get() != 0;
        game_write(
            ADDED_RVA,
            plus(game_read(LIGHT_COLOUR_RVA), game_read(BASE_RVA), mmx),
        );
        Ok(())
    }
}

/// With the lamps' Rust versions.
pub unsafe fn set_object_light_in_game() -> Result<(), Refused> {
    unsafe {
        set_object_light(&mut || {
            add_car_lights(&mut |car| Ok(add_car_light(car)?))?;
            add_point_lights()?;
            Ok(())
        })
    }
}
