//! The hook and self-test machinery for functions that draw through the blitter and leave a
//! few game variables behind: the service area's overlay functions use it. (The four stage
//! panels have their own, older version of the same idea in `stage_panel.rs`.)

use crate::errors::Refused;
use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI, Regs};
use crate::text::draw_text::{Canvas, GameCanvas, Outcome};

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
    /// What a converted text drawer leaves (as `of_text`: the x is the one passed in).
    pub fn of_drawn(out: rc2k_core::screens::screen::Drawn, x: i32) -> Self {
        Self {
            carry: Some(out.too_wide),
            regs: vec![
                (EAX, 0),
                (EBX, out.after),
                (ECX, x as u32),
                (EDX, out.y as u32),
            ],
            zero: None,
        }
    }

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

/// `dispatch_draw` / `dispatch_draw_rewound` outside shadow mode, without its hook (typed tables, 2026-10-22: the entry the menus' and draw lists' enums run, with no hook):
/// `blank` (by the function's name), then the Rust version on the game's screen.
pub fn draw_entry(_name: &str, r: &mut Regs, rust: Draw) -> Result<(), Refused> {
    rust(r, &mut GameCanvas)?.apply(r);
    Ok(())
}
