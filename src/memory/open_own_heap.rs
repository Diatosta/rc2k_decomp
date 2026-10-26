//! `memory::open_own_heap` (0x40298A): with the setting `altmemenable` (`g_ownHeap`
//! 0x4E0584 = 1), the memory of the game's own heap taken at the start.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! The heap's size in Kb is `dword_4E0588`, at least 0x2800 (signed). "MEM: Using
//! alternative scheme, heap=" and the size go to the log (level 1); the size * 0x400 (an
//! `imul`, wrapping) from `memory::virtual_alloc` goes to `dword_4E058C` and
//! `dword_4E0590`, and the memory's first dwords are 0 and the bytes. One caller, `start`
//! (0x401030); in the game this never gets a call (the hook DLL is loaded later). Refuses
//! (before anything) a size whose bytes wrap to 0: the original writes at address 0 there.
//! Shadow mode runs the original alone (memory cannot be taken twice). Self-test: a copy
//! whose four variables are the test's and whose three calls go to stand-ins (the memory a
//! buffer of the test's), on the switch and the sizes, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};

// ---- game data ----

const SIZE_RVA: usize = rva(0x4E0588); // dword_4E0588: the heap's Kb (`altmemsize`)
const HEAP_RVA: usize = rva(0x4E058C); // dword_4E058C, dword_4E0590: the heap's memory
const TEXT_RVA: usize = rva(0x4DA115); // "MEM: Using alternative scheme, heap="

/// The calls by place: 0 `logfile::log_write_l1` (eax the text), 1
/// `logfile::log_int_l1_nl` (eax), 2 `memory::virtual_alloc` (eax the bytes -> eax).
const CALLEES: [usize; 3] = [0x401427, 0x401476, 0x402B1A];
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct HeapVars {
    pub own_heap: u32,
    pub size: u32,
    /// The two dwords the memory goes to.
    pub heap: u32,
}

impl HeapVars {
    pub fn game() -> Self {
        Self {
            own_heap: game_addr(crate::memory::OWN_HEAP_RVA) as u32,
            size: game_addr(SIZE_RVA) as u32,
            heap: game_addr(HEAP_RVA) as u32,
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn set(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

pub unsafe fn open_own_heap(v: &HeapVars, part: Part) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::memory::{own_heap_on, own_heap_size, own_heap_head}`.
        use rc2k_core::memory::{own_heap_head, own_heap_on, own_heap_size};
        if !own_heap_on(get(v.own_heap)) {
            return Ok(());
        }
        let Some((kb, bytes)) = own_heap_size(get(v.size)) else {
            return Err(BadInput("a heap size whose bytes wrap to 0").into());
        };
        part(
            0,
            &mut Regs {
                eax: game_addr(TEXT_RVA) as u32,
                ..Regs::default()
            },
        )?;
        part(
            1,
            &mut Regs {
                eax: kb,
                ..Regs::default()
            },
        )?;
        let mut r = Regs {
            eax: bytes,
            ecx: bytes,
            ..Regs::default()
        };
        part(2, &mut r)?;
        let memory = r.eax;
        set(v.heap, memory);
        set(v.heap + 4, memory);
        let [first, second] = own_heap_head(bytes);
        set(memory, first);
        set(memory + 4, second);
        Ok(())
    }
}

/// In the game: the logs through the game's entries, the memory from the Rust version.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 2 {
        r.eax = unsafe { crate::memory::virtual_alloc::virtual_alloc(r.eax) }?;
        return Ok(());
    }
    crate::calls::call(rva(CALLEES[place as usize]), r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { open_own_heap(&HeapVars::game(), &mut game_part) }?;
    Ok(())
}
