#!/usr/bin/env python3
"""Typed tables (2026-10-22): gives each hook file whose `dispatch` is one call of a shared
dispatcher a `pub fn entry(r: &mut Regs) -> Result<(), Refused>`: the same call made to the
dispatcher's entry twin (no hook, no shadow mode; a refusal returned to the caller). Files
whose `dispatch` does more are listed, not touched.

    tools/menu_tables/add_entries.py <hook name>=<file> ...
"""
import importlib.util
import os
import re
import sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    'strip_hook', os.path.join(here, '..', 'strip_hook.py'))
strip_hook = importlib.util.module_from_spec(spec)
argv, sys.argv = sys.argv, sys.argv[:1]
spec.loader.exec_module(strip_hook)

# dispatcher -> (twin, which of its arguments after `hook, r, call_original` the twin
# takes, whether the twin takes the hook's name first)
TWINS = {
    'dispatch_state': ('state_entry', [1], False),
    'dispatch_keyed_state': ('state_entry', [1], False),
    'dispatch_replayed_state': ('state_entry', [1], False),
    'dispatch_draw': ('draw_entry', [0], True),
    'dispatch_draw_rewound': ('draw_entry', [0], True),
    'dispatch_screen': ('screen_entry', [0], True),
    'dispatch_choose': ('choose_entry', [0], False),
    'dispatch_volume': ('volume_entry', [0], False),
    'dispatch_plain': ('plain_entry', [0], False),
    'dispatch_parted': ('parted_entry', [0, 1], False),
    'dispatch_level': ('level_entry', [0], False),
    'dispatch_edit': ('edit_entry', [0], False),
    'dispatch_panel': ('panel_entry', [0], True),
    'dispatch_leaf': ('leaf_entry', [0], False),
    'dispatch_exit': ('exit_entry', [0], False),
    'dispatch_pixels': ('pixels_entry', [1], False),
    'dispatch_net': ('net_entry', [0], False),
    'dispatch_state_floats': ('floats_entry', [2], False),
    'dispatch_sender': ('send_entry', [0], False),
    'dispatch_turn': ('turn_entry', [0], False),
    'dispatch_effects': ('effects_entry', [0], False),
    'dispatch_effect': ('effect_entry', [0], False),
    'dispatch_parts': ('regs_entry', [0], False),
    'dispatch_lobby': ('lobby_entry', [0], False),
    'dispatch_regs': ('regs_entry', [2], False),
    'dispatch_spread': ('spread_entry', [0], False),
    'dispatch_reader': ('reader_entry', [0], False),
    'dispatch_marker': ('marker_entry', [0], True),
    'dispatch_turn_floats': ('turn_entry', [0], False),
}
# dispatchers of the same name in two modules: by the module the file takes it from
BY_MODULE = {
    ('dispatch_item', 'option_items'): ('item_entry', [0, 1], False),
    ('dispatch_item', 'controller_items'): ('item_entry', [0], False),
    ('dispatch_page', 'car_pages'): ('page_entry', [0], True),
    ('dispatch_page', 'draw_player_details'): ('page_entry', [0], True),
    ('dispatch_rows', 'draw_stage_records'): ('rows_entry', [0], True),
    ('dispatch_rows', 'stage_rows'): ('rows_entry', [0, 1], False),
}


def split_args(text):
    """Top-level arguments: commas inside brackets and inside a closure's `|..|` don't
    split."""
    out, depth, cur, in_pipes = [], 0, '', False
    for n, c in enumerate(text):
        if c == '|' and depth == 0:
            # `|` opens a closure's parameters where an argument starts (`|x|`, `move |x|`)
            if in_pipes:
                in_pipes = False
            elif cur.strip() in ('', 'move', '&', '&move'):
                in_pipes = True
        elif c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        if c == ',' and depth == 0 and not in_pipes:
            out.append(cur.strip())
            cur = ''
        else:
            cur += c
    if cur.strip():
        out.append(cur.strip())
    return out


def module_of(s, helper, file_module):
    m = re.search(r'([a-z_]+)::\{[^}]*\b' + helper + r'\b', s) or \
        re.search(r'([a-z_]+)::' + helper + r'\b', s)
    return m.group(1) if m else file_module


CHECK = os.environ.get('CHECK') == '1'


def entry_for(path, name):
    s = open(path).read()
    if 'pub fn entry(' in s and not CHECK:
        return 'has one'
    m = re.search(r'^extern "C" fn dispatch\(r: &mut Regs\) \{', s, re.M)
    if not m:
        return 'no plain dispatch'
    end = strip_hook.cut_item(s, m.start())
    body = s[m.end():end - 1].strip()
    body = re.sub(r'^HOOK\.count_call\(\);\s*', '', body)
    call = re.search(r'\b((?:[a-z_]+::)*)(dispatch_[a-z_]+)\(', body)
    if not call:
        return 'no dispatcher'
    helper = call.group(2)
    open_at = call.end() - 1
    close_at = strip_hook.cut_item(body, open_at)
    tail = body[close_at:].strip()
    if tail not in ('', ';'):
        return 'more after the call'
    before = body[:call.start()]
    if re.search(r'\b(HOOK|call_original|shadow)\b', before):
        return 'uses the hook before the call'
    args = split_args(body[open_at + 1:close_at - 1 if body[close_at - 1] == ')' else close_at - 2])
    if args[:3] != ['&HOOK', 'r', 'call_original']:
        return 'arguments ' + ', '.join(args[:3])
    rest = args[3:]
    module = module_of(s, helper, os.path.splitext(os.path.basename(path))[0])
    twin = TWINS.get(helper) or BY_MODULE.get((helper, module))
    if not twin:
        return f'no twin for {helper} ({module})'
    twin_name, keep, named = twin
    new_args = ([f'"{name}"'] if named else []) + ['r'] + [rest[k] for k in keep]
    prefix = call.group(1)
    call_text = f'{prefix}{twin_name}({", ".join(new_args)})'
    if CHECK:
        got = s[s.index('pub fn entry('):]
        got = got[:strip_hook.cut_item(got, 0)]
        flat = lambda t: re.sub(r'\s+', '', t)
        return 'same' if flat(call_text) in flat(got) else 'DIFFERENT: ' + call_text
    fn = (
        '\n/// The Rust version where the typed tables run it (no hook; a refusal is the\n'
        '/// caller\'s).\n'
        f'pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {{\n'
        f'    {call_text}\n}}\n'
    ).replace('\n    \n', '\n')
    s = s[:end] + '\n' + fn + s[end:]
    if not prefix:
        # the twin from where the dispatcher comes
        s = re.sub(r'(use [^;]*\b)' + helper + r'\b', r'\1' + helper + ', ' + twin_name, s, count=1)
    open(path, 'w').write(s)
    return 'added'


results = {}
for a in argv[1:]:
    name, path = a.split('=', 1)
    results.setdefault(entry_for(path, name), []).append(name)
for k, v in sorted(results.items()):
    print(f'{k}: {len(v)}')
    if k != 'added':
        for n in v:
            print('   ', n)
