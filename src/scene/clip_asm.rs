//! The x87 arithmetic of the clippers, generated from the listing by
//! `tools/clip_asm/gen_clip.py` (not by hand: do not edit). Every instruction is the
//! original's: those without a memory operand as their own bytes, the others with
//! their operand in a `Work` block instead of the game's entries, records and work
//! variables. A byte the original loads through `dword_62400C` is in the block as a
//! whole number already.

/// The block the arithmetic works in, in dwords. Offsets: 0 the first point's record,
/// 0x20 the second's, 0x40 and 0x88 their entries, 0xD0 and 0xE4 the entries' five
/// bytes (+0x18, +0x19, +0x1A, +0x20, +0x21) as dwords, 0xF8 the work variables
/// `flt_623F80` .. `flt_623FAC`, 0x128 `dword_6241AC` .. `dword_6241BC`, 0x13C
/// `dword_62400C`, 0x140 `dword_62411C`.
pub type Work = [u32; 0x51];
pub const RECORD_A: usize = 0x00;
pub const RECORD_B: usize = 0x20;
pub const ENTRY_A: usize = 0x40;
pub const ENTRY_B: usize = 0x88;
pub const BYTES_A: usize = 0xD0;
pub const BYTES_B: usize = 0xE4;
pub const OUT_1: usize = 0xF8;
pub const OUT_2: usize = 0x128;
pub const CHECKED: usize = 0x13C;
pub const PLANE: usize = 0x140;

/// `sub_44D247` from after its `fstp flt_623F80` (the edge is in the block already) to its `call`.
pub unsafe fn cut_at_x(w: *mut Work) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            "fld dword ptr [{w} + 0xF8]", // fld flt_623F80
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fld dword ptr [{w} + 0xC]", // fld dword ptr [esi+0Ch]
            "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            ".byte 0xD9, 0xE0", // fchs
            ".byte 0xD8, 0xC2", // fadd st, st(2)
            "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            ".byte 0xD8, 0xC2", // fadd st, st(2)
            ".byte 0xD9, 0xCE", // fxch st(6)
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fld dword ptr [{w} + 0xF8]", // fld flt_623F80
            "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            ".byte 0xD8, 0xF3", // fdiv st, st(3)
            ".byte 0xD9, 0xCB", // fxch st(3)
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fld dword ptr [{w} + 0x30]", // fld dword ptr [ebp+10h]
            "fld dword ptr [{w} + 0x10]", // fld dword ptr [esi+10h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x100]", // fstp flt_623F88
            "fld dword ptr [{w} + 0x34]", // fld dword ptr [ebp+14h]
            "fld dword ptr [{w} + 0x14]", // fld dword ptr [esi+14h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x104]", // fstp flt_623F8C
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x108]", // fstp flt_623F90
            "fild dword ptr [{w} + 0x20]", // fild dword ptr [ebp+0]
            "fild dword ptr [{w} + 0x0]", // fild dword ptr [esi]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x10C]", // fistp dword_623F94
            "fild dword ptr [{w} + 0x24]", // fild dword ptr [ebp+4]
            "fild dword ptr [{w} + 0x4]", // fild dword ptr [esi+4]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x110]", // fistp dword_623F98
            "fild dword ptr [{w} + 0x28]", // fild dword ptr [ebp+8]
            "fild dword ptr [{w} + 0x8]", // fild dword ptr [esi+8]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x114]", // fistp dword_623F9C
            "fld dword ptr [{w} + 0x90]", // fld dword ptr [ebp+8]
            "fld dword ptr [{w} + 0x48]", // fld dword ptr [esi+8]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x118]", // fstp flt_623FA0
            "fld dword ptr [{w} + 0x94]", // fld dword ptr [ebp+0Ch]
            "fld dword ptr [{w} + 0x4C]", // fld dword ptr [esi+0Ch]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x11C]", // fstp flt_623FA4
            "fld dword ptr [{w} + 0x98]", // fld dword ptr [ebp+10h]
            "fld dword ptr [{w} + 0x50]", // fld dword ptr [esi+10h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x120]", // fstp flt_623FA8
            "fld dword ptr [{w} + 0x9C]", // fld dword ptr [ebp+14h]
            "fld dword ptr [{w} + 0x54]", // fld dword ptr [esi+14h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x124]", // fstp flt_623FAC
            "fild dword ptr [{w} + 0xE4]", // fild dword_62400C
            "fild dword ptr [{w} + 0xD0]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x130]", // fistp dword_6241B4
            "fild dword ptr [{w} + 0xE8]", // fild dword_62400C
            "fild dword ptr [{w} + 0xD4]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x134]", // fistp dword_6241B8
            "fild dword ptr [{w} + 0xEC]", // fild dword_62400C
            "fild dword ptr [{w} + 0xD8]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x138]", // fistp dword_6241BC
            "fild dword ptr [{w} + 0xF0]", // fild dword_62400C
            "fild dword ptr [{w} + 0xDC]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x128]", // fistp dword_6241AC
            "fild dword ptr [{w} + 0xF4]", // fild dword_62400C
            "fild dword ptr [{w} + 0xE0]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x12C]", // fistp dword_6241B0
            "fistp dword ptr [{w} + 0x13C]", // fistp dword_62400C
            "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
            "fld dword ptr [{w} + 0xC]", // fld dword ptr [esi+0Ch]
            ".byte 0xDC, 0xE9", // fsub st(1), st
            "fld dword ptr [{w} + 0xF8]", // fld flt_623F80
            ".byte 0xDE, 0xE9", // fsubp st(1), st
            "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
            "fld dword ptr [{w} + 0x1C]", // fld dword ptr [esi+1Ch]
            ".byte 0xDC, 0xE9", // fsub st(1), st
            ".byte 0xD9, 0xCA", // fxch st(2)
            ".byte 0xD9, 0xE0", // fchs
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            ".byte 0xD9, 0xCA", // fxch st(2)
            ".byte 0xDE, 0xFA", // fdivp st(2), st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0xFC]", // fstp flt_623F84
            w = in(reg) w,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// `sub_44D532`, likewise with `flt_623F84`.
pub unsafe fn cut_at_y(w: *mut Work) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            "fld dword ptr [{w} + 0xFC]", // fld flt_623F84
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fld dword ptr [{w} + 0x1C]", // fld dword ptr [esi+1Ch]
            "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            ".byte 0xD9, 0xE0", // fchs
            ".byte 0xD8, 0xC2", // fadd st, st(2)
            "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            ".byte 0xD8, 0xC2", // fadd st, st(2)
            ".byte 0xD9, 0xCE", // fxch st(6)
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fld dword ptr [{w} + 0xFC]", // fld flt_623F84
            "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            ".byte 0xD8, 0xF3", // fdiv st, st(3)
            ".byte 0xD9, 0xCB", // fxch st(3)
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            "fld dword ptr [{w} + 0x30]", // fld dword ptr [ebp+10h]
            "fld dword ptr [{w} + 0x10]", // fld dword ptr [esi+10h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x100]", // fstp flt_623F88
            "fld dword ptr [{w} + 0x34]", // fld dword ptr [ebp+14h]
            "fld dword ptr [{w} + 0x14]", // fld dword ptr [esi+14h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x104]", // fstp flt_623F8C
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            "fld dword ptr [{w} + 0x18]", // fld dword ptr [esi+18h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x108]", // fstp flt_623F90
            "fild dword ptr [{w} + 0x20]", // fild dword ptr [ebp+0]
            "fild dword ptr [{w} + 0x0]", // fild dword ptr [esi]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x10C]", // fistp dword_623F94
            "fild dword ptr [{w} + 0x24]", // fild dword ptr [ebp+4]
            "fild dword ptr [{w} + 0x4]", // fild dword ptr [esi+4]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x110]", // fistp dword_623F98
            "fild dword ptr [{w} + 0x28]", // fild dword ptr [ebp+8]
            "fild dword ptr [{w} + 0x8]", // fild dword ptr [esi+8]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x114]", // fistp dword_623F9C
            "fld dword ptr [{w} + 0x90]", // fld dword ptr [ebp+8]
            "fld dword ptr [{w} + 0x48]", // fld dword ptr [esi+8]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x118]", // fstp flt_623FA0
            "fld dword ptr [{w} + 0x94]", // fld dword ptr [ebp+0Ch]
            "fld dword ptr [{w} + 0x4C]", // fld dword ptr [esi+0Ch]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x11C]", // fstp flt_623FA4
            "fld dword ptr [{w} + 0x98]", // fld dword ptr [ebp+10h]
            "fld dword ptr [{w} + 0x50]", // fld dword ptr [esi+10h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x120]", // fstp flt_623FA8
            "fld dword ptr [{w} + 0x9C]", // fld dword ptr [ebp+14h]
            "fld dword ptr [{w} + 0x54]", // fld dword ptr [esi+14h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x124]", // fstp flt_623FAC
            "fild dword ptr [{w} + 0xE4]", // fild dword_62400C
            "fild dword ptr [{w} + 0xD0]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x130]", // fistp dword_6241B4
            "fild dword ptr [{w} + 0xE8]", // fild dword_62400C
            "fild dword ptr [{w} + 0xD4]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x134]", // fistp dword_6241B8
            "fild dword ptr [{w} + 0xEC]", // fild dword_62400C
            "fild dword ptr [{w} + 0xD8]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x138]", // fistp dword_6241BC
            "fild dword ptr [{w} + 0xF0]", // fild dword_62400C
            "fild dword ptr [{w} + 0xDC]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x128]", // fistp dword_6241AC
            "fild dword ptr [{w} + 0xF4]", // fild dword_62400C
            "fild dword ptr [{w} + 0xE0]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x12C]", // fistp dword_6241B0
            "fistp dword ptr [{w} + 0x13C]", // fistp dword_62400C
            "fld dword ptr [{w} + 0x3C]", // fld dword ptr [ebp+1Ch]
            "fld dword ptr [{w} + 0x1C]", // fld dword ptr [esi+1Ch]
            ".byte 0xDC, 0xE9", // fsub st(1), st
            "fld dword ptr [{w} + 0xFC]", // fld flt_623F84
            ".byte 0xDE, 0xE9", // fsubp st(1), st
            "fld dword ptr [{w} + 0x2C]", // fld dword ptr [ebp+0Ch]
            "fld dword ptr [{w} + 0xC]", // fld dword ptr [esi+0Ch]
            ".byte 0xDC, 0xE9", // fsub st(1), st
            ".byte 0xD9, 0xCA", // fxch st(2)
            ".byte 0xD9, 0xE0", // fchs
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            ".byte 0xD9, 0xCA", // fxch st(2)
            ".byte 0xDE, 0xFA", // fdivp st(2), st
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0xF8]", // fstp flt_623F80
            w = in(reg) w,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// `sub_44DB93` from 0x44DC37 to 0x44DD9A (`sub_44DEAA` has the same there).
pub unsafe fn cut_at_depth(w: *mut Work) {
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{w} + 0x140]", // fild dword_62411C
            "fsub dword ptr [{w} + 0x18]", // fsub dword ptr [esi+18h]
            "fld dword ptr [{w} + 0x38]", // fld dword ptr [ebp+18h]
            "fsub dword ptr [{w} + 0x18]", // fsub dword ptr [esi+18h]
            ".byte 0xDE, 0xF9", // fdivp st(1), st
            "fst dword ptr [{w} + 0x13C]", // fst dword_62400C
            "fld dword ptr [{w} + 0x10]", // fld dword ptr [esi+10h]
            "fld dword ptr [{w} + 0x30]", // fld dword ptr [ebp+10h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x100]", // fstp flt_623F88
            "fld dword ptr [{w} + 0x14]", // fld dword ptr [esi+14h]
            "fld dword ptr [{w} + 0x34]", // fld dword ptr [ebp+14h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x104]", // fstp flt_623F8C
            "fild dword ptr [{w} + 0x140]", // fild dword_62411C
            "fstp dword ptr [{w} + 0x108]", // fstp flt_623F90
            "fld dword ptr [{w} + 0x48]", // fld dword ptr [esi+8]
            "fld dword ptr [{w} + 0x90]", // fld dword ptr [ebp+8]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x118]", // fstp flt_623FA0
            "fld dword ptr [{w} + 0x4C]", // fld dword ptr [esi+0Ch]
            "fld dword ptr [{w} + 0x94]", // fld dword ptr [ebp+0Ch]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x11C]", // fstp flt_623FA4
            "fld dword ptr [{w} + 0x50]", // fld dword ptr [esi+10h]
            "fld dword ptr [{w} + 0x98]", // fld dword ptr [ebp+10h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x120]", // fstp flt_623FA8
            "fld dword ptr [{w} + 0x54]", // fld dword ptr [esi+14h]
            "fld dword ptr [{w} + 0x9C]", // fld dword ptr [ebp+14h]
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fstp dword ptr [{w} + 0x124]", // fstp flt_623FAC
            "fild dword ptr [{w} + 0xD0]", // fild dword_62400C
            "fild dword ptr [{w} + 0xE4]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x130]", // fistp dword_6241B4
            "fild dword ptr [{w} + 0xD4]", // fild dword_62400C
            "fild dword ptr [{w} + 0xE8]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x134]", // fistp dword_6241B8
            "fild dword ptr [{w} + 0xD8]", // fild dword_62400C
            "fild dword ptr [{w} + 0xEC]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x138]", // fistp dword_6241BC
            "fild dword ptr [{w} + 0xDC]", // fild dword_62400C
            "fild dword ptr [{w} + 0xF0]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x128]", // fistp dword_6241AC
            "fild dword ptr [{w} + 0xE0]", // fild dword_62400C
            "fild dword ptr [{w} + 0xF4]", // fild dword_62400C
            ".byte 0xD8, 0xE1", // fsub st, st(1)
            ".byte 0xD8, 0xCA", // fmul st, st(2)
            ".byte 0xDE, 0xC1", // faddp st(1), st
            "fistp dword ptr [{w} + 0x12C]", // fistp dword_6241B0
            "fstp dword ptr [{w} + 0x13C]", // fstp dword_62400C
            w = in(reg) w,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}
