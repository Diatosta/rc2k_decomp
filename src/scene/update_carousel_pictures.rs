//! `scene::update_carousel_pictures` (0x49029C): the pictures of four carousel
//! entries into the four places that hold pictures.
//!
//! In:  nothing (`dword_75AAA8` .. `dword_75AAB4` the four entries)
//! Out: eax = one past the last texture assigned, ebx = one past the last slot,
//!      ecx = 0, esi = 24, edi = the fourth entry's record
//!
//! `scene/carousel_pictures.rs` has what it does, with the two functions inlined
//! (`scene::keep_held_pictures` 0x49036F, `scene::assign_picture_slots` 0x4903E3).
//! Two callers.
//! Refuses an entry past the records or without one, and slots past the table.
//! Self-test: 60 settings by lot, with records of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::scene::carousel_pictures::{
    ENTRIES_RVA, FIRST_SLOT_RVA, FIRST_TEXTURE_RVA, HELD_RVA, WANTED_PICTURES_RVA, slot_regions,
    update,
};
use crate::scene::textures::{SLOTS, WANTED_RVA, table};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::update_carousel_pictures",
    rva(0x49029C),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { update(game_read(WANTED_PICTURES_RVA)) }?)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &slot_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

/// A world of the test's: six entries' records, each a picture's number.
pub struct World {
    pictures: Box<[u32; 6]>,
    records: Box<[u32; 6]>,
    regions: [Region; 2],
    inputs: [Region; 3],
    saved: (Vec<u8>, Vec<u8>),
}

impl World {
    pub fn new() -> Self {
        let pictures = Box::new([3u32, 9, 9, 0, 0x41, u32::MAX]);
        let records: Box<[u32; 6]> =
            Box::new(std::array::from_fn(|n| &raw const pictures[n] as u32));
        let regions = [
            (game_addr(WANTED_PICTURES_RVA) as u32, 0x30),
            table(WANTED_RVA, SLOTS),
        ];
        let inputs = [at(ENTRIES_RVA), at(FIRST_SLOT_RVA), at(FIRST_TEXTURE_RVA)];
        let saved = (capture(&regions), capture(&inputs));
        Self {
            pictures,
            records,
            regions,
            inputs,
            saved,
        }
    }

    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    /// Everything the functions read and write, set.
    pub unsafe fn set(&self, entries: [u32; 4], held: [u32; 4], slot: u32, texture: u32) {
        unsafe {
            restore(&self.regions, &vec![0xAA; self.saved.0.len()]);
            game_write(ENTRIES_RVA, self.records.as_ptr() as u32);
            game_write(WANTED_PICTURES_RVA, entries);
            game_write(HELD_RVA, held);
            game_write(FIRST_SLOT_RVA, slot);
            game_write(FIRST_TEXTURE_RVA, texture);
        }
    }

    pub fn picture(&self, entry: u32) -> u32 {
        self.pictures[entry as usize]
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        restore(&self.regions, &self.saved.0);
        restore(&self.inputs, &self.saved.1);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = World::new();
    let mut seed = 0x7E87_0004u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    for case in 0..60 {
        let entries = [lot(6), lot(6), lot(6), lot(6)];
        // Held: pictures the entries have, others, and none.
        let mut held = [0u32; 4];
        for held in &mut held {
            *held = match lot(4) {
                0 => u32::MAX,
                1 => lot(0x50),
                _ => world.picture(lot(6)),
            };
        }
        let slot = [0u32, 0x200, SLOTS - 32][lot(3) as usize];
        let texture = [0u32, 0x300, 0xFFFF_FF00][lot(3) as usize];
        check_state(
            &mut t,
            &format!(
                "case {case}: entries {entries:x?}, held {held:x?}, slot {slot:#x}, texture {texture:#x}"
            ),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { world.set(entries, held, slot, texture) },
            world.regions(),
            &rust,
        );
    }
    t.finish()
}
