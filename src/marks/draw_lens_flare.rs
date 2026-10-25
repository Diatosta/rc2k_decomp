//! `marks::draw_lens_flare` (0x4C4ED3): the lens flare, 0x11 parts along the line from the
//! glow's place on the screen (the settings' +0x2758, +0x275C) through the screen's
//! centre. The way out (`flt_91FFDA` x, `flt_91FFDE` -y) also into `flt_92051E` ..
//! (`flt_920526` 0) and made one long there (`scene::normalise_vector3`); its length
//! `flt_91FFD6`; the alpha `flt_9206D6` less the length times `flt_9206D2` (`fistp`, at
//! least 0) times the settings' +0x2754, >> 8. Then for each part k from 0x10 down to 0:
//! its centre (-x, y times `g_marksWaveLevels`[k] plus the centre, unrounded),
//! `marks::draw_flare_part` with corner `dword_92052A`[k], size `dword_9205B2`[k] times
//! `flt_9206DA` (`fistp`), angle 0, colour 0xFFFFFF, that alpha.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `sub_4C4EA2`. Refuses null settings and a part's corner past `g_marksCorners`,
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::draw_flare_part::draw_flare_part;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const WAY_RVA: [Slot; 2] = [Slot::at(0x91FFDA), Slot::at(0x91FFDE)]; // flt_91FFDA, flt_91FFDE
const UNIT_RVA: Slot = Slot::at(0x92051E); // flt_92051E .. flt_920526
const LENGTH_RVA: Slot = Slot::at(0x91FFD6); // flt_91FFD6
const FADE: [Slot; 2] = [Slot::at(0x9206D2), Slot::at(0x9206D6)]; // flt_9206D2, flt_9206D6
const SIZE_K_RVA: Slot = Slot::at(0x9206DA); // flt_9206DA
const CORNER_RVA: Slot = Slot::at(0x92052A); // dword_92052A: 0x11
const SIZE_RVA: Slot = Slot::at(0x9205B2); // dword_9205B2: 0x11
const PARTS: u32 = 0x11;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header).
pub unsafe fn draw_lens_flare() -> Result<(), Refused> {
    unsafe {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if settings == 0 {
            return Err(BadInput("no marks settings").into());
        }
        if (0..PARTS).any(|k| game_read::<u32>(CORNER_RVA + 4 * k as usize) >= 0x100) {
            return Err(BadInput("a flare corner past g_marksCorners").into());
        }
        let [cx, cy] = CENTRE_RVA.map(|r| game_read::<u32>(r));
        let dx = x87::sub(get(settings + 0x2758), cx);
        let dy = x87::sub(get(settings + 0x275C), cy) ^ 0x8000_0000;
        game_write(WAY_RVA[0], dx);
        game_write(WAY_RVA[1], dy);
        game_write(UNIT_RVA, x87::loaded(dx));
        game_write(UNIT_RVA + 4, x87::loaded(dy));
        game_write(UNIT_RVA + 8, 0u32);
        crate::scene::vector3::normalise_vector3(game_addr(UNIT_RVA) as u32);
        let length = crate::math::x87_load::float_hypot(dx, dy);
        game_write(LENGTH_RVA, length);
        let alpha = x87::times_subr_to_int(length, game_read(FADE[0]), game_read(FADE[1])).max(0);
        let product = u64::from(get(settings + 0x2754)) * u64::from(alpha as u32);
        let alpha = (product as u32) >> 8;
        let size_k: u32 = game_read(SIZE_K_RVA);
        for k in (0..PARTS as usize).rev() {
            let level: u32 = crate::marks::waves::WAVE_LEVELS.levels[k].get();
            let y = x87::times_plus_80(dy, level, cy, false);
            let x = x87::times_plus_80(dx, level, cx, true);
            draw_flare_part(
                game_read(CORNER_RVA + 4 * k),
                0,
                alpha,
                x87::int_times_to_int(game_read(SIZE_RVA + 4 * k), size_k) as u32,
                0xFF_FFFF,
                [x, y],
            )?;
        }
    }
    Ok(())
}
