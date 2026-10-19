//! The hook and self-test machinery for functions that draw through the blitter and leave a
//! few game variables behind: the service area's overlay functions use it. (The four stage
//! panels have their own, older version of the same idea in `stage_panel.rs`.)

use crate::errors::Refused;
use crate::framework::{
    EAX, EBP, EBX, ECX, EDI, EDX, ESI, GameHook, RawHook, Regs, SelfTest, shadow,
};
use crate::gfx::blit_glyph::{BlitCall, record_calls, recording_works};
use crate::screens::blanked;
use crate::service::harness::{Region, capture, restore};
use crate::text::draw_text::{Canvas, GameCanvas, Outcome, RecordingCanvas};
use crate::text::glyph_kerning::state_ptr;
use std::sync::atomic::{AtomicU32, Ordering};

/// Counts the starts and ends of self-tests that change, with the game running, what the
/// game's menu drawing decides by (the highlighted item): odd while one is going on. A
/// shadow comparison that such a test's writes may have fallen into is not reported.
static DISTURBANCES: AtomicU32 = AtomicU32::new(0);

/// Held by a self-test while it has what frames read changed (in case a thread got past the
/// pause: a pause timeout).
pub struct Disturbing(());

impl Disturbing {
    pub fn begin() -> Self {
        DISTURBANCES.fetch_add(1, Ordering::SeqCst);
        Self(())
    }
}

impl Drop for Disturbing {
    fn drop(&mut self) {
        LAST_DISTURBED.store(crate::system::tick_count::tick_count(), Ordering::SeqCst);
        DISTURBANCES.fetch_add(1, Ordering::SeqCst);
    }
}

/// The clock when the last such test let go.
static LAST_DISTURBED: AtomicU32 = AtomicU32::new(0);
/// A comparison takes far less than this many milliseconds.
const SETTLES_IN: u32 = 250;

/// True while such a test holds the highlighted item changed, and for a moment after: a
/// shadow comparison that ends then may have had one of its two runs fall into it.
pub fn disturbed() -> bool {
    DISTURBANCES.load(Ordering::SeqCst) % 2 == 1
        || crate::system::tick_count::tick_count()
            .wrapping_sub(LAST_DISTURBED.load(Ordering::SeqCst))
            < SETTLES_IN
}

/// `hook.mismatch`, unless `disturbed()`: for the shadow comparisons of functions that
/// read the highlighted item.
pub fn mismatch_unless_disturbed(hook: &RawHook, detail: impl FnOnce() -> String) {
    if !disturbed() {
        hook.mismatch(detail);
    }
}

/// `call_original` of a hook.
pub type CallOriginal = unsafe extern "C" fn(*mut Regs);

/// What a drawing function leaves in the registers and flags. Registers not listed are
/// preserved.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Left {
    /// The carry flag, where the function's callers could read it.
    pub carry: Option<bool>,
    /// (register, value): `EAX` and so on.
    pub regs: Vec<(u8, u32)>,
    /// The zero flag, where it is not `!carry` (which `apply` sets otherwise) and the
    /// callers could read it.
    pub zero: Option<bool>,
}

impl Left {
    /// Every register preserved, flags not meaningful.
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Every register preserved, the carry flag as given.
    pub fn carry(carry: bool) -> Self {
        Self {
            carry: Some(carry),
            regs: Vec::new(),
            zero: None,
        }
    }

    /// What `draw_text` leaves: carry, eax = 0, ebx, ecx and edx.
    pub fn of_text(out: &Outcome) -> Self {
        Self {
            carry: Some(out.too_wide),
            regs: vec![
                (EAX, 0),
                (EBX, out.text as u32),
                (ECX, out.x as u32),
                (EDX, out.y as u32),
            ],
            zero: None,
        }
    }

    pub fn with(mut self, reg: u8, value: u32) -> Self {
        self.regs.retain(|&(r, _)| r != reg);
        self.regs.push((reg, value));
        self
    }

    fn changed(&self) -> u8 {
        self.regs.iter().fold(0, |mask, &(reg, _)| mask | reg)
    }

    pub fn apply(&self, r: &mut Regs) {
        for &(reg, value) in &self.regs {
            *field(r, reg) = value;
        }
        if let Some(carry) = self.carry {
            r.set_carry(carry);
            r.set_zero(!carry);
        }
        if let Some(zero) = self.zero {
            r.set_zero(zero);
        }
    }

    /// True if `after` is what this says, starting from `before`.
    pub fn matches(&self, before: &Regs, after: &Regs) -> bool {
        let mut copy = *after;
        self.regs
            .iter()
            .all(|&(reg, value)| *field(&mut copy, reg) == value)
            && after.preserved_except(before, self.changed())
            && self.carry.is_none_or(|c| after.carry() == c)
            && self
                .zero
                .is_none_or(|z| (after.eflags & crate::framework::ZERO_FLAG != 0) == z)
    }
}

fn field(r: &mut Regs, reg: u8) -> &mut u32 {
    match reg {
        EAX => &mut r.eax,
        ECX => &mut r.ecx,
        EDX => &mut r.edx,
        EBX => &mut r.ebx,
        EBP => &mut r.ebp,
        ESI => &mut r.esi,
        EDI => &mut r.edi,
        _ => unreachable!("not a register"),
    }
}

/// The Rust version of a drawing function, as the harness runs it: it gets the registers at
/// entry and a canvas, and says what it leaves in the registers.
pub type Draw<'a> = &'a dyn Fn(&Regs, &mut dyn Canvas) -> Result<Left, Refused>;

/// The game variables and buffers a function leaves behind, as bytes, for comparing.
pub type State<'a> = &'a dyn Fn() -> Vec<u8>;

/// The whole `dispatch` of such a hook: `blank`, the Rust version when shadow is off, and the
/// shadow comparison. In shadow mode the Rust version runs first without drawing, so it must
/// be a function whose second run (the original's) starts from the same inputs.
pub fn dispatch_draw(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: Draw,
    state: State,
) {
    dispatch_draw_rewound(hook, r, call_original, rust, state, &[]);
}

/// `dispatch_draw` for a function that also changes what it decides by (a "last seen"
/// variable, say): in shadow mode `rewind` is put back after the Rust run, so that the
/// original decides from the same values.
pub fn dispatch_draw_rewound(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: Draw,
    state: State,
    rewind: &[Region],
) {
    hook.count_call();
    if blanked(hook.name()) {
        return;
    }

    if !shadow() {
        match rust(r, &mut GameCanvas) {
            Ok(left) => left.apply(r),
            // The original, run from the start, redraws the same things and meets the same
            // error.
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }

    if !crate::service::harness::shadow_due(hook) {
        unsafe { call_original(r) };
        return;
    }

    let disturbances = DISTURBANCES.load(Ordering::SeqCst);
    let before = capture(rewind);
    let mut canvas = RecordingCanvas::new(true);
    let left = match rust(r, &mut canvas) {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let state_r = state();
    restore(rewind, &before);

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let state_o = state();

    let ok = left.matches(r, &o)
        && state_o == state_r
        && unsafe { *state_ptr() } == canvas.state
        && (!recording_works() || calls == canvas.calls);
    let disturbed = disturbances % 2 == 1 || DISTURBANCES.load(Ordering::SeqCst) != disturbances;
    if !ok && !disturbed {
        hook.mismatch(|| {
            format!(
                "regs_ok={} same_state={} original calls={} rust calls={} first different call={:?}",
                left.matches(r, &o),
                state_o == state_r,
                calls.len(),
                canvas.calls.len(),
                first_difference(&calls, &canvas.calls)
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

fn first_difference(a: &[BlitCall], b: &[BlitCall]) -> Option<usize> {
    if a == b {
        return None;
    }
    Some(
        a.iter()
            .zip(b)
            .position(|(x, y)| x != y)
            .unwrap_or(a.len().min(b.len())),
    )
}

/// One self-test case: `prepare` puts the inputs in place, the original runs with the blitter
/// calls swallowed, `prepare` runs again, the Rust version runs the same way, and the two are
/// compared. `what` names the case in a failure line.
pub fn check_draw(
    t: &mut SelfTest,
    what: &str,
    call_original: CallOriginal,
    before: Regs,
    prepare: &dyn Fn(),
    rust: Draw,
    state: State,
) {
    let saved_kerning = unsafe { *state_ptr() };

    prepare();
    let mut ro = before;
    let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
    let state_o = state();
    let kerning_o = unsafe { *state_ptr() };

    prepare();
    let mut canvas = RecordingCanvas::new(false);
    let mine = rust(&before, &mut canvas);
    let state_r = state();

    let regs_ok = mine.as_ref().is_ok_and(|left| left.matches(&before, &ro));
    let ok = regs_ok && state_o == state_r && calls_o == canvas.calls && kerning_o == canvas.state;
    t.check(ok, || {
        let first = first_difference(&calls_o, &canvas.calls);
        let shown = first.map(|n| (calls_o.get(n).copied(), canvas.calls.get(n).copied()));
        let state_at = state_o
            .iter()
            .zip(&state_r)
            .position(|(a, b)| a != b)
            .map(|n| {
                let around = n.saturating_sub(8)..(n + 8).min(state_o.len()).min(state_r.len());
                (n, state_o[around.clone()].to_vec(), state_r[around].to_vec())
            });
        format!(
            "{what}: regs_ok={regs_ok} same_state={} same_kerning={} original eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x} carry={} calls={}; rust {mine:?} calls={} first different call={:?}: original / rust {shown:x?}; state first differs at (byte, original's and rust's bytes from 8 before) {state_at:x?}",
            state_o == state_r,
            kerning_o == canvas.state,
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.edx,
            ro.esi,
            ro.edi,
            ro.carry(),
            calls_o.len(),
            canvas.calls.len(),
            first_difference(&calls_o, &canvas.calls)
        )
    });

    unsafe { *state_ptr() = saved_kerning };
}

/// Why drawing functions cannot be self-tested right now, or None if they can.
pub fn drawing_unavailable() -> Option<&'static str> {
    (!recording_works())
        .then_some("the gfx::blit_glyph hook is off, so the original would really draw")
}

/// Appends game variables to a state: each as four bytes.
pub fn push_dwords(state: &mut Vec<u8>, rvas: &[usize]) {
    for &at in rvas {
        let v: u32 = unsafe { crate::framework::game_read(at) };
        state.extend_from_slice(&v.to_le_bytes());
    }
}
