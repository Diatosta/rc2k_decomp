# IDAPython, run through the IDA MCP `py_eval`:
#   g = {'OUT': '<a scratch folder>/loops.json'}
#   exec(open('<repo>/tools/screen_loops/extract_loops.py').read(), g); print(g['run']())
# Cuts the game's 83 screen loops at fixed anchors into the places of a `Screen`
# (src/menu/screen_loop.rs), turns each place's instructions into ops, compares the
# frame-and-action part with the service area's, and writes one record per loop with the
# reasons ('why') it does not fit the description, if any. The output is derived from the
# game's code: keep it out of the repository.
import idautils, idc, ida_lines, ida_funcs, ida_bytes, re, json, collections
T = lambda h: ' '.join(ida_lines.tag_remove(idc.generate_disasm_line(h, 0)).split()).split(';')[0].strip()
LISTP = (0x7554F4, 0x7554F8, 0x7554FC, 0x755500, 0x755504)
KNOWN = {0x758806, 0x755525, 0x755A3C, 0x755561, 0x75558D, 0x7556A4, 0x5189A0, 0x613830, 0x731BBB} | set(LISTP)
REGS = ('eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'ebp')

def find_loops():
    loops = set()
    for a in LISTP:
        for x in idautils.XrefsTo(a):
            if idc.print_insn_mnem(x.frm) == 'mov' and idc.get_operand_value(x.frm, 0) == a:
                f = ida_funcs.get_func(x.frm)
                if f: loops.add(f.start_ea)
    return sorted(loops)

def ops(H, lo, hi, ea, end, why, roles=None):
    """Instructions H[lo:hi] as a list of ops; conditionals become nested ('if', ...).
    With `roles` (label address -> 'LOOP', 'LEAVE', 'RESTART'), jumps to those labels
    become ('goto', role) and ('ifgoto', what, cmp, value, role)."""
    out = []; i = lo
    idx = {h: n for n, h in enumerate(H)}
    roles = roles or {}
    while i < hi:
        h = H[i]; m = idc.print_insn_mnem(h); t0 = idc.get_operand_type(h, 0); t1 = idc.get_operand_type(h, 1)
        p0 = idc.print_operand(h, 0); p1 = idc.print_operand(h, 1)
        v0 = idc.get_operand_value(h, 0); v1 = idc.get_operand_value(h, 1) & 0xFFFFFFFF
        if m == 'call' and t0 in (6, 7):
            if i + 1 < len(H) and idc.print_insn_mnem(H[i + 1]) == 'jb':
                jt = idc.get_operand_value(H[i + 1], 0)
                if jt in idx and i + 2 <= idx[jt] <= hi:
                    # a carry from the call skips what follows, up to a label of this place
                    out.append(['call', v0]); out.append(['ifnc', ops(H, i + 2, idx[jt], ea, end, why, roles)])
                    i = idx[jt]; continue
                if not (ea <= jt < end) and idc.print_insn_mnem(jt) == 'mov' and idc.get_operand_value(jt, 0) == 0x4F52A4:
                    out.append(['callfatal', v0, idc.get_operand_value(jt, 1) & 0xFFFFFFFF]); i += 2; continue
                why.append('jb after call at %X is not to a fatal stub' % h); return out
            out.append(['call', v0])
        elif m == 'mov' and t0 == 2 and t1 == 5: out.append(['wr', v0, v1])
        elif m == 'mov' and p0 in REGS and re.match(r'^\w+\[(e[a-z][a-z])\*4\]$', p1):
            out.append(['rtab', p0, v1, re.match(r'^\w+\[(e[a-z][a-z])\*4\]$', p1).group(1)])
        elif m == 'lea' and p0 in REGS and re.match(r'^\[(e[a-z][a-z])(?:\+([0-9A-F]+)h?)?\]$', p1):
            mm = re.match(r'^\[(e[a-z][a-z])(?:\+([0-9A-F]+)h?)?\]$', p1)
            if mm.group(1) != p0: out.append(['rr', p0, mm.group(1)])
            if mm.group(2): out.append(['radd', p0, int(mm.group(2), 16)])
        elif T(h) == 'rep movsb': out.append(['movsb'])
        elif re.match(r'^imul (e[a-z][a-z]), ([0-9A-F]+)h?$', T(h)):
            mm = re.match(r'^imul (e[a-z][a-z]), ([0-9A-F]+)h?$', T(h)); out.append(['rmul', mm.group(1), int(mm.group(2), 16)])
        elif m == 'add' and t0 == 1 and p0 in REGS and t1 == 2: out.append(['raddv', p0, v1])
        elif m == 'mov' and t0 == 1 and p0 in REGS and t1 == 5: out.append(['ri', p0, v1])
        elif m == 'xor' and p0 == p1 and p0 in REGS: out.append(['ri', p0, 0])
        elif m == 'mov' and t0 == 1 and p0 in REGS and t1 == 2: out.append(['rv', p0, v1])
        elif m == 'mov' and t0 == 2 and t1 == 1 and p1 in REGS: out.append(['vr', v0, p1])
        elif m == 'mov' and t0 == 1 and t1 == 1 and p0 in REGS and p1 in REGS: out.append(['rr', p0, p1])
        elif m == 'lea' and p0 in REGS and t1 == 2: out.append(['ra', p0, v1])
        elif m in ('add', 'sub') and t0 == 1 and p0 in REGS and t1 == 5: out.append(['radd', p0, v1 if m == 'add' else (-v1) & 0xFFFFFFFF])
        elif m in ('inc', 'dec') and t0 == 1 and p0 in REGS: out.append(['radd', p0, 1 if m == 'inc' else 0xFFFFFFFF])
        elif m == 'mov' and t0 in (3, 4) and t1 == 1 and p1 in REGS:
            mm = re.match(r'^\[(e[a-z][a-z])(?:\+([0-9A-F]+)h?)?\]$', p0)
            if not mm: why.append('op %X: %s' % (h, T(h))); return out
            out.append(['mr', mm.group(1), int(mm.group(2) or '0', 16), p1])
        elif m == 'mov' and t0 == 1 and p0 in REGS and t1 in (3, 4):
            mm = re.match(r'^\[(e[a-z][a-z])(?:\+([0-9A-F]+)h?)?\]$', p1)
            if not mm: why.append('op %X: %s' % (h, T(h))); return out
            out.append(['rm', p0, mm.group(1), int(mm.group(2) or '0', 16)])
        elif m == 'pusha': out.append(['pusha'])
        elif m == 'popa': out.append(['popa'])
        elif m == 'push' and p0 in REGS: out.append(['push', p0])
        elif m == 'pop' and p0 in REGS: out.append(['pop', p0])
        elif m == 'push' and t0 == 2: out.append(['pushv', v0])
        elif m == 'pop' and t0 == 2: out.append(['popv', v0])
        elif m == 'fild' and i + 1 < hi and idc.print_insn_mnem(H[i + 1]) == 'fstp':
            # IDA gives these a hidden st(0) operand first: the memory operand is the one of type 2
            def mem(hh):
                for n in (0, 1):
                    if idc.get_operand_type(hh, n) == 2: return idc.get_operand_value(hh, n)
            src, dst = mem(h), mem(H[i + 1])
            if src is None or dst is None or not T(h).startswith('fild dword_') or not T(H[i + 1]).startswith('fstp flt_'):
                why.append('op %X: %s' % (h, T(h))); return out
            out.append(['itof', src, dst]); i += 2; continue
        elif m == 'cmp' and t1 == 5 and (t0 == 2 or (t0 == 1 and p0 in REGS)) and i + 1 < hi and idc.print_insn_mnem(H[i + 1]) in ('jz', 'jnz'):
            jt = idc.get_operand_value(H[i + 1], 0)
            lhs = ['v', v0] if t0 == 2 else ['r', p0]
            if jt in roles and not (jt in idx and idx[jt] == hi):
                # jz goes there when equal
                out.append(['ifgoto', lhs, '==' if idc.print_insn_mnem(H[i + 1]) == 'jz' else '!=', v1, roles[jt]]); i += 2; continue
            if jt not in idx or not (i + 2 <= idx[jt] <= hi): why.append('conditional at %X leaves its place' % h); return out
            body = ops(H, i + 2, idx[jt], ea, end, why, roles)
            # jz skips the body when equal: the body runs when not equal
            out.append(['if', ['v', v0] if t0 == 2 else ['r', p0], '!=' if idc.print_insn_mnem(H[i + 1]) == 'jz' else '==', v1, body])
            i = idx[jt]; continue
        elif m == 'jmp' and v0 in roles: out.append(['goto', roles[v0]])
        else:
            why.append('op %X: %s' % (h, T(h))); return out
        i += 1
    return out

def writes_of(o, known=None):
    """Variables a list of ops writes; None in the set means a write through a register that is not known."""
    w = set(); known = dict(known or {}); imm = {}
    for op in o:
        k = op[0]
        if k == 'ri': imm[op[1]] = op[2]
        elif k in ('rv', 'rr', 'rm', 'pop', 'rtab', 'rmul', 'raddv', 'radd') and len(op) > 1: imm.pop(op[1], None)
        elif k in ('call', 'callfatal', 'popa'): imm.clear()
        if k == 'movsb':
            if 'edi' in known and 'ecx' in imm: w |= {(known['edi'] + n) & 0xFFFFFFFF for n in range(0, imm['ecx'], 4)}
            else: w.add(None)
            known.pop('edi', None); continue
        if k in ('rtab', 'rmul', 'raddv'): known.pop(op[1], None); continue
        if k in ('wr', 'vr', 'popv'): w.add(op[1])
        elif k == 'itof': w.add(op[2])
        elif k == 'ra': known[op[1]] = op[2]
        elif k == 'radd' and op[1] in known: known[op[1]] = (known[op[1]] + op[2]) & 0xFFFFFFFF
        elif k in ('ri', 'rv', 'rr', 'rm', 'pop'): known.pop(op[1], None)
        elif k in ('call', 'callfatal', 'popa'): known.clear()
        elif k == 'mr': w.add((known[op[1]] + op[2]) & 0xFFFFFFFF if op[1] in known else None)
        elif k == 'if': w |= writes_of(op[4], known); known.clear()
        elif k == 'ifnc': w |= writes_of(op[1], known); known.clear()
    return w

def parse(ea):
    end = [e for s, e in idautils.Chunks(ea) if s == ea][0]
    H = list(idautils.Heads(ea, end)); tx = [T(h) for h in H]
    why = []
    r = {'ea': ea, 'len': end - ea, 'first8': ida_bytes.get_bytes(ea, 8).hex(), 'why': why,
         'calls': [h - ea for h in H if ida_bytes.get_byte(h) == 0xE8],
         'callers': [idc.get_func_name(x) or ('%X' % x) for x in idautils.CodeRefsTo(ea, 0)]}
    def find(pred, start=0):
        for i in range(start, len(tx)):
            if pred(tx[i]): return i
        return None
    i_hl = find(lambda t: t == 'mov dword_755525, ebx')
    i_l0 = max([i for i, t in enumerate(tx) if t == 'mov dword_758806, 0' and i < (i_hl or 0)], default=None)
    i_menu = find(lambda t: t == 'mov dword_75558D, edi'); i_draw = find(lambda t: t == 'call menu__draw_frame')
    i_rf = find(lambda t: t == 'call screens__run_frame'); i_hk = find(lambda t: t == 'call menu__handle_keys')
    blinks = [i for i, t in enumerate(tx) if t == 'call menu__update_blink']
    if None in (i_l0, i_hl, i_menu, i_draw, i_rf, i_hk) or len(blinks) < 2 or tx[0] != 'pusha':
        why.append('anchors'); return r
    i_sound = find(lambda t: t.startswith('mov dword_7556A4, 6Fh'), i_menu)
    if i_sound is None or i_sound > i_draw or tx[i_sound + 1] != 'mov dword_755561, 0': why.append('sound anchor'); return r
    if not (tx[i_hl - 2].startswith('lea edi, ') and tx[i_hl - 1].startswith('mov ebx, ')): why.append('highlight anchor'); return r
    r['menu'] = idc.get_operand_value(H[i_hl - 2], 1); r['saved'] = idc.get_operand_value(H[i_hl - 1], 1)
    if i_hl - 2 != i_l0 + 1: why.append('between 758806=0 and the menu: %s' % tx[i_l0 + 1:i_hl - 2])
    targets = {}
    for h in H:
        if idc.print_insn_mnem(h).startswith('j'):
            t = idc.get_operand_value(h, 0)
            if ea <= t < end: targets.setdefault(t, []).append(h)
    restart = None
    for i in range(i_l0, 0, -1):
        if H[i] in targets and any(src > H[blinks[1]] for src in targets[H[i]]): restart = i; break
    if restart is None: why.append('no restart label'); return r
    i_act = find(lambda t: t.startswith('cmp dword_7556A4, 14Dh'), blinks[1])
    i_keys = i_hk - 4; i_leave = i_hk + 7
    if [tx[i_hk + 1], tx[i_hk + 3], tx[i_hk + 5]] != ['cmp eax, 1', 'cmp eax, 2', 'cmp eax, 3']: why.append('after handle_keys'); return r
    roles = {H[restart]: 'RESTART', H[blinks[1]]: 'LOOP', H[i_act]: 'ACT', H[i_keys]: 'KEYS', H[i_leave]: 'LEAVE'}
    # "next frame" going to the steps after the first frame instead of past them
    top = H[i_rf + 1]
    r['next_first'] = top != H[blinks[1]] and any(src > H[blinks[1]] for src in targets.get(top, []))
    if r['next_first']: roles[top] = 'LOOP'
    # another variable for the highlight around the action
    r['action_saved'] = None
    k = find(lambda t: t == 'mov ecx, dword_755525', i_act)
    if k is not None and k < i_keys and re.match(r'^mov dword_[0-9A-F]+, ecx$', tx[k + 1]) and idc.get_operand_value(H[k + 1], 0) != r['saved']:
        r['action_saved'] = idc.get_operand_value(H[k + 1], 0)
    # every `lea edi` names the screen's menu, but for the two around the action
    r['action_menu'] = None
    r['keys_menu'] = None
    for k in range(restart, len(tx)):
        if tx[k].startswith('lea edi, ') and idc.get_operand_value(H[k], 1) != r['menu']:
            if k == i_hk - 3: r['keys_menu'] = idc.get_operand_value(H[k], 1)
            elif i_act < k < i_keys and r['action_menu'] in (None, idc.get_operand_value(H[k], 1)): r['action_menu'] = idc.get_operand_value(H[k], 1)
            else: why.append('another menu at %X' % H[k])
    # the other labels inside the frame-and-action part, by their order
    inner = sorted(t for t in targets if H[blinks[1]] <= t < H[i_leave] and t not in roles)
    def norm(i):
        t = tx[i]
        def lab(m):
            a = int(m.group(1), 16)
            if a in roles: return roles[a]
            if a in inner: return 'L%d' % inner.index(a)
            return 'OUT%X' % (a - ea)
        t = re.sub(r'loc_([0-9A-F]+)', lab, t)
        t = t.replace(idc.get_name(r['saved']) or '\0', 'SAVED')
        if r['action_saved']: t = t.replace(idc.get_name(r['action_saved']) or '\0', 'SAVED')
        t = re.sub(r'^lea (e[ds]i|eax), .*', r'lea \1, X', t)
        if re.match(r'^mov \S+, esi$', t) and idc.get_operand_value(H[i], 0) in LISTP: t = 'mov LISTP, esi'
        return t
    # The frame-and-action part, with what the options of `Screen` account for taken out:
    # a check of the Escape flag at the top of the frame, and steps of the screen's own
    # between the elapsed time and the action check.
    skip = set()
    r['escape_leaves'] = tx[blinks[1] + 1] == 'cmp dword_755A3C, 1' and tx[blinks[1] + 2].startswith('jz') and idc.get_operand_value(H[blinks[1] + 2], 0) == H[i_leave]
    if r['escape_leaves']: skip |= {blinks[1] + 1, blinks[1] + 2}
    q = blinks[1] + (3 if r['escape_leaves'] else 1)
    r['no_quit'] = tx[q] == 'mov dword_7556A4, 6Fh'
    if r['no_quit']: skip.add(q)
    i_el = find(lambda t: t == 'mov dword_613830, eax', blinks[1])
    r['each_frame'] = []
    if i_el is not None and i_el + 1 < i_act:
        r['each_frame'] = ops(H, i_el + 1, i_act, ea, end, why, {H[blinks[1]]: 'NextFrame', H[i_leave]: 'Leave', H[restart]: 'SetUp'})
        skip |= set(range(i_el + 1, i_act))
    inner[:] = [t for t in inner if H.index(t) not in skip]  # labels inside the screen's own steps do not count
    r['core'] = [norm(i) for i in range(blinks[1], i_leave) if i not in skip]
    if r['no_quit']: r['core'][1:1] = ['cmp dword_51837C, 1', 'jz LEAVE']
    r['before'] = ops(H, 1, restart, ea, end, why)
    r['set_up_first'] = ops(H, restart, i_l0, ea, end, why)
    ce = ['cmp g_clearsEscapeFlag, 1', None, 'cmp dword_755A3C, 1', None, 'mov dword_755A3C, 0', 'mov dword_755561, 0']
    def is_ce(i): return i + 6 <= len(tx) and all(c is None or tx[i + k] == c for k, c in enumerate(ce))
    if not is_ce(i_hl + 1): why.append('clear_escape after the highlight'); return r
    r['prepare'] = ops(H, i_hl + 7, i_menu - 1, ea, end, why)
    def lists(lo, hi):
        out = []; i = lo
        while i < hi:
            if not tx[i].startswith('lea esi, '): why.append('list at %X: %s' % (H[i], tx[i])); return out
            tab = idc.get_operand_value(H[i], 1); run = False; i += 1
            if tx[i] == 'call menu__run_list': run = True; i += 1
            if not (re.match(r'^mov \S+, esi$', tx[i]) and idc.get_operand_value(H[i], 0) in LISTP): why.append('list store at %X: %s' % (H[i], tx[i])); return out
            out.append([idc.get_operand_value(H[i], 0), tab, run]); i += 1
        return out
    i_al = i_menu + 1
    while i_al < i_sound and (tx[i_al].startswith('lea esi, ') or tx[i_al] == 'call menu__run_list' or (re.match(r'^mov \S+, esi$', tx[i_al]) and idc.get_operand_value(H[i_al], 0) in LISTP)): i_al += 1
    r['lists'] = lists(i_menu + 1, i_al)
    r['after_lists'] = ops(H, i_al, i_sound, ea, end, why)
    r['background'] = ops(H, i_sound + 2, i_draw - 1, ea, end, why)
    fixed = tx[i_draw + 1:i_draw + 5]
    if fixed != ['call menu__update_blink', 'mov eax, dword_51899C', 'mov dword_5189A0, eax', 'mov dword_613830, 0']: why.append('after draw_frame: %s' % fixed); return r
    if not tx[i_draw + 5].startswith('lea eax, '): why.append('callback'); return r
    r['cb'] = idc.get_operand_value(H[i_draw + 5], 1)
    if tx[i_draw + 7] != 'mov dword_75558D, edi': why.append('first frame menu'); return r
    r['first_lists'] = lists(i_draw + 8, i_rf)
    r['after_first_frame'] = ops(H, i_rf + 1, blinks[1], ea, end, why, {H[restart]: 'SetUp', H[i_leave]: 'Leave'})
    i_rf2 = find(lambda t: t == 'call screens__run_frame', i_rf + 1)
    i_m2 = find(lambda t: t == 'mov dword_75558D, edi', blinks[1])
    r['frame_lists'] = lists(i_m2 + 1, i_rf2)
    cb2 = [idc.get_operand_value(H[k], 1) for k in range(blinks[1], i_rf2) if tx[k].startswith('lea eax, ')]
    r['cb_later'] = cb2[0] if cb2 else None
    i = i_leave; r['leave_at_once'] = None; r['confirm'] = None
    if re.match(r'^cmp dword_[0-9A-F]+, 1$', tx[i]) and idc.get_operand_value(H[i], 0) != 0x758806 and tx[i + 1].startswith('jz') and T(idc.get_operand_value(H[i + 1], 0)) == 'popa':
        r['leave_at_once'] = idc.get_operand_value(H[i], 0); i += 2
    r['ask_also_if'] = None; r['confirmed'] = []; r['skips_save'] = False
    if (re.match(r'^cmp dword_[0-9A-F]+, 1$', tx[i]) and idc.get_operand_value(H[i], 0) != 0x758806 and tx[i + 1].startswith('jz')
            and tx[i + 2] == 'cmp dword_758806, 1' and idc.get_operand_value(H[i + 1], 0) == H[i + 4]):
        r['ask_also_if'] = idc.get_operand_value(H[i], 0); i += 2
    r['leaving'] = []; r['before_asking'] = []; r['ask_unless'] = None
    c = find(lambda t: t == 'call menu__confirm_leaving', i)
    if tx[i] == 'cmp dword_758806, 1' and not (tx[i + 1].startswith('jnz') and c is not None and c <= i + 8):
        # no question: steps of the screen's own before the highlight is saved
        ks = find(lambda t: t == 'mov ecx, dword_755525', i)
        if ks is None: why.append('confirm shape'); return r
        r['leaving'] = ops(H, i, ks, ea, end, why); i = ks
    elif tx[i] == 'cmp dword_758806, 1':
        save = idc.get_operand_value(H[i + 1], 0); k = i + 2
        if re.match(r'^cmp dword_[0-9A-F]+, 1$', tx[k]) and tx[k + 1].startswith('jz') and idc.get_operand_value(H[k + 1], 0) == save:
            r['ask_unless'] = idc.get_operand_value(H[k], 0); k += 2
        r['before_asking'] = ops(H, k, c, ea, end, why); i = c - 2
        if tx[i + 3] != 'cmp g_yesNoAnswer, 1': why.append('confirm shape'); return r
        j = H[i + 4]
        if tx[i + 4].startswith('jnz') and idc.get_operand_value(j, 0) == H[restart]:
            r['confirm'] = []; i += 5
        elif tx[i + 4].startswith('jz') and idc.get_operand_value(j, 0) == save:
            k = i + 5
            while k < len(tx) and not tx[k].startswith('jmp'): k += 1
            if idc.get_operand_value(H[k], 0) != H[restart]: why.append('a refusal does not go to set-up'); return r
            r['confirm'] = ops(H, i + 5, k, ea, end, why); i = k + 1
        else: why.append('confirm tail: %s' % tx[i + 4]); return r
        if H[i] != save:
            if save not in H or H.index(save) <= i: why.append('save label'); return r
            ks = H.index(save)
            if tx[ks] == 'mov ecx, dword_755525': r['confirmed'] = ops(H, i, ks, ea, end, why); i = ks
            elif ks == i + 8: r['skips_save'] = True
            else: why.append('save label'); return r
    if not (tx[i] == 'mov ecx, dword_755525' and re.match(r'^mov \S+, ecx$', tx[i + 1]) and is_ce(i + 2)):
        why.append('leave save: %s' % tx[i:i + 3]); return r
    r['leave_saved'] = idc.get_operand_value(H[i + 1], 0) if idc.get_operand_value(H[i + 1], 0) != r['saved'] else None
    # the end: popa and retn, or a call or a result in eax between the two
    pp = max(k for k, t in enumerate(tx) if t == 'popa')
    if tx[-1] != 'retn' or pp < i + 8: why.append('tail'); return r
    r['after'] = ops(H, i + 8, pp, ea, end, why)
    rest = tx[pp + 1:-1]; r['after_restored'] = []; r['result_tail'] = None
    if len(rest) == 1 and rest[0].startswith('call '): r['after_restored'] = ops(H, pp + 1, len(tx) - 1, ea, end, why)
    elif rest: r['result_tail'] = tx[pp + 1:]
    places = ('before', 'set_up_first', 'prepare', 'after_lists', 'background', 'after_first_frame', 'after', 'leaving', 'before_asking')
    own = set([r['saved']])
    if r['leave_saved']: own.add(r['leave_saved'])
    if r['action_saved']: own.add(r['action_saved'])
    for k in places: own |= writes_of(r[k])
    own |= writes_of(r['confirm'] or [])
    own |= writes_of(r['confirmed'])
    own |= writes_of(r['each_frame'])
    if None in own: why.append('a write through a register that is not known'); own.discard(None)
    r['writes'] = sorted(own - KNOWN - {r['saved']})
    extra = []
    for h in H:
        m = idc.print_insn_mnem(h)
        n = 1 if m == 'fstp' and idc.get_operand_type(h, 0) != 2 else 0
        if m in ('mov', 'fstp', 'inc', 'dec', 'add', 'sub', 'xor', 'or', 'and', 'xchg', 'pop') and idc.get_operand_type(h, n) == 2:
            a = idc.get_operand_value(h, n)
            if a not in KNOWN and a not in own: extra.append('%X' % a)
    if extra: why.append('unwatched writes: %s' % sorted(set(extra)))
    n = 0; k = 0
    while n < 5: n += idc.get_item_size(H[k]); k += 1
    r['needs_copy'] = any(ea < t < ea + n for t in targets) or any(ida_bytes.get_byte(H[q]) == 0xE8 for q in range(k))
    r['jumps_out'] = [h - ea for h in H if ida_bytes.get_byte(h) == 0x0F and 0x80 <= ida_bytes.get_byte(h + 1) <= 0x8F and not (ea <= idc.get_operand_value(h, 0) < end)]
    r['other_jumps_out'] = [h - ea for h in H if idc.print_insn_mnem(h).startswith('j') and not (ea <= idc.get_operand_value(h, 0) < end) and h - ea not in r['jumps_out']]
    r['indirect_calls'] = [h - ea for h in H if idc.print_insn_mnem(h) == 'call' and idc.get_operand_type(h, 0) not in (6, 7)]
    if r['indirect_calls']: why.append('indirect calls')
    if r['other_jumps_out']: why.append('other jumps out')
    def strs(o, acc):
        for op in o:
            if op[0] == 'ra':
                s = idc.get_strlit_contents(op[2])
                if s: acc['%X' % op[2]] = s.decode('latin1')
            elif op[0] == 'if': strs(op[4], acc)
        return acc
    r['strings'] = {}
    for k in places: strs(r[k], r['strings'])
    labels = idc.get_wide_dword(r['menu'])
    r['title'] = idc.get_wide_dword(labels + 0xC) if labels and idc.get_wide_dword(labels) != 0xFFFFFFFF else None
    return r

def short_check(core):
    """The service area's frame-and-action part with the shorter check after an action:
    no look at dword_755980, and straight back to set-up unless the screen is left."""
    out = list(core)
    i = out.index('cmp dword_755980, 1'); del out[i:i + 2]
    i = out.index("mov dword_7556A4, 6Fh"); assert out[i + 4].startswith('jz short L'); del out[i:i + 5]
    return out

def run(keep_core=False):
    ref = parse(0x47284A); res = []
    for ea in find_loops():
        r = parse(ea)
        if 'core' in r:
            r['short_check'] = r['core'] == short_check(ref['core'])
            if r['core'] != ref['core'] and not r['short_check']: r['why'].append('core differs')
            if not keep_core: del r['core']
        res.append(r)
    json.dump(res, open(OUT, 'w'))
    fit = [r for r in res if not r['why']]
    out = ['%d loops, %d fit' % (len(res), len(fit))]
    c = collections.Counter()
    for r in res:
        for w in set(re.sub(r'[0-9A-F]{5,}', 'X', w)[:44] for w in r['why']): c[w] += 1
    for k, v in c.most_common(30): out.append('%3d  %s' % (v, k))
    out.append('fit: ' + ' '.join('%X' % r['ea'] for r in fit))
    return '\n'.join(out)
