# IDAPython, run through the IDA MCP `py_eval`:
#   g = {'OUT': '<a scratch folder>/menu_tables.json'}
#   exec(open('<repo>/tools/menu_tables/extract.py').read(), g); print(g['run']())
# Reads RAL.EXE's menus and its screens' draw lists into JSON for `gen.py`: every menu (a
# labels pointer, items of 0x5C bytes ended by a -1 dword; the menus found from their names
# in 0x747000 .. 0x755200 and from each end mark, as eight unnamed ones follow another's
# end), its labels (16 bytes each, ended by a -1 dword), the draw lists (dwords ended by -1,
# from their names in 0x758B00 .. 0x7594B0 and every place there something points at), and
# the names of every function and font they point at. The output is derived from the game's
# data: keep it out of the repository.
import idc, ida_name, ida_bytes, idautils, ida_funcs, json

MENU_AREA = (0x747000, 0x755200)
LIST_AREA = (0x758B00, 0x7594B0)
ITEM_LEN, LABEL_LEN, END = 0x5C, 0x10, 0xFFFFFFFF
FUNCTION_FIELDS = (0x34, 0x38, 0x3C, 0x40, 0x44)
FONT_FIELDS = (0x18, 0x4C, 0x50)


def dword(ea):
    return ida_bytes.get_dword(ea)


def is_code(v):
    return 0x401000 <= v < 0x4DA000


def is_menu(ea):
    first = dword(ea)
    if not (first == 0 or MENU_AREA[0] <= first < MENU_AREA[1]):
        return False
    return all(is_code(dword(ea + 4 + o)) for o in FUNCTION_FIELDS)


def heads(lo, hi):
    ea = lo
    while ea < hi:
        yield ea
        nx = idc.next_head(ea, hi)
        if nx == idc.BADADDR:
            return
        ea = nx


def menus():
    todo = [ea for ea in heads(*MENU_AREA) if ida_name.get_name(ea) and is_menu(ea)]
    found = {}
    while todo:
        ea = todo.pop()
        if ea in found:
            continue
        items, p = [], ea + 4
        while dword(p) != END:
            items.append([dword(p + 4 * k) for k in range(ITEM_LEN // 4)])
            p += ITEM_LEN
            if len(items) > 100:
                raise Exception('menu %x: no end within 100 items' % ea)
        labels_at, labels = dword(ea), []
        if labels_at:
            q = labels_at
            while dword(q) != END:
                labels.append([dword(q + 4 * k) for k in range(LABEL_LEN // 4)])
                q += LABEL_LEN
                if len(labels) > 100:
                    raise Exception('labels %x: no end within 100' % labels_at)
        found[ea] = {
            'at': ea,
            'name': ida_name.get_name(ea) or '',
            'referenced': any(True for _ in idautils.XrefsTo(ea, 0)),
            'labels_at': labels_at,
            'labels': labels,
            'items': items,
        }
        if is_menu(p + 4):
            todo.append(p + 4)
    return [found[k] for k in sorted(found)]


def list_starts():
    """Every place in the list area something points at, named or not (most screens'
    lists have no name), and the named ones."""
    starts = set()
    for ea in range(*LIST_AREA):
        if ida_name.get_name(ea) or any(True for _ in idautils.XrefsTo(ea, 0)):
            starts.add(ea)
    # lists nothing points at: functions right after another list's end mark (or marks)
    for ea in sorted(starts):
        p = ea
        while p < LIST_AREA[1] and is_code(dword(p)):
            p += 4
        while p < LIST_AREA[1] and dword(p) == END:
            while p < LIST_AREA[1] and dword(p) == END:
                p += 4
            if not is_code(dword(p)):
                break
            starts.add(p)
            while p < LIST_AREA[1] and is_code(dword(p)):
                p += 4
    return sorted(starts)


def lists():
    out = []
    for ea in list_starts():
        name = ida_name.get_name(ea) or ('list_%X' % ea)
        fns, p = [], ea
        while dword(p) != END:
            if not is_code(dword(p)):
                fns = None
                break
            fns.append(dword(p))
            p += 4
        if fns is not None:
            out.append({
                'at': ea,
                'name': name,
                'referenced': any(True for _ in idautils.XrefsTo(ea, 0)),
                'functions': fns,
            })
    return out


def run():
    ms, ls = menus(), lists()
    functions, fonts = set(), set()
    for m in ms:
        for it in m['items']:
            functions.update(it[o // 4] for o in FUNCTION_FIELDS)
            fonts.update(it[o // 4] for o in FONT_FIELDS)
        fonts.update(l[2] for l in m['labels'])
    for l in ls:
        functions.update(l['functions'])
    # the fonts `menu::disable_item` / `enable_item` put in place
    fonts.update([0x51EA04, 0x51E8E4])
    names = {}
    for f in sorted(functions):
        fn = ida_funcs.get_func(f)
        if not fn or fn.start_ea != f:
            raise Exception('%x is not a function start' % f)
        names[f] = idc.get_func_name(f)
    out = {
        'menus': ms,
        'lists': ls,
        'functions': names,
        'fonts': {f: ida_name.get_name(f) or ('font_%X' % f) for f in sorted(fonts)},
    }
    json.dump(out, open(OUT, 'w'), indent=1)
    return '%d menus (%d items, %d unreferenced), %d lists, %d functions, %d fonts' % (
        len(ms), sum(len(m['items']) for m in ms), sum(not m['referenced'] for m in ms),
        len(ls), len(names), len(fonts))
