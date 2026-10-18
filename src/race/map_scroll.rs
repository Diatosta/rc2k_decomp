//! The map's scroll: where the views' cars are on the course map and the map picture
//! redrawn around them (`sub_45B4E6` -> `sub_45B527` -> `sub_45B663` -> `sub_45B68F`, part
//! R3, IDA 2026-10-18). The work copy `g_car` holds +0x98 the car's last checkpoint, +0x134 /
//! +0x13C its place, +0xAD4 / +0xAD8 the map picture's corner (16-pixel steps), +0xADC 1 when
//! it was redrawn, +0xAE0 / +0xAE4 the car on the map (16.16), +0xAE8 / +0xAEC its place in
//! the picture.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

const CAR: u32 = 0x719FD8; // g_car
const CORNER_X: u32 = CAR + 0xAD4;
const CORNER_Y: u32 = CAR + 0xAD8;
const REDRAWN: u32 = CAR + 0xADC;
const MAP_X: u32 = CAR + 0xAE0;
const MAP_Y: u32 = CAR + 0xAE4;

/// `race::map_tiles` (0x45B663): the map picture's slot `dword_5D0E00` (+ `g_currentCar`
/// unless `dword_52A910`) drawn from tile (corner x >> 4, corner y >> 4) (part 1:
/// `sub_45B68F`, ecx / edx the tile, eax the slot).
pub unsafe fn map_tiles(r: &mut Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    r.ecx = ((k(CORNER_X) as i32) >> 4) as u32;
    r.edx = ((k(CORNER_Y) as i32) >> 4) as u32;
    r.eax = k(0x5D0E00);
    if k(0x52A910) == 0 {
        r.eax = r.eax.wrapping_add(k(0x71B3F8));
    }
    part(1, r)
}

/// `n` x 0x60 / 1000 as `imul` / `idiv`.
fn scaled(n: u32) -> u32 {
    ((n as i32 as i64 * 0x60) / 1000) as i32 as u32
}

/// What `scroll_map` reads through the course: refused before anything.
unsafe fn scroll_refuses() -> Result<(), Refused> {
    if k(0x623640) >= 2 && (k(0x5CCCBC) == 0 || k(0x5CCCC0) == 0) {
        return Err(BadInput("no course faces or points").into());
    }
    Ok(())
}

/// `race::scroll_map` (0x45B527): with two checkpoints or more, the car on the map
/// (`g_mapX` / `g_mapY` at its last checkpoint, plus its distance from that checkpoint's
/// face's point x 0x60 / 1000, z negated); unless it is still 0x34 .. 0x4B pixels inside the
/// picture, the picture's corner moved (centred, 16-pixel steps) and redrawn (part 1:
/// `map_tiles`), +0xADC 1; its place in the picture ((place - corner << 16 - 0x300000) x 2).
/// Gives the registers it leaves. Refuses (before anything) no course faces or points.
pub unsafe fn scroll_map(r: &mut Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    unsafe {
        scroll_refuses()?;
        w(REDRAWN, 0);
        if k(0x623640) < 2 {
            return Ok(());
        }
        let n = k(CAR + 0x98);
        r.ebx = n;
        w(MAP_X, k(0x9FC940 + 4 * n));
        w(MAP_Y, k(0xA019A8 + 4 * n));
        let face = k(0x5CCCBC).wrapping_add(k(0x9E3738 + 4 * n) << 6);
        let point = ((face + 4) as *const u32).read_unaligned();
        let p = point.wrapping_mul(12).wrapping_add(k(0x5CCCC0));
        r.esi = p;
        let px = (p as *const u32).read_unaligned();
        let pz = ((p + 8) as *const u32).read_unaligned();
        w(
            MAP_X,
            k(MAP_X).wrapping_add(scaled(k(CAR + 0x134).wrapping_sub(px))),
        );
        let dz = k(CAR + 0x13C).wrapping_sub(pz).wrapping_neg();
        w(MAP_Y, k(MAP_Y).wrapping_add(scaled(dz)));
        // edx the last `idiv`'s remainder (of dz x 0x60 / 1000)
        r.edx = ((dz as i32 as i64 * 0x60) % 1000) as i32 as u32;
        r.ebx = 1000;
        let x = ((k(MAP_X) as i32) >> 16) as u32;
        let y = ((k(MAP_Y) as i32) >> 16) as u32;
        let (dx, dy) = (x.wrapping_sub(k(CORNER_X)), y.wrapping_sub(k(CORNER_Y)));
        let inside =
            |d: u32| (d.wrapping_sub(0x34) as i32) >= 0 && (d.wrapping_sub(0x4C) as i32) < 0;
        r.eax = dx;
        r.ebx = dy;
        if !(inside(dx) && inside(dy)) {
            w(CORNER_X, x.wrapping_add(8).wrapping_sub(0x40) & !0xF);
            w(CORNER_Y, y.wrapping_add(8).wrapping_sub(0x40) & !0xF);
            // the corner as the original has just stored it, through eax and ebx
            r.eax = k(CORNER_X);
            r.ebx = k(CORNER_Y);
            part(1, r)?;
            w(REDRAWN, 1);
        }
        let at = |corner: u32, place: u32| {
            (corner.wrapping_neg() << 16)
                .wrapping_add(place)
                .wrapping_sub(0x30_0000)
                << 1
        };
        r.eax = at(k(CORNER_X), k(MAP_X));
        w(CAR + 0xAE8, r.eax);
        r.eax = at(k(CORNER_Y), k(MAP_Y));
        w(CAR + 0xAEC, r.eax);
        Ok(())
    }
}

/// `race::scroll_maps` (0x45B4E6): `scroll_map` on each view's car in the work copy (part 1
/// the scroll): with `dword_52A910` the third view car only (`race::load_car_c` /
/// `store_car_c`), else the first and, unless `dword_63DB98` is 1, the second. Refuses
/// (before anything) a view car without its block, or what `scroll_map` refuses.
pub unsafe fn scroll_maps(r: &mut Regs, part: crate::race::frame::Part) -> Result<(), Refused> {
    use crate::race::frame::{VIEW_CARS, load_view_car, store_view_car};
    unsafe {
        let views: &[usize] = if k(0x52A910) != 0 {
            &[2]
        } else if k(0x63DB98) == 1 {
            &[0]
        } else {
            &[0, 1]
        };
        for &v in views {
            let car = game_read::<u32>(VIEW_CARS[v]) & 7;
            if k(0x71B3B8 + 4 * car) == 0 {
                return Err(BadInput("a view car without its block").into());
            }
        }
        scroll_refuses()?;
        for &v in views {
            load_view_car(v)?.apply(r);
            part(1, r)?;
            store_view_car(v)?.apply(r);
        }
        Ok(())
    }
}

/// The parts in the game: `scroll_map` with `map_tiles` with the tiles' blit
/// (`sub_45B68F` -> `race::blit_map_tiles`).
pub fn scroll_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { scroll_map(r, &mut |_, r| map_tiles(r, &mut tiles_part)) }
}

/// `map_tiles`' part in the game: `sub_45B68F` -> `race::blit_map_tiles` (the tiles'
/// blit, its Rust version).
pub fn tiles_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::race::blit_map_tiles::blit_map_tiles_in_game(r) }
}
