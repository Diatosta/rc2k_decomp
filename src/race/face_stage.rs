//! A test stage for the course-face drawers (part R1, `race/course_faces.rs`): the light
//! test's scene (`race::light_colour_at`'s `LightStage`), the projection test's world
//! (`scene::project_point::World`: its own points, records and view), and the face's
//! working variables set by a `FaceCase`. Everything the drawers write is in `kept`.

use crate::framework::{Regs, call_game, game_addr, game_write, rva};
use crate::race::level_init::{note, take_called};
use crate::race::light_colour_at::{LIGHT_CASES, LightStage};
use crate::scene::project_point::World;
use crate::service::harness::{Region, at, capture};

/// One face to draw.
#[derive(Clone, Copy, Debug)]
pub struct FaceCase {
    /// `LIGHT_CASES` index.
    pub light: usize,
    /// The world's view (0 .. 3) and flat projection.
    pub view: usize,
    pub flat: bool,
    /// The four corners (points of the world: avoid 5 and 6), the fourth -1 for a
    /// triangle.
    pub corners: [u32; 4],
    /// The corners' lights (`dword_63DA54` ..; bits 16 .. 23 the lamps).
    pub lights: [u32; 4],
    /// `dword_63D9E4`, `dword_63D9E8`: the triangles facing away.
    pub away: (u32, u32),
    /// `g_polygonMode`.
    pub mode: u32,
    /// The two textures and their corner orders.
    pub textures: (u32, u32),
    pub orders: (u32, u32),
}

impl FaceCase {
    pub const fn new(corners: [u32; 4], lights: [u32; 4]) -> Self {
        Self {
            light: 0,
            view: 0,
            flat: false,
            corners,
            lights,
            away: (0, 0),
            mode: 2,
            textures: (3, 0),
            orders: (0, 1),
        }
    }
}

pub struct FaceStage {
    light: LightStage,
    world: World,
    pub kept: Vec<Region>,
}

impl FaceStage {
    /// `extra`: more to keep and compare. None when a car block is null.
    pub fn new(extra: &[Region]) -> Option<Self> {
        let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
        let mut kept = crate::race::draw_screen_quad::quad_regions();
        kept.extend([
            g(0x63D9E0, 0x120),
            g(0x6240BC, 0x30),
            g(0x623DD8, 0x10),
            g(0x63E168, 8),
            g(0x5183C0, 8),
            at(rva(0x63DB58)),
        ]);
        kept.extend(crate::scene::project_point::inputs());
        kept.extend_from_slice(extra);
        let light = LightStage::new(&kept)?;
        let kept = light.kept.clone();
        Some(Self {
            light,
            world: World::new(),
            kept,
        })
    }

    /// Everything put back, then the case in place.
    pub fn prepare(&mut self, c: &FaceCase) {
        self.light
            .prepare(&LIGHT_CASES[c.light % LIGHT_CASES.len()]);
        unsafe {
            self.world.set(c.view, c.flat, false);
            for k in 0..4usize {
                let w = |a: usize, v: u32| game_write(rva(a) + 4 * k, v);
                w(0x63D9EC, c.corners[k]);
                w(0x63DA44, 0x0030_2010u32.wrapping_mul(k as u32 + 1));
                w(0x63DA54, c.lights[k]);
                w(0x63DA64, 0x30 + 0x50 * k as u32);
                game_write(rva(0x63DA0C) + 8 * k, 0x1000 * k as u32);
                game_write(rva(0x63DA10) + 8 * k, 0x3000 - 0x800 * k as u32);
            }
            game_write(rva(0x63D9E4), c.away.0);
            game_write(rva(0x63D9E8), c.away.1);
            game_write(rva(0x63DAB4), c.mode);
            game_write(rva(0x63DB58), u32::from(c.lights.iter().any(|&l| l != 0)));
            game_write(rva(0x63DA2C), c.textures.0);
            game_write(rva(0x63DA38), c.textures.1);
            game_write(rva(0x63DA30), c.orders.0);
            game_write(rva(0x63DA3C), c.orders.1);
            crate::race::course_faces::fill_entries(&[(0, 0), (1, 1), (2, 2)]);
            game_write(rva(0x6240F8), 3u32);
            take_called();
        }
    }

    /// What the drawers wrote: the kept memory and the world's records.
    pub fn state(&self) -> (Vec<u8>, Vec<[u32; 8]>, u32) {
        let (records, checked) = self.world.state();
        (capture(&self.kept), records, checked)
    }

    pub fn end(self) {
        self.light.end();
    }
}

/// One case run both ways: the copy (`copy`, calls to stand-ins noting their places)
/// and the Rust version (`rust`, its parts noting the same places). Gives a line when they
/// differ: the calls and where the memory first differs.
pub fn compare(
    stage: &mut FaceStage,
    c: &FaceCase,
    copy: usize,
    carry: bool,
    rust: &mut dyn FnMut(&mut Regs) -> Result<bool, crate::errors::Refused>,
) -> Option<String> {
    stage.prepare(c);
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy) };
    let calls_o: Vec<u32> = take_called().iter().map(|c| c.0).collect();
    let theirs = stage.state();
    stage.prepare(c);
    let mut rr = Regs::SENTINEL;
    let done = rust(&mut rr);
    let carried = matches!(done, Ok(true));
    let calls_r: Vec<u32> = take_called().iter().map(|c| c.0).collect();
    let mine = stage.state();
    if done.is_ok() && calls_o == calls_r && theirs == mine && (!carry || ro.carry() == carried) {
        return None;
    }
    let memory = theirs.0.iter().zip(&mine.0).position(|(a, b)| a != b);
    let records = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
    Some(format!(
        "{c:x?}: {} carry {} / {carried}, calls {calls_o:?} / {calls_r:?}; memory first differs at {memory:x?}, records at {records:x?}, checked {:#x} / {:#x}",
        if done.is_ok() { "done" } else { "refused" },
        ro.carry(),
        theirs.2,
        mine.2
    ))
}

/// The noting part for the Rust side.
pub fn noted(place: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    note(place, r);
    Ok(())
}

/// The carry the stand-ins leave (for `race::draw_face_polygons`' quad).
pub static STAND_IN_CARRY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// The stand-ins' handler: notes the place, leaves `STAND_IN_CARRY`.
pub fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    r.set_carry(STAND_IN_CARRY.load(std::sync::atomic::Ordering::SeqCst));
}

/// One run both ways without the stage: `prepare` before each, the copy with `entry`'s
/// registers, then the Rust version; compares `regions` and the stand-ins' places and
/// the `seen` values the handler noted.
pub fn compare_plain(
    what: &str,
    prepare: &dyn Fn(),
    regions: &[Region],
    copy: usize,
    entry: Regs,
    rust: &mut dyn FnMut(&mut Regs) -> Result<(), crate::errors::Refused>,
) -> Option<String> {
    prepare();
    take_called();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy) };
    let theirs = (take_called(), capture(regions));
    prepare();
    take_called();
    let mut rr = entry;
    let done = rust(&mut rr);
    let mine = (take_called(), capture(regions));
    if done.is_ok() && theirs == mine {
        return None;
    }
    let memory = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
    Some(format!(
        "{what}: {} calls {:x?} / {:x?}; memory first differs at {memory:x?}",
        if done.is_ok() { "done" } else { "refused" },
        theirs.0,
        mine.0
    ))
}

/// The game variable (an RVA) whose value `seen_handler` notes at each stand-in's call.
pub static SEEN_RVA: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Notes the place and, in the note's eax, the variable `SEEN_RVA` (the other registers
/// 0): for calls whose registers are not compared.
pub fn seen_handler(place: u32, _: &mut Regs) {
    let at = SEEN_RVA.load(std::sync::atomic::Ordering::SeqCst);
    let mut r = Regs {
        eax: if at != 0 {
            unsafe { crate::framework::game_read(at) }
        } else {
            0
        },
        ..Regs::default()
    };
    note(place, &mut r);
}
