//! `screens::yes_no_caption` (0x46FA5A): the "leave?" question's button shows the
//! answer.
//!
//! In:  nothing
//! Out: eax as `strings::append_string_by_id` leaves it, ebx = after the NUL
//!
//! String 0x56B + `g_yesNoAnswer` (0x56B "não", 0x56C "sim", by the language file) and
//! a NUL into `g_captionText`, from which `screens::draw_highlighted_caption` draws the
//! button. In the question screen's draw list (0x758C9E). IDA has not made it a
//! function.
//! Refuses a caption longer than its place.
//! Self-test: four answers.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::yes_no_caption", rva(0x46FA5A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const CAPTION_LEN: usize = 0x100;
const FIRST: u32 = 0x56B;

// ---- Rust version ----

/// Returns eax and ebx as the original leaves them.
pub unsafe fn yes_no_caption() -> Result<(u32, u32), Refused> {
    unsafe {
        let id = game_read::<u32>(ANSWER_RVA).wrapping_add(FIRST);
        let mut text = Vec::new();
        let eax = append_string_by_id(id, &mut |p, n| {
            text.extend_from_slice(std::slice::from_raw_parts(p, n))
        });
        // Converted: `rc2k_core::screens::captions::fit`.
        let text = rc2k_core::screens::captions::fit(text, CAPTION_LEN)?;
        let caption = game_addr(CAPTION_RVA) as *mut u8;
        std::ptr::copy_nonoverlapping(text.as_ptr(), caption, text.len());
        Ok((eax, caption as u32 + text.len() as u32))
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(CAPTION_RVA) as u32, CAPTION_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let (eax, ebx) = unsafe { yes_no_caption() }?;
    Ok(Left::nothing().with(EAX, eax).with(EBX, ebx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let inputs = [at(ANSWER_RVA)];
    let saved = (capture(&regions()), capture(&inputs));
    for answer in [0u32, 1, 2, 3] {
        check_state(
            &mut t,
            &format!("answer={answer}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(ANSWER_RVA, answer);
                restore(&regions(), &[0xAAu8; CAPTION_LEN]);
            },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
