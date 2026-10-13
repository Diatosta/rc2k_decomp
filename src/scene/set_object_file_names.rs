//! `scene::set_object_file_names` (0x43964A): the paths of a kind of object's two
//! files.
//!
//! In:  eax = the kind (which folder), `dword_634FE8` = the kind (the number)
//! Out: eax = the tens digit as a character, ecx = 10, edx = the units digit as a
//!      character, esi and ebp = where the number is in the two names
//!
//! By the kind: under 0x190 `files\a\obj????.ro3` and `.lwo`, under 0x258 `files\b\`,
//! under 0x320 `files\g\frnt\`, else `files\s\extr\`. Four pointers are set: the
//! `.ro3` path (`dword_60AD28`) and where its number is (`dword_60AD24`), the `.lwo`
//! path (`dword_60AD2C`) and its number (`dword_60AD30`). The number's four decimal
//! digits are written over the `????` of both (the thousands digit as one byte:
//! anything above 9999 gives another character).
//! One caller, `scene::load_object`.
//! Self-test: 40 kinds; the names are put back afterwards.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBP, ECX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm,
    rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::set_object_file_names",
    rva(0x43964A),
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

// ---- game data ----

/// `dword_60AD24` the `.ro3` name's number, `dword_60AD28` the `.ro3` path,
/// `dword_60AD2C` the `.lwo` path, `dword_60AD30` the `.lwo` name's number.
pub const NAMES_RVA: usize = rva(0x60AD24);
pub const KIND_RVA: usize = rva(0x634FE8); // dword_634FE8
/// The eight path texts, from `aFilesAObjRo3` on.
const TEXTS_RVA: usize = rva(0x60AD4E);
const TEXTS_LEN: usize = 0xB4;
/// For each folder: the `.ro3` path, the `.lwo` path, and how far in the number is.
const FOLDERS: [(u32, usize, usize, usize); 4] = [
    (0x190, rva(0x60AD4E), rva(0x60AD62), 0x0B),
    (0x258, rva(0x60AD76), rva(0x60AD8A), 0x0B),
    (0x320, rva(0x60AD9E), rva(0x60ADB7), 0x10),
    (u32::MAX, rva(0x60ADD0), rva(0x60ADE9), 0x10),
];

// ---- Rust version ----

pub unsafe fn set_object_file_names(folder_kind: u32) -> Left {
    unsafe {
        let &(_, ro3, lwo, number) = FOLDERS
            .iter()
            .find(|&&(below, ..)| folder_kind < below)
            .unwrap_or(&FOLDERS[3]);
        let (ro3_number, lwo_number) = (game_addr(ro3 + number), game_addr(lwo + number));
        game_write(
            NAMES_RVA,
            [
                ro3_number as u32,
                game_addr(ro3) as u32,
                game_addr(lwo) as u32,
                lwo_number as u32,
            ],
        );
        let kind: u32 = game_read(KIND_RVA);
        let digits = [
            (kind / 1000) as u8,
            (kind % 1000 / 100) as u8,
            (kind % 100 / 10) as u8,
            (kind % 10) as u8,
        ]
        .map(|digit| digit.wrapping_add(b'0'));
        (ro3_number as *mut [u8; 4]).write(digits);
        (lwo_number as *mut [u8; 4]).write(digits);
        Left::nothing()
            .with(EAX, digits[2] as u32)
            .with(ECX, 10)
            .with(EDX, digits[3] as u32)
            .with(ESI, ro3_number as u32)
            .with(EBP, lwo_number as u32)
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        (game_addr(NAMES_RVA) as u32, 16),
        (game_addr(TEXTS_RVA) as u32, TEXTS_LEN),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { set_object_file_names(r.eax) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let kind_region = [(game_addr(KIND_RVA) as u32, 4)];
    let saved = (capture(&regions), capture(&kind_region));
    let kinds = [
        0u32,
        7,
        0x18F,
        0x190,
        0x257,
        0x258,
        0x31F,
        0x320,
        999,
        1000,
        4321,
        9999,
        10000,
        0xFFFF_FFFF,
        570,
        601,
        0x2BC,
        123_456,
        2_000_000_000,
        259_999,
    ];
    for (n, &kind) in kinds.iter().enumerate() {
        // The folder's kind is the number's but for every other case.
        for folder in [kind, kinds[(n + 3) % kinds.len()]] {
            check_state(
                &mut t,
                &format!("kind {kind:#x}, folder by {folder:#x}"),
                call_original,
                Regs {
                    eax: folder,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    restore(&regions, &saved.0);
                    game_write(NAMES_RVA, [0xAAAA_AAAAu32; 4]);
                    game_write(KIND_RVA, kind);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved.0);
    restore(&kind_region, &saved.1);
    t.finish()
}
