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
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("memory::open_own_heap", rva(0x40298A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
    unsafe { call_game(r, game_addr(rva(CALLEES[place as usize]))) };
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { open_own_heap(&HeapVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

/// The calls noted: (place, eax).
static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());
/// The memory the stand-in of `virtual_alloc` gives.
static GIVES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn note(place: u32, r: &mut Regs) {
    NOTED.lock().unwrap().push((place, r.eax));
    if place == 2 {
        r.eax = GIVES.load(std::sync::atomic::Ordering::SeqCst);
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    act(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = vec![0u32; 4];
    let at = vars.as_mut_ptr() as u32;
    let v = HeapVars {
        own_heap: at,
        size: at + 4,
        heap: at + 8,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40298A,
            len: 0x51,
            entry: &[0x60, 0x83, 0x3D, 0x84, 0x05, 0x4E, 0x00, 0x01],
            calls: &[0x22, 0x29, 0x37],
            jumps_out: &[],
        },
        &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        },
        &[
            (0x4E0584, v.own_heap),
            (0x4E0588, v.size),
            (0x4E058C, v.heap),
            (0x4E0590, v.heap + 4),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(note);
    let mut memory = vec![0xAAu8; 16];
    GIVES.store(
        memory.as_mut_ptr() as u32,
        std::sync::atomic::Ordering::SeqCst,
    );
    for own in [0u32, 1, 2] {
        for size in [0u32, 0x100, 0x2800, 0x4000, 0xFFFF_FFFF, 0x8000_0000] {
            let run = |rust: bool| unsafe {
                let p = at as *mut u32;
                p.write_volatile(own);
                p.add(1).write_volatile(size);
                p.add(2).write_volatile(0xAAAA_AAAA);
                p.add(3).write_volatile(0xAAAA_AAAA);
                std::ptr::write_bytes(
                    GIVES.load(std::sync::atomic::Ordering::SeqCst) as *mut u8,
                    0xAA,
                    16,
                );
                NOTED.lock().unwrap().clear();
                let (done, preserved) = if rust {
                    (open_own_heap(&v, &mut test_part).is_ok(), true)
                } else {
                    let before = Regs::SENTINEL;
                    let mut ro = before;
                    crate::framework::call_game(&mut ro, c.addr());
                    (true, ro.preserved_except(&before, 0))
                };
                let state: Vec<u32> = (0..4).map(|n| p.add(n).read_volatile()).collect();
                let given = std::slice::from_raw_parts(
                    GIVES.load(std::sync::atomic::Ordering::SeqCst) as *const u8,
                    16,
                )
                .to_vec();
                (
                    done && preserved,
                    state,
                    given,
                    std::mem::take(&mut *NOTED.lock().unwrap()),
                )
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs == mine, || {
                format!("switch {own}, size {size:#x}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    drop(memory);
    drop(vars);
    t.finish()
}
