# python3 gen_clip.py clip.json > src/scene/clip_asm.rs, then cargo fmt. clip.json comes from
# dump_clip.py. The listing must have the names IDA gave at first (flt_623F80, dword_62400C, ..).
import json, re, sys
d = json.load(open(sys.argv[1]))
RA, RB, EA, EB, BA, BB, OUT1, OUT2, JUNK, PLANE = 0x00, 0x20, 0x40, 0x88, 0xD0, 0xE4, 0xF8, 0x128, 0x13C, 0x140
BYTE_AT = {0x18: 0, 0x19: 1, 0x1A: 2, 0x20: 3, 0x21: 4}
MEM_OK = {"fld", "fild", "fstp", "fistp", "fst", "fsub", "fadd", "fmul", "fdiv", "fcom", "fcomp"}

def gen(func, start, stop, skip_first_store):
    """x87 lines of func from address start up to (not including) stop."""
    mode = {"esi": "entry", "ebp": "entry"}
    lines, pending, skipped = [], None, not skip_first_store
    for ea, text, raw in d[func]:
        ea = int(ea, 16)
        text = text.split(";")[0].strip()
        mnem = text.split()[0]
        ops = text[len(mnem):].strip()
        # the registers' meaning is followed from the function's start
        if text in ("mov     esi, [esi+4]",): mode["esi"] = "rec"; continue
        if text in ("mov     ebp, [ebp+4]",): mode["ebp"] = "rec"; continue
        if text == "pop     esi": mode["esi"] = "entry"; continue
        if text == "pop     ebp": mode["ebp"] = "entry"; continue
        if ea < start or ea >= stop: continue
        m = re.match(r"mov\s+al, \[(esi|ebp)\+([0-9A-F]+)h\]", text)
        if m:
            assert mode[m.group(1)] == "entry"
            pending = (BA if m.group(1) == "esi" else BB) + 4 * BYTE_AT[int(m.group(2), 16)]
            continue
        if not mnem.startswith("f"):
            assert mnem in ("xor", "mov", "push", "pop", "imul", "lea"), text
            continue
        if "[" not in ops and "_" not in ops:
            b = bytes.fromhex(raw); assert len(b) == 2, text
            lines.append('".byte 0x%02X, 0x%02X", // %s' % (b[0], b[1], " ".join(text.split())))
            continue
        assert mnem in MEM_OK, text
        m = re.match(r"dword ptr \[(esi|ebp)(?:\+([0-9A-F]+)h?)?\]$", ops)
        if m:
            reg, off = m.group(1), int(m.group(2) or "0", 16)
            base = {("esi", "rec"): RA, ("ebp", "rec"): RB, ("esi", "entry"): EA, ("ebp", "entry"): EB}[(reg, mode[reg])]
            at = base + off
        else:
            m = re.match(r"(?:flt|dword)_([0-9A-F]+)$", ops); assert m, text
            a = int(m.group(1), 16)
            if a == 0x62400C:
                if mnem == "fild":
                    assert pending is not None, text
                    at, pending = pending, None
                else:
                    at = JUNK
            elif a == 0x62411C: at = PLANE
            elif 0x623F80 <= a < 0x623FB0: at = OUT1 + a - 0x623F80
            elif 0x6241AC <= a < 0x6241C0: at = OUT2 + a - 0x6241AC
            else: raise Exception(text)
            if not skipped and mnem == "fstp" and a in (0x623F80, 0x623F84):
                skipped = True; continue
        lines.append('"%s dword ptr [{w} + 0x%X]", // %s' % (mnem, at, " ".join(text.split())))
    assert skipped and pending is None
    return lines

def emit(name, doc, lines):
    print("/// %s" % doc)
    print("pub unsafe fn %s(w: *mut Work) {" % name)
    print("    unsafe {\n        core::arch::asm!(")
    for l in lines: print("            " + l)
    print("            w = in(reg) w,")
    print('            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,')
    print('            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,')
    print("            options(nostack),\n        );\n    }\n}\n")

print('''//! The x87 arithmetic of the clippers, generated from the listing by
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
''')
emit("cut_at_x", "`sub_44D247` from after its `fstp flt_623F80` (the edge is in the block already) to its `call`.", gen("44D247", 0, 0x44D49F, True))
emit("cut_at_y", "`sub_44D532`, likewise with `flt_623F84`.", gen("44D532", 0, 0x44D532 + (0x44D49F - 0x44D247), True))
emit("cut_at_depth", "`sub_44DB93` from 0x44DC37 to 0x44DD9A (`sub_44DEAA` has the same there).", gen("44DB93", 0x44DC37, 0x44DDA0, False))
