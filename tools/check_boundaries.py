#!/usr/bin/env python3
"""Lists every reference in the workspace that crosses a layer boundary the wrong way.

The layer map is in LAYERS below (explained in the wiki page Code-layers): a module may use modules of
its own layer and of lower layers; a use of a higher layer is a violation. Three further rules:
  * the system crates (sdl3_sys, libc, windows_sys, dear_imgui_*, mdns_sd) only under
    src/platform/ (EXTERN_HOMES lists the few other homes allowed);
  * rc2k_core does no I/O, time, threads, env or process calls (CORE_BANNED);
  * the tools (debug, remote) are used by game code only through their doors (TOOL_DOORS).
Test code (`#[cfg(test)]` items and modules, files named tests.rs / *_tests.rs, test_support)
is left out: tests may reach anywhere.

    check_boundaries.py [repo]            summary: counts per pair of modules, worst files
    check_boundaries.py [repo] --list     every violation as file:line: from -> to  (text)
    check_boundaries.py [repo] --matrix   the whole module-to-module use matrix
    check_boundaries.py [repo] --max N    exit 1 when there are more than N violations (CI)
"""
import argparse
import os
import re
import sys
from collections import Counter, defaultdict

# Lower number = lower layer. A module may use its own layer and below.
LAYERS = {
    # 0: foundation: no game knowledge, no platform calls of its own.
    "errors": 0, "memory": 0, "math": 0, "strings": 0, "program_log": 0,
    # 1: the platform: the only code that talks to the system and to SDL.
    "platform": 1, "threading": 1, "logfile": 1, "files": 1, "clock": 1, "language": 1,
    # 2: engine services over the platform: drawing, pictures, sound, input, film, wire.
    "render": 2, "gfx": 2, "text": 2, "d3d": 2, "audio": 2, "sound": 2, "input": 2, "video": 2, "net": 2,
    # 3: the simulation and the race.
    "physics": 3, "scene": 3, "effects": 3, "hud": 3, "drive": 3, "race": 3,
    "records": 3, "saves": 3, "settings": 3, "service": 3,
    # 4: the front end: menus, screens, the installer, dialogs.
    "menu": 4, "screens": 4, "save_code": 4, "install": 4, "first_run": 4,
    "error_dialog": 4, "single_copy": 4,
    # 5: the root: builds the owners, starts the threads, runs the program, ends it.
    "owners": 5, "system": 5, "fatal": 5, "standalone": 5, "shell": 5, "bin": 5, "lib": 5,
    # 6: tools beside the game: may read anything; the game reaches them through doors only.
    "debug": 6, "remote": 6,
}
LAYER_NAMES = ["foundation", "platform", "engine", "simulation", "front end", "root", "tools"]
# Items of a higher layer every layer may name (the doors). Kept short on purpose.
DOORS = {
    "owners": None,                    # the owners' types are passed down everywhere
    "fatal": {"raise"},                # ending the program with a reason
    "system": {"pause"},
    "shell": {"frame"},    # the frame's end and the window's events: they need every owner   # the pause pumps events and takes keys, a root service used from below
}
TOOL_DOORS = {
    "debug": {"timing", "imgui_sdl", "switches"},
    # the test driver: every function of src/remote/off.rs has the same signature in driver.rs
    "remote": {
        "BUILT", "audio_turn_done", "autodrive", "autodrive_keys", "count_sound",
        "course_hashes", "fake_camera", "fake_camera_count", "fake_camera_frame", "fold_pixel",
        "frame", "held_clock", "held_race", "hold_loading_frame", "lose_this_frame", "mangled",
        "mask_hashes", "note_open", "note_racing", "note_shake",
        "note_sound_peak", "pace_call", "poll", "race_loading", "race_loop_begins",
        "refresh_rate", "renderer_step_fails", "replay_tick", "reset_texture_hash",
        "scaled_millis", "shot_frame_drawn", "shot_wanted", "shots_loading", "shots_on",
        "sim_hash_step", "sound_loaded", "speed", "started_hashes", "steam_off",
        "stick_at_start", "texture_row", "turn_end_taken", "wall_micros", "window_shot_due",
        "window_shot_taken", "window_taken",
    },
}
TOOLS = {"debug", "remote"}
EXTERN = ["sdl3_sys", "libc", "windows_sys", "dear_imgui_rs", "dear_imgui_sys", "mdns_sd"]
# Only the platform names the system crates.
EXTERN_HOMES = ("src/platform/",)
CORE_BANNED = ["std::fs", "std::net", "std::thread", "std::time", "std::env", "std::process",
               "std::io::stdin", "std::io::stdout", "Instant::", "SystemTime::"]
# Value types under those paths that do no I/O: a duration, an address.
CORE_VALUE_TYPES = re.compile(r"std::time::Duration\b|std::net::(Ipv4Addr|Ipv6Addr|SocketAddrV4)\b")

USE_GROUP = re.compile(r"\bcrate::\{")
PATH = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)(?:::([A-Za-z_][A-Za-z0-9_]*))?")


def strip_comments(line, state):
    """Drops // comments and /* */ blocks (strings holding these are rare enough to ignore)."""
    out = ""
    i = 0
    while i < len(line):
        if state["block"]:
            j = line.find("*/", i)
            if j < 0:
                return out
            state["block"] = False
            i = j + 2
        elif line.startswith("/*", i):
            state["block"] = True
            i += 2
        elif line.startswith("//", i):
            return out
        else:
            out += line[i]
            i += 1
    return out


def code_lines(path):
    """Yields (line number, text) of the non-test code of a file."""
    name = os.path.basename(path)
    if name == "tests.rs" or name.endswith("_tests.rs") or "/test_support/" in path:
        return
    try:
        lines = open(path, encoding="utf-8").read().split("\n")
    except OSError:
        return
    state = {"block": False}
    skip_depth = None      # brace depth at which a #[cfg(test)] item ends
    depth = 0
    pending_test = False
    for n, raw in enumerate(lines, 1):
        text = strip_comments(raw, state)
        stripped = text.strip()
        if skip_depth is None and re.match(r"#\[cfg\((all\()?test\b", stripped):
            pending_test = True
            continue
        opens, closes = text.count("{"), text.count("}")
        if pending_test and stripped and not stripped.startswith("#["):
            pending_test = False
            if opens > closes:
                skip_depth = depth
            depth += opens - closes
            continue          # the item's first (or only) line
        depth += opens - closes
        if skip_depth is not None:
            if depth <= skip_depth:
                skip_depth = None
            continue
        yield n, text


def module_of(rel):
    parts = rel.split("/")
    if parts[0] != "src":
        return None
    top = parts[1]
    return top[:-3] if top.endswith(".rs") else top


def group_names(text, start):
    """Top-level names (and their first item) inside `crate::{ ... }` starting at `start`."""
    depth, i, token, names, first = 1, start, "", [], True
    while i < len(text) and depth:
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
        if depth == 1 and c in ",}" or depth == 0:
            token, first = "", True
        elif depth >= 1 and first and (c.isalnum() or c in "_:"):
            token += c
            if depth == 1 or token.count("::") == 0:
                pass
        if depth >= 1 and c in ",{}" and token:
            head = token.split("::")
            if depth <= 2 and head[0]:
                names.append((head[0], head[1] if len(head) > 1 and head[1] else None))
            token, first = "", depth == 1 and c == ","
            if c == "{":
                first = False
        i += 1
    return names


def references(path):
    """Yields (line, target module, first item or None) for each `crate::x` use in a file."""
    buffered, start_line = "", 0
    for n, text in code_lines(path):
        if buffered:
            buffered += " " + text
            if buffered.count("{") > buffered.count("}"):
                continue
            text, n, buffered = buffered, start_line, ""
        elif USE_GROUP.search(text) and text.count("{") > text.count("}"):
            buffered, start_line = text, n
            continue
        for m in USE_GROUP.finditer(text):
            for name, item in group_names(text, m.end()):
                yield n, name, item
        for m in PATH.finditer(text):
            yield n, m.group(1), m.group(2)


def main():
    parser = argparse.ArgumentParser(description="Uses across the layer map the wrong way.")
    parser.add_argument("repo", nargs="?", default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                        help="the repository (default: the one this script is in)")
    show = parser.add_mutually_exclusive_group()
    show.add_argument("--list", action="store_true", help="every violation as file:line")
    show.add_argument("--matrix", action="store_true", help="every module-to-module count")
    parser.add_argument("--max", type=int, metavar="N", help="exit 1 above N violations (the rounds' ratchet)")
    opts = parser.parse_args()
    repo, limit = opts.repo, opts.max
    os.chdir(repo)
    matrix = Counter()
    violations = []       # (kind, file, line, from, to, item)
    unknown = Counter()
    for root in ("src", "rc2k_core/src"):
        for folder, _, files in os.walk(root):
            for f in sorted(files):
                if not f.endswith(".rs"):
                    continue
                rel = os.path.join(folder, f)
                core = rel.startswith("rc2k_core/")
                src = module_of(rel)
                for n, text in code_lines(rel):
                    if core:
                        for banned in CORE_BANNED:
                            if banned in CORE_VALUE_TYPES.sub("", text):
                                violations.append(("core-io", rel, n, "rc2k_core", banned, None))
                    elif not rel.startswith(EXTERN_HOMES):
                        for ext in EXTERN:
                            if re.search(r"\b%s::" % ext, text):
                                violations.append(("extern", rel, n, src, ext, None))
                                break
                if core:
                    continue
                for n, to, item in references(rel):
                    if to == src:
                        # Inside platform: only ffi and os may be leaves; ffi must not look up.
                        if src == "platform" and "/ffi/" in rel and item not in (None, "ffi"):
                            violations.append(("ffi-up", rel, n, "platform::ffi", "platform::" + str(item), None))
                        continue
                    if to == "test_support":
                        continue
                    if to not in LAYERS:
                        unknown[to] += 1
                        continue
                    matrix[(src, to)] += 1
                    if src not in LAYERS:
                        unknown[src] += 1
                        continue
                    if to in TOOLS and src not in TOOLS and LAYERS[src] < 5:
                        doors = TOOL_DOORS[to]
                        if doors is not None and item not in doors:
                            violations.append(("tool", rel, n, src, to, item))
                        continue
                    if LAYERS[to] > LAYERS[src]:
                        doors = DOORS.get(to, set())
                        if doors is None or item in doors:
                            continue
                        # Naming a higher module's owner type (to take it as a parameter) is
                        # counted apart from calling its code.
                        owner = item is not None and (item == "owner" or item.startswith("owner_"))
                        violations.append(("up-owner" if owner else "up", rel, n, src, to, item))

    if opts.matrix:
        for (a, b), c in sorted(matrix.items()):
            print(f"{a:14} -> {b:14} {c}")
        return 0
    if opts.list:
        for kind, rel, n, a, b, item in sorted(violations):
            print(f"{rel}:{n}: [{kind}] {a} -> {b}" + (f"::{item}" if item else ""))
        return 0
    kinds = Counter(v[0] for v in violations)
    print("violations: %d  (%s)" % (len(violations), ", ".join(f"{k} {c}" for k, c in sorted(kinds.items()))))
    print("\nupward uses, per pair of modules (from -> to: uses, files):")
    pairs, pair_files = Counter(), defaultdict(set)
    for kind, rel, n, a, b, item in violations:
        if kind in ("up", "up-owner", "tool", "ffi-up"):
            pairs[(a, b)] += 1
            pair_files[(a, b)].add(rel)
    for (a, b), c in pairs.most_common():
        la = LAYER_NAMES[LAYERS[a]] if a in LAYERS else "ffi"
        lb = LAYER_NAMES[LAYERS[b]] if b in LAYERS else "platform"
        print(f"  {a:14} -> {b:14} {c:5} uses in {len(pair_files[(a, b)]):3} files   ({la} -> {lb})")
    print("\nsystem crates outside src/platform/ (file: lines):")
    ext = Counter((v[1], v[4]) for v in violations if v[0] == "extern")
    for (rel, crate), c in ext.most_common():
        print(f"  {rel}: {c} lines of {crate}")
    print("\nrc2k_core I/O (file:line what):")
    for v in violations:
        if v[0] == "core-io":
            print(f"  {v[1]}:{v[2]} {v[4]}")
    print("\nworst files (upward uses):")
    worst = Counter(v[1] for v in violations if v[0] in ("up", "up-owner", "tool", "ffi-up"))
    for rel, c in worst.most_common(25):
        print(f"  {c:4} {rel}")
    if unknown:
        print("\nnames after crate:: that are no module of the map:", dict(unknown.most_common(8)))
    if limit is not None and len(violations) > limit:
        print(f"\nBOUNDARIES: {len(violations)} violations, above the allowed {limit}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
