#!/usr/bin/env python3
"""Writes the hook file of a screen loop from the extraction.

  gen_loops.py <loops.json> <tokens.json> <repo> <ADDRESS>:<name>:<what it is> ...

Each loop becomes src/menu/<name>.rs (a `Screen`, its `Original`, the hook) and is added to
src/menu/mod.rs. A place whose steps are more than calls and stores becomes a `Step::Do`
function. The files it writes are new: stage them with `git add`.
"""
import json, re, sys, textwrap

PTR = {0x7554F4: "g_unreadList", 0x7554F8: "g_drawList", 0x7554FC: "g_overlayList",
       0x755500: "dword_755500", 0x755504: "g_sceneList"}
SIMPLE = {'call', 'callfatal', 'wr'}
PLACES = ('before', 'set_up_first', 'prepare', 'after_lists', 'background', 'after_first_frame', 'after',
          'before_asking', 'leaving', 'after_restored')
QUIT = 0x51837C  # the quit switch: read through the Env, which the self-test redirects


def simple(o):
    return all(op[0] in SIMPLE or (op[0] in ('ri', 'ra') and op[1] == 'eax') for op in o)


def step(op):
    k = op[0]
    if k == 'call': return f"Step::Call(rva(0x{op[1]:X}))"
    if k == 'wr': return f"Step::Write(rva(0x{op[1]:X}), 0x{op[2]:X})"
    if k == 'ri': return f"Step::Eax(0x{op[2]:X})"
    if k == 'ra': return f"Step::EaxAddr(rva(0x{op[2]:X}))"
    if k == 'callfatal': return f"Step::CallOrFatal(rva(0x{op[1]:X}), 0x{op[2]:X})"
    raise SystemExit("step " + k)


def lhs_of(what):
    if what[0] == 'r': return f"g.{what[1]}"
    if what[1] == QUIT: return "env.quitting() as u32"
    return f"game_read::<u32>(rva(0x{what[1]:X}))"


def code(o, ind):
    out = []; p = " " * ind
    for op in o:
        k = op[0]
        if k == 'rtab': out.append(f"{p}g.{op[1]} = read_at((game_addr(rva(0x{op[2]:X})) as u32).wrapping_add(g.{op[3]}.wrapping_mul(4)));"); continue
        if k == 'movsb': out.append(f"{p}copy_bytes(g);"); continue
        if k == 'rmul': out.append(f"{p}g.{op[1]} = g.{op[1]}.wrapping_mul(0x{op[2]:X});"); continue
        if k == 'raddv': out.append(f"{p}g.{op[1]} = g.{op[1]}.wrapping_add(game_read(rva(0x{op[2]:X})));"); continue
        if k == 'call': out.append(f"{p}env.call(rva(0x{op[1]:X}), g);")
        elif k == 'callfatal': out.append(f"{p}call_or_fatal(rva(0x{op[1]:X}), 0x{op[2]:X}, g, env);")
        elif k == 'wr': out.append(f"{p}game_write(rva(0x{op[1]:X}), 0x{op[2]:X}u32);")
        elif k == 'ri': out.append(f"{p}g.{op[1]} = 0x{op[2]:X};")
        elif k == 'ra': out.append(f"{p}g.{op[1]} = game_addr(rva(0x{op[2]:X})) as u32;")
        elif k == 'rv': out.append(f"{p}g.{op[1]} = game_read(rva(0x{op[2]:X}));")
        elif k == 'vr': out.append(f"{p}game_write(rva(0x{op[1]:X}), g.{op[2]});")
        elif k == 'rr': out.append(f"{p}g.{op[1]} = g.{op[2]};")
        elif k == 'radd': out.append(f"{p}g.{op[1]} = g.{op[1]}.wrapping_add(0x{op[2]:X});")
        elif k == 'mr': out.append(f"{p}write_at(g.{op[1]}.wrapping_add(0x{op[2]:X}), g.{op[3]});")
        elif k == 'rm': out.append(f"{p}g.{op[1]} = read_at(g.{op[2]}.wrapping_add(0x{op[3]:X}));")
        elif k == 'pusha': out.append(f"{p}pusha(g, &mut all);")
        elif k == 'popa': out.append(f"{p}popa(g, &mut all);")
        elif k == 'push': out.append(f"{p}stack.push(g.{op[1]});")
        elif k == 'pop': out.append(f"{p}g.{op[1]} = stack.pop().expect(\"a pop without its push\");")
        elif k == 'pushv': out.append(f"{p}stack.push(game_read(rva(0x{op[1]:X})));")
        elif k == 'popv': out.append(f"{p}game_write(rva(0x{op[1]:X}), stack.pop().expect(\"a pop without its push\"));")
        elif k == 'itof': out.append(f"{p}game_write(rva(0x{op[2]:X}), game_read::<i32>(rva(0x{op[1]:X})) as f32);")
        elif k == 'goto': out.append(f"{p}return Flow::{op[1]};")
        elif k == 'ifgoto':
            lhs = lhs_of(op[1])
            out.append(f"{p}if {lhs} {op[2]} 0x{op[3]:X} {{"); out.append(f"{p}    return Flow::{op[4]};"); out.append(f"{p}}}")
        elif k == 'ifnc':
            out.append(f"{p}if !g.carry() {{"); out += code(op[1], ind + 4); out.append(f"{p}}}")
        elif k == 'if' and len(op[4]) == 1 and op[4][0][0] == 'ifgoto':
            # `cmp; jnz L; cmp; jnz there; L:` as one condition
            inner = op[4][0]
            out.append(f"{p}if {lhs_of(op[1])} {op[2]} 0x{op[3]:X} && {lhs_of(inner[1])} {inner[2]} 0x{inner[3]:X} {{")
            out.append(f"{p}    return Flow::{inner[4]};"); out.append(f"{p}}}")
        elif k == 'if':
            lhs = lhs_of(op[1])
            out.append(f"{p}if {lhs} {op[2]} 0x{op[3]:X} {{"); out += code(op[4], ind + 4); out.append(f"{p}}}")
        else: raise SystemExit("op " + k)
    return out


def flat(o):
    for op in o:
        yield op
        if op[0] == 'if': yield from flat(op[4])
        if op[0] == 'ifnc': yield from flat(op[1])


def lists(ls):
    return "&[\n" + "".join(
        f"        List {{\n            pointer: rva(0x{p:X}), // {PTR[p]}\n            table: rva(0x{t:X}),\n"
        f"            run: {'true' if run else 'false'},\n        }},\n" for p, t, run in ls) + "    ]"


def gen(r, name, what, note):
    assert not r['why'], r['why']
    fl = [l for l in r['lists'] if not l[2]]
    assert r['frame_lists'] == fl
    fns = []; uses = set()

    def place(key, ops):
        if simple(ops):
            if not ops: return "&[]"
            return "&[\n" + "".join(f"        {step(op)},\n" for op in ops) + "    ]"
        body = code(ops, 8); kinds = {op[0] for op in flat(ops)}
        reads = any(op[0] in ('if', 'ifgoto') and op[1][0] == 'v' and op[1][1] != QUIT for op in flat(ops))
        quits = any(op[0] in ('if', 'ifgoto') and op[1] == ['v', QUIT] for op in flat(ops))
        flows = bool(kinds & {'goto', 'ifgoto'})
        assert not flows or key in ('each_frame', 'after_first_frame'), "a jump out of " + key
        pre = []
        if kinds & {'pusha', 'popa'}: pre.append("        let mut all = Vec::new();"); uses.update(('pusha', 'popa'))
        if kinds & {'push', 'pop', 'pushv', 'popv'}: pre.append("        let mut stack: Vec<u32> = Vec::new();")
        for k, u in (('callfatal', 'call_or_fatal'), ('mr', 'write_at'), ('rm', 'read_at'), ('rtab', 'read_at'), ('movsb', 'copy_bytes')):
            if k in kinds: uses.add(u)
        if kinds & {'wr', 'vr', 'popv', 'itof'}: uses.add('game_write')
        if kinds & {'rv', 'pushv', 'itof', 'raddv'} or reads: uses.add('game_read')
        if kinds & {'ra', 'rtab'}: uses.add('game_addr')
        envname = "env" if kinds & {'call', 'callfatal'} or quits else "_env"
        gname = "g" if any("g." in l or "(g" in l or ", g)" in l for l in pre + body) else "_g"
        head = f"unsafe fn {key}({gname}: &mut Regs, {envname}: &mut dyn Env){' -> Flow' if flows else ''} {{\n"
        last = ["        Flow::On"] if flows and not (ops and ops[-1][0] == 'goto') else []
        if flows: uses.add('Flow')
        if kinds & {'wr', 'vr', 'rv', 'popv', 'pushv', 'itof', 'callfatal', 'mr', 'rm', 'rtab', 'movsb', 'raddv'} or reads:
            fns.append(head + "    unsafe {\n" + "\n".join(pre + body + last) + "\n    }\n}\n")
        else:
            fns.append(head + "\n".join(l[4:] for l in pre + body + last) + "\n}\n")
        return f"&[Step::{'DoFlow' if flows else 'Do'}({key})]"

    P = {k: place(k, r.get(k) or []) for k in PLACES}
    EF = place('each_frame', r['each_frame'])
    confirm = "None" if r['confirm'] is None else "Some(" + place('on_refusal', r['confirm']) + ")"
    confirmed = place('confirmed', r.get('confirmed') or [])
    also_if = f"Some(rva(0x{r['ask_also_if']:X}))" if r.get('ask_also_if') else "None"
    cars = sorted({op[2] for k in PLACES + ('each_frame',) for op in flat(r.get(k) or []) if op[0] == 'rtab'})
    if cars: uses.add('game_read')
    car_check = "".join(f'''    // The screen reads through this table's entries: without them the original would trap.
    if (0..4).any(|n| unsafe {{ game_read::<u32>(rva(0x{c:X}) + n * 4) }} == 0) {{
        return t.skip("a player is missing");
    }}
''' for c in cars)
    if r.get('result_tail'):
        fns.append("/// What the loop returns in eax. Written by hand from the end of the listing:\n"
                   + "".join(f"///   {l}\n" for l in r['result_tail']) + "unsafe fn result() -> u32 {\n    todo!()\n}\n")
    first = "None" if r['first_lists'] == fl else "Some(" + lists(r['first_lists']) + ")"
    later = "None" if r['cb_later'] in (None, r['cb']) else f"Some(rva(0x{r['cb_later']:X}))"
    lao = f"Some(rva(0x{r['leave_at_once']:X}))" if r['leave_at_once'] else "None"
    also = "&[" + ", ".join(f"rva(0x{w:X})" for w in r['writes']) + "]" if fns else "&[]"
    bg = [v.split('\\')[-1] for v in r['strings'].values() if v.lower().endswith('.png')]
    parts = []
    if bg: parts.append(f"background `{bg[0]}`")
    parts.append("leaving is asked about" if r['confirm'] is not None else "leaving is not asked about")
    if r.get('ask_also_if'): parts.append(f"also when `dword_{r['ask_also_if']:X}` is 1")
    if r.get('skips_save'): parts.append("left without the question, the highlight is not saved")
    if r['leave_at_once']: parts.append(f"it returns at once on leaving if `dword_{r['leave_at_once']:X}` is 1")
    if first != "None": parts.append("its own 3D list goes to `g_sceneList` before the first frame only")
    if r['escape_leaves']: parts.append("the Escape flag is looked at at the top of every frame")
    if r['each_frame']: parts.append("it has a step of its own once a frame")
    if r.get('next_first'): parts.append("the next frame begins with the steps after the first frame")
    if r.get('action_saved'): parts.append(f"around an action the highlight is kept in `dword_{r['action_saved']:X}`")
    if r.get('action_menu'): parts.append(f"the action run is that of the item of another menu, 0x{r['action_menu']:X} (the original's slip)")
    if r.get('before_asking'): parts.append("something runs before the question is put")
    if r.get('ask_unless'): parts.append(f"the question is not put while `dword_{r['ask_unless']:X}` is 1")
    if r.get('leaving'): parts.append("it has steps of its own first on leaving")
    if r.get('leave_saved'): parts.append(f"on leaving the highlight is saved in `dword_{r['leave_saved']:X}`, not in the variable it is taken from (the original's slip)")
    if r.get('keys_menu'): parts.append(f"the keys are handled with another menu, 0x{r['keys_menu']:X}, in `dword_731BBB` (the original's slip)")
    if r.get('after_restored'): parts.append("one call comes after the registers are put back")
    if r.get('result_tail'): parts.append("it returns a value in eax")
    if r.get('no_quit'): parts.append("the quit switch is not looked at")
    if r['short_check']: parts.append("after an action it is left or set up again, nothing else")
    if fns: parts.append("the steps that are more than a call or a store are the functions below, generated from the listing")
    callers = ", ".join(f"`{c}`" for c in r['callers']) or "none in code: it is reached through a table"
    relocs = sorted(r['calls'] + r['jumps_out'])
    ea = r['ea']
    if r['needs_copy']:
        hook = (f'    RawHook::new("menu::{name}", rva(0x{ea:X}), thunk, &ORIGINAL)\n        .private_copy(0x{r["len"]:X})\n'
                f'        .relocate_calls(&[{", ".join("0x%X" % x for x in relocs)}])\n        .manual_patch()\n        .self_test(self_test);')
        copy_doc = (f" Manual patch with a private copy (0x{r['len']:X} bytes, {len(relocs)} calls and jumps relocated): "
                    "a call or a jump target sits in the first bytes.")
    else:
        hook = f'    RawHook::new("menu::{name}", rva(0x{ea:X}), thunk, &ORIGINAL).self_test(self_test);'
        copy_doc = ""
    mem = {'game_read', 'game_write', 'game_addr'}
    fw = sorted({'RawHook', 'Regs', 'SelfTest', 'hook_asm', 'rva'} | (uses & mem))
    sl = sorted({'List', 'Original', 'Screen', 'Step', 'check_screen', 'dispatch_screen'} | (uses - mem) | ({'Env'} if fns else set()))
    head = [f"`menu::{name}` (0x{ea:X}): {what}", "", "In:  nothing", ("Out: eax = the result; every other register preserved" if r.get('result_tail') else "Out: every register preserved"),
            "The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as extracted "
            "from the disassembly: " + "; ".join(parts) + ".", note,
            f"Callers: {callers}. Ungated thunk; shadow mode runs the original alone; each real call writes a line to "
            "hook.log. Self-test: `screen_loop::check_screen`." + copy_doc]
    doc = []
    for h in head:
        if h in ("", "In:  nothing") or h.startswith("Out: "): doc.append("//!" + (" " + h if h else ""))
        else: doc += ["//! " + w for w in textwrap.wrap(h, 92, break_long_words=False, break_on_hyphens=False)]
    return "\n".join(doc) + f'''

use crate::framework::{{{", ".join(fw)}}};
use crate::menu::screen_loop::{{{", ".join(sl)}}};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
{hook}

hook_asm!(thunk, call_original, dispatch = dispatch, original = ORIGINAL, ungated);

// ---- Rust version: `menu::screen_loop` with this ----

{"".join(f + chr(10) for f in fns)}pub const SCREEN: Screen = Screen {{
    menu: rva(0x{r['menu']:X}),
    saved_item: rva(0x{r['saved']:X}),
    lists: {lists(r['lists'])},
    first_frame_lists: {first},
    frame_callback: rva(0x{r['cb']:X}),
    later_frame_callback: {later},
    before: {P['before']},
    set_up_first: {P['set_up_first']},
    prepare: {P['prepare']},
    after_lists: {P['after_lists']},
    background: {P['background']},
    after_first_frame: {P['after_first_frame']},
    confirm: {confirm},
    ask_also_if: {also_if},
    confirmed: {confirmed},
    unasked_skips_save: {"true" if r.get('skips_save') else "false"},
    after: {P['after']},
    escape_leaves: {"true" if r['escape_leaves'] else "false"},
    each_frame: {EF},
    short_action_check: {"true" if r['short_check'] else "false"},
    leave_at_once: {lao},
    next_frame_repeats_first: {"true" if r.get('next_first') else "false"},
    action_saved_item: {f"Some(rva(0x{r['action_saved']:X}))" if r.get('action_saved') else "None"},
    action_menu: {f"Some(rva(0x{r['action_menu']:X}))" if r.get('action_menu') else "None"},
    no_quit_check: {"true" if r.get('no_quit') else "false"},
    before_asking: {P['before_asking']},
    ask_unless: {f"Some(rva(0x{r['ask_unless']:X}))" if r.get('ask_unless') else "None"},
    leaving: {P['leaving']},
    leave_saved_item: {f"Some(rva(0x{r['leave_saved']:X}))" if r.get('leave_saved') else "None"},
    keys_menu: {f"Some(rva(0x{r['keys_menu']:X}))" if r.get('keys_menu') else "None"},
    after_restored: {P['after_restored']},
    result: {"Some(result)" if r.get('result_tail') else "None"},
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
    also_reads: &[],
    also_writes: {also},
}};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {{
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {{
    function: rva(0x{ea:X}),
    len: 0x{r['len']:X},
    entry_bytes: &[{", ".join("0x%02X" % b for b in bytes.fromhex(r['first8']))}],
    calls: &[{", ".join("0x%X" % x for x in r['calls'])}],
}};

fn self_test() -> (usize, usize) {{
    let Some(mut t) = SelfTest::begin(&HOOK) else {{
        return (0, 0);
    }};
{car_check}    if let Err(why) = check_screen(&mut t, &SCREEN, &THE_ORIGINAL) {{
        return t.skip(why);
    }}
    t.finish()
}}
'''


def register(repo, names):
    p = repo + "/src/menu/mod.rs"; s = open(p).read()
    have = re.findall(r"^pub mod (\w+);", s, flags=re.M)
    first = s.index("pub mod "); last = s.rindex("pub mod "); end = s.index("\n", last) + 1
    s = s[:first] + "".join(f"pub mod {m};\n" for m in sorted(set(have + names))) + s[end:]
    k = s.index("];", s.index("pub static HOOKS"))
    s = s[:k] + "".join(f"    &{n}::HOOK,\n" for n in names if f"&{n}::HOOK" not in s) + s[k:]
    open(p, "w").write(s)


if __name__ == "__main__":
    loops = {r['ea']: r for r in json.load(open(sys.argv[1]))}
    tokens = json.load(open(sys.argv[2])); repo = sys.argv[3]; names = []
    for spec in sys.argv[4:]:
        ea, name, what = spec.split(":", 2); r = loops[int(ea, 16)]
        t = r.get('title'); title = tokens[t][1] if t is not None and t < len(tokens) else ""
        note = (f'The name is a reading from the title string of its menu, "{title}", not yet a visit.' if title
                else "The name is a reading from its background and callers, not yet a visit.")
        open(f"{repo}/src/menu/{name}.rs", "w").write(gen(r, name, what, note)); names.append(name)
        print(f"src/menu/{name}.rs")
    register(repo, names)
