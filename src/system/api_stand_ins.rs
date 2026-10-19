//! For self-tests: stand-ins for the Win32 functions an original calls (`stdcall`, the
//! arguments on the stack, popped by the callee). Each notes its place and its arguments
//! and gives in eax what the test set for the place (`set_gives`); ecx and edx are kept.
//! The Rust versions' calls of the same functions go through `Api` (in the game the real
//! functions, in the tests `note`), so both sides' calls can be compared.
//! `stand_in(place, args)` gives the address of the stand-in for a place that takes
//! `args` arguments (places 0 .. 7, 0 .. 6 arguments).

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// A Win32 call by place: the arguments in, eax out.
pub type Api<'a> = &'a mut dyn FnMut(u32, &[u32]) -> u32;

/// A call noted: the place and the arguments.
pub type Noted = (u32, Vec<u32>);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());
static GIVES: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];

/// What the stand-in of `place` gives from now on.
pub fn set_gives(place: u32, value: u32) {
    GIVES[place as usize & 7].store(value, Ordering::SeqCst);
}

/// The place (if not `u32::MAX`) whose second argument points at as many dwords as its
/// first says (`WaitForMultipleObjects`' handles): noted as those dwords, so that both
/// sides' arrays compare whatever their addresses.
static ARRAY_PLACE: AtomicU32 = AtomicU32::new(u32::MAX);

/// Sets the place whose second argument is noted as the array it points at.
pub fn set_array_place(place: u32) {
    ARRAY_PLACE.store(place, Ordering::SeqCst);
}

/// Notes a call and gives what the test set: the Rust side of the stand-ins.
pub fn note(place: u32, args: &[u32]) -> u32 {
    let mut noted = args.to_vec();
    if place == ARRAY_PLACE.load(Ordering::SeqCst) && args.len() >= 2 && args[0] <= 64 {
        let items = (0..args[0] as usize)
            .map(|k| unsafe { (args[1] as *const u32).add(k).read_unaligned() });
        noted = std::iter::once(args[0])
            .chain(items)
            .chain(args[2..].iter().copied())
            .collect();
    }
    NOTED.lock().unwrap().push((place, noted));
    GIVES[place as usize & 7].load(Ordering::SeqCst)
}

/// The calls noted since the last time, and none from then on.
pub fn take_noted() -> Vec<Noted> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

extern "C" fn note_from_stack(place: u32, args: *const u32, n: u32) -> u32 {
    let args: Vec<u32> = (0..n as usize)
        .map(|k| unsafe { args.add(k).read_unaligned() })
        .collect();
    note(place, &args)
}

macro_rules! api_stand_in {
    ($name:ident, $place:expr, $args:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push ecx",
                "push edx",
                // the arguments: past the two saved registers and the return address
                "lea eax, [esp + 12]",
                "push {n}",
                "push eax",
                "push {place}",
                "call {note}",
                "add esp, 12",
                "pop edx",
                "pop ecx",
                "ret {bytes}",
                n = const $args,
                place = const $place,
                bytes = const 4 * $args,
                note = sym note_from_stack,
            )
        }
    };
}

macro_rules! api_stand_ins {
    ($($place:literal: $a0:ident $a1:ident $a2:ident $a3:ident $a4:ident $a5:ident $a6:ident;)+) => {
        $(
            api_stand_in!($a0, $place, 0);
            api_stand_in!($a1, $place, 1);
            api_stand_in!($a2, $place, 2);
            api_stand_in!($a3, $place, 3);
            api_stand_in!($a4, $place, 4);
            api_stand_in!($a5, $place, 5);
            api_stand_in!($a6, $place, 6);
        )+
        const STAND_INS: [[unsafe extern "C" fn(); 7]; 8] = [
            $([$a0, $a1, $a2, $a3, $a4, $a5, $a6]),+
        ];
    };
}

api_stand_ins! {
    0: s0_0 s0_1 s0_2 s0_3 s0_4 s0_5 s0_6;
    1: s1_0 s1_1 s1_2 s1_3 s1_4 s1_5 s1_6;
    2: s2_0 s2_1 s2_2 s2_3 s2_4 s2_5 s2_6;
    3: s3_0 s3_1 s3_2 s3_3 s3_4 s3_5 s3_6;
    4: s4_0 s4_1 s4_2 s4_3 s4_4 s4_5 s4_6;
    5: s5_0 s5_1 s5_2 s5_3 s5_4 s5_5 s5_6;
    6: s6_0 s6_1 s6_2 s6_3 s6_4 s6_5 s6_6;
    7: s7_0 s7_1 s7_2 s7_3 s7_4 s7_5 s7_6;
}

/// The stand-in of `place` (0 .. 7) for a function of `args` (0 .. 6) arguments.
pub fn stand_in(place: u32, args: u32) -> usize {
    STAND_INS[place as usize & 7][(args as usize).min(6)] as *const () as usize
}

/// The test's `Api`: every call noted.
pub fn test_api(place: u32, args: &[u32]) -> u32 {
    note(place, args)
}

/// A copy of an original for a test: `outs` (opcode offsets of its calls and jumps out),
/// the calls to `places` (callees as IDA shows them) to the noting stand-ins of
/// `sound::turn_parts` (`take_called`), the calls at the offsets in `apis` to these
/// stand-ins (offset, place, arguments; the offsets also in `outs`), and every dword of
/// it inside one of `vars` (an RVA as IDA shows it, a length, the test's address for it)
/// moved to the test's memory; `references` such dwords expected.
#[allow(clippy::too_many_arguments)] // one test copy's whole description
pub fn copy_with(
    address: usize,
    len: usize,
    entry: &[u8],
    outs: &[usize],
    places: &[u32],
    apis: &[(usize, u32, u32)],
    vars: &[(usize, usize, u32)],
    references: usize,
) -> Result<crate::redirect::RedirectedCopy, String> {
    use crate::framework::{game_addr, rva};
    let copy = crate::sound::turn_parts::copy_of(address, len, entry, outs, places)
        .map_err(str::to_string)?;
    unsafe {
        for &(offset, place, args) in apis {
            if ((copy.addr() + offset) as *const u8).read() != 0xE8 {
                return Err(format!("no call at +{offset:#x}"));
            }
            copy.retarget(offset + 1, stand_in(place, args));
        }
        let mut patched = 0;
        let mut offset = 0;
        while offset + 4 <= len {
            let value = ((copy.addr() + offset) as *const u32).read_unaligned() as usize;
            let var = vars.iter().find(|&&(lo, n, _)| {
                let base = game_addr(rva(lo));
                (base..base + n).contains(&value)
            });
            match var {
                Some(&(lo, _, test)) => {
                    let base = game_addr(rva(lo));
                    copy.patch_u32(offset, test.wrapping_add((value - base) as u32));
                    patched += 1;
                    offset += 4;
                }
                None => offset += 1,
            }
        }
        if patched != references {
            return Err(format!(
                "{patched} references to the variables, not {references}"
            ));
        }
    }
    Ok(copy)
}
