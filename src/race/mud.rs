//! `race::loadlevel_init`'s "render3d initmud" (IDA, 2026-10-16): 64 KB on
//! `stru_5C6F50` for the mud (`dword_623D50`), the car's mud file
//! (`files\b\mudfile.0xx`, xx the car's file number) read into it, the mud's state
//! cleared, and 256 colours sampled from a picture surface (`dword_5D0DEC`, locked and
//! unlocked through DirectDraw: the game's, `sub_41AA4B` / `sub_41AB02`).

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

pub const MUD_LIST_RVA: usize = rva(0x5C6F50); // stru_5C6F50
pub const MUD_RVA: usize = rva(0x623D50); // dword_623D50
pub const MUD_LEN: u32 = 0x1_0000;
pub const LIST_TOTAL_RVA: usize = rva(0x5C6F90); // dword_5C6F90
pub const CAR_RVA: usize = rva(0x70F410); // dword_70F410: the car
const FILE_NUMBERS_RVA: usize = rva(0x5C5928); // dword_5C5928: 0x22 cars' file numbers
pub const MUD_PATH_RVA: usize = rva(0x6238F4); // "files\b\mudfile.0xx"
pub const MUD_PATH_DIGITS_RVA: usize = rva(0x623905);
pub const STATE_RVA: usize = rva(0x623D58); // dword_623D58 .. dword_623D9C
pub const STATE_DWORDS: usize = 18;
pub const STATE_MORE: [usize; 2] = [rva(0x62390C), rva(0x623918)];
pub const PICTURE_RVA: usize = rva(0x5D0DEC); // dword_5D0DEC: the surface sampled
pub const COLOURS_RVA: usize = rva(0x623930); // 256 dwords
const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const HEIGHT_RVA: usize = rva(0x4F49A4); // g_surface.height
const FORMAT_RVA: usize = rva(0x4F49B8); // g_surface.format
const PITCH_RVA: usize = rva(0x4F49BC); // g_surface.pitch
const PIXELS_RVA: usize = rva(0x4F49C0); // g_surface.pixels
const BITS_RVA: usize = rva(0x4F49A8); // g_surface.bits_per_pixel

/// `race::car_file_number` (0x41F4FD): eax held to 0 .. 0x21 (signed), its entry of
/// `dword_5C5928`.
pub fn car_file_number(car: u32) -> u32 {
    // Converted: `rc2k_core::race::cars::table_entry`.
    let n = rc2k_core::race::cars::table_entry(car);
    unsafe { game_read(FILE_NUMBERS_RVA + 4 * n) }
}

/// `gfx::read_pixel` (0x406584): the pixel at (ecx, edx) of `g_surface`, taken apart by
/// `gfx::pixel_to_components`. A 16-bit pixel is read with `mov ax`: the upper half of eax
/// is the row's address's, and that is what the conversion gets.
pub unsafe fn read_pixel(x: u32, y: u32) -> u32 {
    unsafe {
        let (_, pixel) = raw_pixel(x, y);
        let format = game_read::<u32>(FORMAT_RVA) as *const crate::gfx::pixel_format::PixelFormat;
        crate::gfx::pixel_to_components::pixel_to_components(&format.read_unaligned(), pixel)
    }
}

/// The row's address and the value `read_pixel` hands to the conversion (one place for
/// both: a second copy of this compiled without the row's upper half, 2026-10-16).
pub unsafe fn raw_pixel(x: u32, y: u32) -> (u32, u32) {
    unsafe {
        let row = game_read::<u32>(PIXELS_RVA).wrapping_add(y.wrapping_mul(game_read(PITCH_RVA)));
        let pixel = if game_read::<u32>(BITS_RVA) == 0x10 {
            // As the original: eax the row's address, then `mov ax` of the pixel. Written in
            // assembly: the optimiser dropped the row's half from the same expression in
            // Rust in one of two places (2026-10-16).
            let mut eax = row;
            core::arch::asm!(
                "mov ax, word ptr [{at}]",
                at = in(reg) row.wrapping_add(x.wrapping_mul(2)),
                inout("eax") eax,
                options(nostack, readonly, preserves_flags),
            );
            eax
        } else {
            (row.wrapping_add(x.wrapping_mul(4)) as *const u32).read_unaligned()
        };
        (row, pixel)
    }
}

/// `race::mud_pixel` (0x44A58A): `read_pixel` inside the surface (unsigned), else 0.
pub unsafe fn mud_pixel(x: u32, y: u32) -> u32 {
    unsafe {
        // Converted: `rc2k_core::race::mud::inside`.
        if !rc2k_core::race::mud::inside(x, y, game_read(WIDTH_RVA), game_read(HEIGHT_RVA)) {
            return 0;
        }
        read_pixel(x, y)
    }
}

/// A call: 0 `memory::allocate` (eax, ebx), 1 `files::read_whole_file` (eax, ebx, ecx), 2
/// the surface's lock (eax the surface), 3 its unlock, 4 `race::load_mud`, 5
/// `race::clear_mud_state`, 6 `race::sample_mud_colours`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `race::load_mud` (0x449D95): the car's file number's two digits (`div 10`, each +
/// '0', as bytes) into the path, the file (0x10000 bytes at most) into the mud. Leaves eax
/// the quotient (its low byte + '0'), edx the remainder (its low byte + '0'), ebx 10, esi
/// the mud.
pub unsafe fn load_mud(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let n = car_file_number(game_read(CAR_RVA));
        let (q, r) = (n / 10, n % 10);
        g.eax = q & !0xFF | (q as u8).wrapping_add(b'0') as u32;
        g.edx = r & !0xFF | (r as u8).wrapping_add(b'0') as u32;
        g.ebx = 10;
        let digits = game_addr(MUD_PATH_DIGITS_RVA) as *mut u8;
        *digits = g.eax as u8;
        *digits.add(1) = g.edx as u8;
        g.esi = game_read(MUD_RVA);
        let mut r = Regs {
            eax: game_addr(MUD_PATH_RVA) as u32,
            ebx: g.esi,
            ecx: MUD_LEN,
            ..g
        };
        part(1, &mut r)?;
    }
    Ok(g)
}

/// `race::clear_mud_state` (0x449E01): the 18 dwords at 0x623D58 and the two more 0.
pub unsafe fn clear_mud_state() {
    unsafe {
        game_write(STATE_RVA, [0u32; STATE_DWORDS]);
        for at in STATE_MORE {
            game_write(at, 0u32);
        }
    }
}

/// `race::sample_mud_colours` (0x44A0D6): the surface locked (part 2), 256 colours, n's
/// at (0x89 + (n & 3) + (n >> 5) * 4, 0x68 + ((n & 0x1C) >> 2)) by `mud_pixel`, unlocked
/// (part 3). Leaves eax the surface, ecx and edx the last place, edi past the colours, ebp
/// 0x100.
pub unsafe fn sample_mud_colours(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(PICTURE_RVA);
        part(2, &mut g)?;
        g.edi = game_addr(COLOURS_RVA) as u32;
        for n in 0..0x100u32 {
            g.ecx = 0x89 + (n & 3) + (n >> 5) * 4;
            g.edx = 0x68 + ((n & 0x1C) >> 2);
            g.eax = mud_pixel(g.ecx, g.edx);
            (g.edi as *mut u32).write_unaligned(g.eax);
            g.edi += 4;
        }
        g.ebp = 0x100;
        g.eax = game_read(PICTURE_RVA);
        part(3, &mut g)?;
    }
    Ok(g)
}

/// `race::init_mud` (0x449DD6): the list's total + 0x10000, the mud taken (part 0) into
/// `dword_623D50`, then parts 4, 5, 6. Leaves the registers as they leave them.
pub unsafe fn init_mud(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = MUD_LEN;
        g.ebx = game_addr(MUD_LIST_RVA) as u32;
        game_write(
            LIST_TOTAL_RVA,
            game_read::<u32>(LIST_TOTAL_RVA).wrapping_add(MUD_LEN),
        );
        part(0, &mut g)?;
        game_write(MUD_RVA, g.eax);
        for place in [4, 5, 6] {
            part(place, &mut g)?;
        }
    }
    Ok(g)
}

/// The game's calls: the Rust versions, and the surface's lock and unlock at the game's
/// addresses (DirectDraw, the platform).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::framework::call_game;
    unsafe {
        match place {
            0 | 1 => crate::race::level_init::game_part(place, r)?,
            2 => call_game(r, game_addr(rva(0x41AA4B))),
            3 => call_game(r, game_addr(rva(0x41AB02))),
            4 => *r = load_mud(r, &mut game_part)?,
            5 => clear_mud_state(),
            _ => *r = sample_mud_colours(r, &mut game_part)?,
        }
    }
    Ok(())
}

// ---- for the self-tests ----

/// A copy whose calls to `places`' callees (as IDA shows them) go to stand-ins of those
/// places (`level_init::note`: noted with eax, ebx, ecx; place 0 gives `GIVES`); other calls
/// go where they went.
pub fn copy(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    places: &'static [(usize, u32)],
) -> Result<crate::redirect::RedirectedCopy, &'static str> {
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    unsafe {
        crate::redirect::RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            places
                .iter()
                .find(|&&(k, _)| rva(k) == c)
                .map(|&(_, p)| crate::install::stand_ins::stand_in(p))
        })
    }
}

/// A surface of the test's in `g_surface` until dropped: 64 x 48 pixels of the given
/// bits, filled by a pattern, the game's pixel format kept.
pub struct TestSurface {
    pub pixels: Vec<u8>,
    saved: Vec<u8>,
}

fn surface_region() -> crate::service::harness::Region {
    (game_addr(rva(0x4F4998)) as u32, 0x30)
}

impl TestSurface {
    pub fn new(bits: u32) -> Self {
        let saved = crate::service::harness::capture(&[surface_region()]);
        let bytes = bits / 8;
        let mut pixels: Vec<u8> = (0..64 * 48 * bytes).map(|n| (n * 37 + 11) as u8).collect();
        unsafe {
            game_write(WIDTH_RVA, 64u32);
            game_write(HEIGHT_RVA, 48u32);
            game_write(PITCH_RVA, 64 * bytes);
            game_write(BITS_RVA, bits);
            game_write(PIXELS_RVA, pixels.as_mut_ptr() as u32);
        }
        Self { pixels, saved }
    }
}

impl Drop for TestSurface {
    fn drop(&mut self) {
        crate::service::harness::restore(&[surface_region()], &self.saved);
    }
}

/// Whether the game has a pixel format to take pixels apart with.
pub fn has_format() -> bool {
    unsafe { game_read::<u32>(FORMAT_RVA) != 0 }
}

/// One case: `prepare`, the copy, `prepare`, the Rust version (its calls by `note` too);
/// the registers, the calls and `regions` compared.
pub fn run_both(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &crate::redirect::RedirectedCopy,
    regions: &[crate::service::harness::Region],
    prepare: &dyn Fn(),
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    run_both_from(t, what, copy, regions, prepare, Regs::SENTINEL, rust);
}

/// `run_both` with both sides starting from the registers `before` (a function's inputs).
pub fn run_both_from(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &crate::redirect::RedirectedCopy,
    regions: &[crate::service::harness::Region],
    prepare: &dyn Fn(),
    before: Regs,
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
) {
    use crate::race::level_init::take_called;
    use crate::service::harness::capture;
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    prepare();
    take_called();
    let mut ro = before;
    unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(regions));
    prepare();
    let mine = rust(&before);
    let mine = (
        mine.as_ref().ok().map(regs),
        take_called(),
        capture(regions),
    );
    t.check(theirs == mine, || {
        // Where the state first differs: the region and the byte in it.
        let mut at = None;
        let mut start = 0;
        for &(base, len) in regions {
            let (a, b) = (&theirs.2[start..start + len], &mine.2[start..start + len]);
            if let Some(k) = a.iter().zip(b).position(|(x, y)| x != y) {
                at = Some((base, k, a[k], b[k]));
                break;
            }
            start += len;
        }
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; state first differs at {at:x?} (region, byte, original, rust)",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}

/// The Rust side's calls in the tests: noted as the stand-ins note them.
pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::race::level_init::note(place, r);
    Ok(())
}
