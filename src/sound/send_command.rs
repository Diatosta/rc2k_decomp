//! `sound::send_command` (0x409C45): puts a command for the audio thread in the queue.
//!
//! In:  the command's number on the stack (the caller takes it off again); the registers
//!      are what goes with the command
//! Out: every register preserved
//! See `sound/queue.rs`. Ten callers, all of them a few bytes long: one a command
//! (`push n; call; pop`), which the ported senders do not go through.
//! Shadow mode runs the original alone (a command must not be queued twice), through a
//! caller of its own that puts the command on the stack.
//! Self-test: a redirected copy whose queue is the test's, called through a copy of one
//! of those callers.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{
    COUNT_RVA, NO_AUDIO_RVA, Queue, RECORD_LEN, RECORDS, RECORDS_RVA, WRITE_RVA, send_command, sent,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::send_command", rva(0x409C45), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

// The generated `call_original` is not used: see `call_with_command`.
const _: unsafe extern "C" fn(*mut Regs) = call_original;

/// Calls the original with the registers of `regs` and the command on the stack, as its
/// callers do. `call_original` cannot: it puts nothing on the stack, and the original
/// would take whatever lies there for the command (which it did until 2026-10-12: with
/// shadow on no command reached the audio thread as itself, and the game was silent).
/// The original keeps every register, so none is stored back.
#[unsafe(naked)]
unsafe extern "C" fn call_with_command(_regs: *const Regs, _command: u32, _original: usize) {
    core::arch::naked_asm!(
        "push ebx",
        "push esi",
        "push edi",
        "push ebp",
        "mov eax, [esp + 20]",       // regs
        "push dword ptr [esp + 28]", // the original's address
        "push dword ptr [esp + 28]", // the command
        "mov edi, [eax]",
        "mov esi, [eax + 4]",
        "mov ebp, [eax + 8]",
        "mov ebx, [eax + 16]",
        "mov edx, [eax + 20]",
        "mov ecx, [eax + 24]",
        "mov eax, [eax + 28]",
        "call dword ptr [esp + 4]",
        "add esp, 8",
        "pop ebp",
        "pop edi",
        "pop esi",
        "pop ebx",
        "ret",
    )
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // Above the flags the thunk pushed: the return address, then the command.
    let command = unsafe { ((r.esp as usize + 8) as *const u32).read_unaligned() };
    if shadow() {
        let original = ORIGINAL.load(Ordering::SeqCst);
        unsafe { call_with_command(r, command, original) };
        return;
    }
    unsafe { send_command(&Queue::game(), command, sent(r)) };
}

// ---- self-test ----

const LEN: usize = 0x89;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0xB4, 0x82, 0x4F, 0x00, 0x00];
/// How often the code names the audio switch, the count, the place to write and the
/// records.
const NAMED: [usize; 4] = [1, 3, 4, 1];
/// One of the callers: `pusha; push 5; call send_command; pop eax; popa; retn`.
const CALLER_RVA: usize = rva(0x409BD5);
const CALLER_LEN: usize = 0x0B;
const CALLER_CALLS: [usize; 1] = [0x03];
/// A small queue's worth of the test's memory (the code always wraps at 0x400).
const TEST_RECORDS: usize = RECORDS as usize;

/// What the test's queue holds: the switch, the count, the place to write, the records.
struct Memory {
    vars: [u32; 3],
    records: Vec<u32>,
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy =
        unsafe { RedirectedCopy::new(game_addr(rva(0x409C45)), LEN, &ENTRY_BYTES, &[], &|_| None) };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut memory = Memory {
        vars: [0; 3],
        records: vec![0; TEST_RECORDS * RECORD_LEN / 4],
    };
    let queue = Queue {
        no_audio: &raw const memory.vars[0],
        count: &raw mut memory.vars[1],
        write: &raw mut memory.vars[2],
        records: memory.records.as_mut_ptr(),
    };
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let real = [NO_AUDIO_RVA, COUNT_RVA, WRITE_RVA, RECORDS_RVA].map(|at| game_addr(at) as u32);
        let new = [
            queue.no_audio as u32,
            queue.count as u32,
            queue.write as u32,
            queue.records as u32,
        ];
        let mut patches = Vec::new();
        let mut seen = [0usize; 4];
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            match real.iter().position(|&address| address == named) {
                Some(n) => {
                    seen[n] += 1;
                    patches.push((at, new[n]));
                    at += 4;
                }
                None => at += 1,
            }
        }
        if seen != NAMED {
            return t.skip("the original is not what the test was made from");
        }
        for (at, new) in patches {
            copy.patch_u32(at, new);
        }
    }
    let caller = unsafe {
        RedirectedCopy::new(
            game_addr(CALLER_RVA),
            CALLER_LEN,
            &[0x60, 0x6A, 0x05, 0xE8],
            &CALLER_CALLS,
            &|_| Some(copy.addr()),
        )
    };
    let caller = match caller {
        Ok(caller) => caller,
        Err(why) => return t.skip(why),
    };

    // (what, the switch, the count, the place to write, the command)
    let cases: [(&str, u32, u32, u32, u8); 8] = [
        ("an empty queue", 0, 0, 0, 1),
        ("some waiting", 0, 5, 9, 0x0B),
        ("the last place", 0, 0x10, 0x3FF, 3),
        ("one place left", 0, 0x3FF, 0x77, 0x0A),
        ("a full queue", 0, 0x400, 0x20, 5),
        ("no audio", 1, 7, 3, 5),
        ("no audio, a full queue", 0xFFFF_FFFF, 0x400, 0x3FF, 8),
        ("a count past full", 0, 0x401, 0x3FE, 9),
    ];
    for (what, no_audio, count, write, command) in cases {
        let before = Regs {
            eax: 0x1111_0001,
            ebx: 0x2222_0002,
            ecx: 0x3333_0003,
            edx: 0x4444_0004,
            esi: 0x5555_0005,
            edi: 0x6666_0006,
            ebp: 0x7777_0007,
            ..Regs::SENTINEL
        };
        let mut run = |rust: bool| unsafe {
            std::ptr::write_volatile(&raw mut memory.vars, [no_audio, count, write]);
            memory.records.fill(0xAAAA_AAAA);
            let preserved = if rust {
                send_command(&queue, command as u32, sent(&before));
                true
            } else {
                caller.patch_u32(0, u32::from_le_bytes([0x60, 0x6A, command, 0xE8]));
                let mut ro = before;
                call_game(&mut ro, caller.addr());
                ro.preserved_except(&before, 0)
            };
            (
                preserved,
                std::ptr::read_volatile(&raw const memory.vars),
                memory.records.clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!(
                "{what}: original preserved={} left {:x?}; rust left {:x?}; first different dword of the records {:?}",
                theirs.0,
                theirs.1,
                mine.1,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
            )
        });
    }
    t.finish()
}
