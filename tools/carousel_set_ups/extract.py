# Run in IDA (py_eval): set OUT to a scratch path, exec this file, call run().
# Writes, for each menu number the dispatcher scene::set_up_carousel (0x4904DD)
# handles, the function it calls and that function's instructions as stores
# ("store", address, value), the fill of eight dwords with -1 ("fill"), "ret", or
# ("other", text) for anything else.
import idautils, idc, json

DISPATCHER = 0x4904DD


def menus():
    found = {}
    pending = None
    for ea in idautils.FuncItems(DISPATCHER):
        m = idc.print_insn_mnem(ea)
        if m == "cmp":
            pending = idc.get_operand_value(ea, 1)
        elif m == "jz" and pending is not None:
            target = idc.get_operand_value(ea, 0)
            call = idc.get_operand_value(target, 0)
            found[pending] = call
            pending = None
    return found


def ops(start):
    out = []
    items = list(idautils.FuncItems(start))
    i = 0
    while i < len(items):
        ea = items[i]
        m = idc.print_insn_mnem(ea)
        text = idc.generate_disasm_line(ea, 1)
        if m == "mov" and idc.get_operand_type(ea, 0) == 2 and idc.get_operand_type(ea, 1) == 5:
            out.append(["store", idc.get_operand_value(ea, 0), idc.get_operand_value(ea, 1) & 0xFFFFFFFF])
        elif text.startswith("mov     eax, 0FFFFFFFFh") and i + 3 < len(items) \
                and idc.generate_disasm_line(items[i + 1], 1).startswith("mov     ecx, 8") \
                and idc.generate_disasm_line(items[i + 2], 1).startswith("lea     edi, dword_75A622") \
                and idc.generate_disasm_line(items[i + 3], 1).startswith("rep stosd"):
            out.append(["fill"])
            i += 3
        elif m == "retn":
            out.append(["ret"])
        else:
            out.append(["other", text])
        i += 1
    return out


def run():
    result = {}
    for menu, function in sorted(menus().items()):
        result[menu] = {"function": function, "ops": ops(function)}
    with open(OUT, "w") as f:
        json.dump(result, f, indent=1)
    return len(result)
