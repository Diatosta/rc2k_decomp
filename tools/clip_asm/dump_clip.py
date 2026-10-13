# Run in IDA (py_eval: set OUT to a scratch path, exec this file, call run()): writes the
# listing of the clippers' four functions for gen_clip.py. The output is derived from
# the game and stays out of the repository.
import json
import idautils, idc, ida_bytes

OUT = None


def run():
    dump = {}
    for t in (0x44D247, 0x44D532, 0x44DB93, 0x44DEAA):
        dump["%X" % t] = [
            ["%X" % h, idc.generate_disasm_line(h, 1), ida_bytes.get_bytes(h, idc.get_item_size(h)).hex()]
            for h in idautils.FuncItems(t)
        ]
    open(OUT, "w").write(json.dumps(dump))
