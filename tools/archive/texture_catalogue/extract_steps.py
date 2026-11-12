# Run in IDA (through py_eval: exec the file, then call run(out_path)).
# Reads scene::build_texture_catalogue (0x42274B) and writes its steps as JSON: the
# function is one long list of the same few moves, which src/scene/texture_steps.rs
# (made by gen_steps.py from that JSON) holds as a table.
#
# The moves, as the listing has them:
#   log text / log number      pusha; lea eax, text | mov eax, n; call log_*; popa
#   count                      mov eax, n; cmp eax, -1; jz +; add dword_5DC97C[eax*4], 1
#   raise                      pusha; mov ebx, tag; mov eax, level; call raise_texture_needs; popa
#   try                        pusha; call try_allocate_textures; popa; jnb done
#   release                    call release_all_textures
#   unless flag                cmp dword flag, 0; jnz past  (the moves up to `past`)
import json
import idautils, idc, ida_funcs

START = 0x42274B
LOGS = {
    "logfile::log_write_l1": ("text", False),
    "logfile::log_write_l1_nl": ("text", True),
    "logfile::log_int_l1": ("number", False),
    "logfile::log_int_l1_nl": ("number", True),
}

def run(out_path):
    f = ida_funcs.get_func(START)
    items = [ea for ea in idautils.FuncItems(START) if f.start_ea <= ea < f.end_ea]
    mnem = lambda ea: idc.print_insn_mnem(ea)
    steps, unknown = [], []
    guards = []  # (end address, list being filled)
    out = steps
    last_eax = None   # ("imm", n) or ("text", address)
    last_ebx = None
    i = 0
    while i < len(items):
        ea = items[i]
        while guards and ea >= guards[-1][0]:
            guards.pop()
            out = guards[-1][1] if guards else steps
        m = mnem(ea)
        op0, op1 = idc.print_operand(ea, 0), idc.print_operand(ea, 1)
        if m in ("pusha", "popa"):
            pass
        elif m == "lea" and op0 == "eax":
            last_eax = ("text", idc.get_operand_value(ea, 1))
        elif m == "mov" and op0 == "eax" and idc.get_operand_type(ea, 1) == 5:
            last_eax = ("imm", idc.get_operand_value(ea, 1) & 0xFFFFFFFF)
        elif m == "mov" and op0 == "ebx" and idc.get_operand_type(ea, 1) == 5:
            last_ebx = idc.get_operand_value(ea, 1) & 0xFFFFFFFF
        elif m == "call":
            name = idc.get_func_name(idc.get_operand_value(ea, 0))
            if name in LOGS:
                kind, newline = LOGS[name]
                if kind == "text":
                    text = idc.get_strlit_contents(last_eax[1], -1, 0).decode("latin-1")
                    out.append({"op": "text", "text": text, "newline": newline})
                else:
                    out.append({"op": "number", "value": last_eax[1], "newline": newline})
            elif name == "scene::raise_texture_needs":
                out.append({"op": "raise", "level": last_eax[1], "tag": last_ebx})
            elif name == "scene::count_texture_needs":
                out.append({"op": "count_needs"})
            elif idc.get_operand_value(ea, 0) == 0x4224C4:
                nxt = items[i + 1]
                if mnem(nxt) == "popa":
                    nxt = items[i + 2]; i += 1
                assert mnem(nxt) == "jnb", hex(nxt)
                out.append({"op": "try"})
                i += 1
            elif idc.get_operand_value(ea, 0) == 0x4051D3:
                out.append({"op": "release"})
            else:
                unknown.append("%X call %s" % (ea, name))
        elif m == "cmp" and op0 == "eax" and op1 == "0FFFFFFFFh":
            # count: cmp eax, -1; jz +; add table[eax*4], 1
            assert mnem(items[i + 1]) == "jz" and mnem(items[i + 2]) == "add", hex(ea)
            table = idc.get_operand_value(items[i + 2], 0)
            if last_eax[1] != 0xFFFFFFFF:
                out.append({"op": "count", "index": last_eax[1], "table": table})
            i += 2
        elif m == "cmp" and idc.get_operand_type(ea, 0) == 2 and op1 == "0":
            nxt = items[i + 1]
            assert mnem(nxt) == "jnz", hex(ea)
            inner = []
            out.append({"op": "unless", "flag": idc.get_operand_value(ea, 0), "steps": inner})
            guards.append((idc.get_operand_value(nxt, 0), inner))
            out = inner
            i += 1
        else:
            unknown.append("%X %s" % (ea, idc.generate_disasm_line(ea, 0)))
        i += 1
    # Every call of the function, as an offset from its start: for the self-test's copy.
    calls = [ea - START for ea in items if mnem(ea) == "call"]
    json.dump({"steps": steps, "unknown": unknown, "calls": calls}, open(out_path, "w"), indent=1)
    return len(steps), unknown
