#!/usr/bin/env python3
"""The conversion against the original: the same replays in RAL.EXE and in rc2k.

Each run starts a game in a headless gamescope (no window on the desktop), walks the
menus with key presses to the replay viewer (options, the replay list, a replay, play),
lets the replay play and measures a stretch of it: frames a second, the CPU the game's
process used, its resident memory. The games:

  vanilla  RAL.EXE under GE-Proton (the frame rate from Wine's `fps` channel)
  linux    rc2k, native (the frame rate from rc2k.log's "the last 5.0 s" lines)
  windows  rc2k.exe under GE-Proton (the same lines)
  linux-gpu, windows-gpu   the same with rc2k's GPU renderer (RC2K_RENDERER=gpu); the
           original draws through Wine's Direct3D, so these are the like-for-like rows

Everything runs on a sandbox copy of the game folder (FILES linked, the rest copied), so
the install's saves and settings are left alone; the replays are copied in as replay00 and
replay01. RAL.EXE.bak (the particles cut out) is never used.

  tools/vanilla_bench.py setup GAME_DIR REPLAY00 REPLAY01 [--linux BIN] [--windows EXE]
  tools/vanilla_bench.py run [--games vanilla,linux-gpu,windows-gpu,linux,windows] [--replays 0,1] [--runs 3]
                             [--seconds 60]

Results: one line per run in ~/.cache/rc2k_vbench/results.jsonl, then a table.
"""

import argparse
import json
import os
import shutil
import statistics
import subprocess
import sys
import time

HOME = os.path.expanduser("~/.cache/rc2k_vbench")
GAME = os.path.join(HOME, "game")
PROTON = os.path.expanduser("~/.config/heroic/tools/proton/GE-Proton-latest")
WIDTH, HEIGHT = 1024, 768
TICK = os.sysconf("SC_CLK_TCK")
PAGE = os.sysconf("SC_PAGE_SIZE")

# ---- setup ----


def setup(args):
    """The sandbox: the game folder copied (FILES and the other big folders linked)."""
    src = os.path.abspath(args.game_dir)
    if os.path.exists(GAME):
        shutil.rmtree(GAME)
    os.makedirs(GAME)
    linked = {"FILES", "DIRECTX", "x64"}
    skipped = {"RAL.EXE.bak", "linux"}
    for name in os.listdir(src):
        if name in skipped or name.endswith(".log") or name.startswith("rc2k"):
            continue
        s, d = os.path.join(src, name), os.path.join(GAME, name)
        if name in linked:
            os.symlink(s, d)
        elif os.path.isdir(s):
            shutil.copytree(s, d, symlinks=True)
        else:
            shutil.copy2(s, d)
    replays = os.path.join(GAME, "savedata", "replay")
    os.makedirs(replays, exist_ok=True)
    for n, r in enumerate([args.replay00, args.replay01]):
        shutil.copy2(r, os.path.join(replays, f"replay0{n}"))
    cfg = os.path.join(GAME, "var", "ral.cfg")
    with open(cfg) as f:
        lines = [l for l in f.read().splitlines() if not l.startswith("nofmv=")]
    with open(cfg, "w") as f:
        f.write("\n".join(["nofmv=1"] + lines) + "\n")
    if args.linux:
        os.makedirs(os.path.join(GAME, "linux"), exist_ok=True)
        shutil.copy2(args.linux, os.path.join(GAME, "linux", "rc2k"))
    if args.windows:
        shutil.copy2(args.windows, os.path.join(GAME, "rc2k.exe"))
        for dll in ("SDL3.dll", "avcodec-61.dll", "avutil-59.dll"):
            beside = os.path.join(os.path.dirname(args.windows), dll)
            if os.path.exists(beside):
                shutil.copy2(beside, os.path.join(GAME, dll))
    print("sandbox ready:", GAME)


# ---- inside gamescope: one run ----


def processes(names):
    """The pids whose command line names one of `names`."""
    found = []
    for pid in os.listdir("/proc"):
        if not pid.isdigit():
            continue
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as f:
                cmd = f.read().replace(b"\0", b" ").decode(errors="replace")
        except OSError:
            continue
        if any(n in cmd for n in names) and "python" not in cmd and "proton " not in cmd:
            found.append(int(pid))
    return found


def cpu_and_memory(pid):
    """The process's CPU ticks so far (user + system) and resident bytes."""
    with open(f"/proc/{pid}/stat") as f:
        fields = f.read().rsplit(")", 1)[1].split()
    return int(fields[11]) + int(fields[12]), int(fields[21]) * PAGE


def key(d, name):
    from Xlib import XK
    from Xlib.ext import xtest

    code = d.keysym_to_keycode(XK.string_to_keysym(name))
    xtest.fake_input(d, 2, code)
    d.sync()
    time.sleep(0.08)
    xtest.fake_input(d, 3, code)
    d.sync()


def shot(name):
    subprocess.run(
        ["gamescopectl", "screenshot", os.path.join(HOME, f"shot_{name}.png")],
        capture_output=True,
    )


def inner(args):
    """One run, inside gamescope (DISPLAY is its Xwayland)."""
    from Xlib import display

    game, replay, seconds = args.game, args.replay, args.seconds
    env = dict(os.environ)
    log = os.path.join(HOME, f"game_{game}.log")
    # "linux-gpu" / "windows-gpu": rc2k with its GPU renderer (the software one otherwise).
    build, _, renderer = game.partition("-")
    if renderer == "gpu":
        env["RC2K_RENDERER"] = "gpu"
    if build == "linux":
        cmd = [os.path.join(GAME, "linux", "rc2k")]
        names = [os.path.join(GAME, "linux", "rc2k")]
    else:
        exe = "RAL.EXE" if game == "vanilla" else "rc2k.exe"
        env.update(
            STEAM_COMPAT_DATA_PATH=os.path.join(HOME, "compat"),
            STEAM_COMPAT_CLIENT_INSTALL_PATH=os.path.join(HOME, "client"),
            WINEDEBUG="+fps" if game == "vanilla" else "-all",
        )
        os.makedirs(env["STEAM_COMPAT_DATA_PATH"], exist_ok=True)
        os.makedirs(env["STEAM_COMPAT_CLIENT_INSTALL_PATH"], exist_ok=True)
        cmd = [os.path.join(PROTON, "proton"), "run", os.path.join(GAME, exe)]
        names = [exe]
    rc2k_log = os.path.join(GAME, "rc2k.log")
    if os.path.exists(rc2k_log):
        os.remove(rc2k_log)
    started = time.time()
    with open(log, "w") as out:
        proc = subprocess.Popen(cmd, cwd=GAME, env=env, stdout=out, stderr=subprocess.STDOUT)
    d = display.Display(os.environ["DISPLAY"])
    # The main menu after the loading (the sandbox's ral.cfg says nofmv=1: no opening films
    # in any of the games, so all three start alike).
    time.sleep(25)
    shot(f"{game}{replay}_menu")
    # Options (one Left: the menu wraps from the first item to it), the replay list (two
    # down), the replay, play.
    key(d, "Left")
    time.sleep(2.5)
    key(d, "Return")
    time.sleep(5)
    for _ in range(2):
        key(d, "Down")
        time.sleep(2.5)
    key(d, "Return")
    time.sleep(6)
    for _ in range(replay):
        key(d, "Down")
        time.sleep(1.5)
    shot(f"{game}{replay}_list")
    chosen = time.time()
    key(d, "Return")
    time.sleep(12)
    key(d, "Return")
    time.sleep(8)
    shot(f"{game}{replay}_playing")
    pids = processes(names)
    if not pids:
        result = {"game": game, "replay": replay, "error": "no game process"}
    else:
        pid = max(pids, key=lambda p: cpu_and_memory(p)[1])
        cpu0, _ = cpu_and_memory(pid)
        t0 = time.time()
        rss_peak = 0
        while time.time() - t0 < seconds:
            time.sleep(1)
            rss_peak = max(rss_peak, cpu_and_memory(pid)[1])
        cpu1, _ = cpu_and_memory(pid)
        t1 = time.time()
        shot(f"{game}{replay}_end")
        result = {
            "game": game,
            "replay": replay,
            "cpu_percent": 100.0 * (cpu1 - cpu0) / TICK / (t1 - t0),
            "rss_mb": rss_peak / 1e6,
            "fps": frame_rates(game, log, rc2k_log, t0 - started, t1 - started),
            "to_replay_s": chosen - started,
        }
    with open(os.path.join(HOME, "inner_result.json"), "w") as f:
        json.dump(result, f)
    stop(game, env, proc)


def stop(game, env, proc):
    if game.startswith("linux"):
        proc.terminate()
        try:
            proc.wait(10)
        except subprocess.TimeoutExpired:
            proc.kill()
    else:
        wine_env = dict(env, WINEPREFIX=os.path.join(env["STEAM_COMPAT_DATA_PATH"], "pfx"))
        subprocess.run([os.path.join(PROTON, "files", "bin", "wineserver"), "-k"], env=wine_env)


def frame_rates(game, log, rc2k_log, start, end):
    """The frames a second in the measured stretch: one figure per report line."""
    rates = []
    if game == "vanilla":
        # "trace:fps:wined3d_cs_exec_present ... @ approx 59.96fps": about one a second;
        # the lines carry no time, so the stretch is the lines' share of the run.
        with open(log, errors="replace") as f:
            lines = [l for l in f if "wined3d_cs_exec_present" in l and "approx" in l]
        if lines:
            total = len(lines)
            a, b = int(total * start / max(end, 1)), total
            for l in lines[a:b]:
                rates.append(float(l.split("approx")[1].split("fps")[0]))
    else:
        if os.path.exists(rc2k_log):
            with open(rc2k_log, errors="replace") as f:
                for l in f:
                    if "platform: the last" in l and "a second)" in l:
                        rates.append(float(l.split("frames (")[1].split(" a second")[0]))
        rates = rates[-max(1, int((end - start) / 5)) :]
    return rates


# ---- the runs ----


def run(args):
    games = args.games.split(",")
    replays = [int(r) for r in args.replays.split(",")]
    results_path = os.path.join(HOME, "results.jsonl")
    for n in range(args.runs):
        for game in games:
            for replay in replays:
                inner_path = os.path.join(HOME, "inner_result.json")
                if os.path.exists(inner_path):
                    os.remove(inner_path)
                cmd = [
                    "gamescope", "--backend", "headless",
                    "-W", str(WIDTH), "-H", str(HEIGHT), "-w", str(WIDTH), "-h", str(HEIGHT),
                    "--", sys.executable, os.path.abspath(__file__), "inner",
                    "--game", game, "--replay", str(replay), "--seconds", str(args.seconds),
                ]
                env = dict(os.environ, SDL_AUDIODRIVER="dummy")
                limit = args.seconds + 240
                try:
                    subprocess.run(cmd, env=env, timeout=limit,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                except subprocess.TimeoutExpired:
                    print(f"{game} replay {replay}: TIMED OUT after {limit} s", flush=True)
                if os.path.exists(inner_path):
                    with open(inner_path) as f:
                        result = json.load(f)
                else:
                    result = {"game": game, "replay": replay, "error": "no result"}
                result["run"] = n
                with open(results_path, "a") as f:
                    f.write(json.dumps(result) + "\n")
                print(json.dumps(result), flush=True)
    table(results_path)


def table(path):
    rows = {}
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            if "error" in r:
                continue
            rows.setdefault((r["game"], r["replay"]), []).append(r)
    print(f"\n{'game':8} {'replay':>6} {'runs':>4} {'fps avg':>8} {'fps min':>8} "
          f"{'CPU %':>7} {'RSS MB':>7}")
    for (game, replay), rs in sorted(rows.items()):
        fps = [x for r in rs for x in r["fps"]]
        avg = statistics.mean(fps) if fps else float("nan")
        low = min(fps) if fps else float("nan")
        cpu = statistics.mean(r["cpu_percent"] for r in rs)
        rss = max(r["rss_mb"] for r in rs)
        print(f"{game:8} {replay:>6} {len(rs):>4} {avg:8.1f} {low:8.1f} {cpu:7.1f} {rss:7.0f}")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="what", required=True)
    s = sub.add_parser("setup")
    s.add_argument("game_dir")
    s.add_argument("replay00")
    s.add_argument("replay01")
    s.add_argument("--linux")
    s.add_argument("--windows")
    r = sub.add_parser("run")
    r.add_argument("--games", default="vanilla,linux-gpu,windows-gpu,linux,windows")
    r.add_argument("--replays", default="0,1")
    r.add_argument("--runs", type=int, default=3)
    r.add_argument("--seconds", type=int, default=60)
    i = sub.add_parser("inner")
    i.add_argument("--game", required=True)
    i.add_argument("--replay", type=int, required=True)
    i.add_argument("--seconds", type=int, required=True)
    t = sub.add_parser("table")
    args = p.parse_args()
    {"setup": setup, "run": run, "inner": inner, "table": lambda a: table(os.path.join(HOME, "results.jsonl"))}[args.what](args)


if __name__ == "__main__":
    main()
