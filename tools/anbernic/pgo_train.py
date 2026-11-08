#!/usr/bin/env python3
"""The training run of a profile-guided build (tools/anbernic/pgo.sh): the instrumented game on
the handheld through tools/remote.py, from the title through the menus into the first stage,
driven along the stage line by the pilot, then quit (the counts are written as it ends).

    pgo_train.py [seconds]    the pilot's time on the stage (default 85)

Needs RC2K_REMOTE_HOST (pgo.sh sets it) and a `remote` build installed as rc2k_remote.
"""
import re, subprocess, sys, time

ROOT = __file__.rsplit('/tools/', 1)[0]


def remote(*args, timeout=60):
    return subprocess.run(['python3', 'tools/remote.py', *args], cwd=ROOT, capture_output=True,
                          text=True, timeout=timeout).stdout.strip()


def state():
    """(in a race, the menu's number or None, the focused item)."""
    m = re.search(r'race=(\d+) menu=(?:Some\(MenuId\((\d+)\)\)|None) item=(\d+)', remote('state'))
    if not m:
        return None, None, None
    return int(m[1]), int(m[2]) if m[2] else None, int(m[3])


def key(name, wait):
    remote('key', name, '100')
    time.sleep(wait)


def main():
    seconds = sys.argv[1] if len(sys.argv) > 1 else '85'
    print(remote('start', '--gpu')[-20:], flush=True)
    time.sleep(10)
    try:
        # Past the title and the films to the main menu (5), backing out of any other.
        for _ in range(40):
            menu = state()[1]
            if menu == 5:
                break
            key('Return' if menu is None else 'Escape', 5 if menu is None else 3)
        else:
            sys.exit('pgo_train: the main menu was not reached')
        # The first entries all the way down: a single rally's first stage.
        for _ in range(26):
            if state()[0] == 1:
                break
            key('Return', 6)
        else:
            sys.exit('pgo_train: no race was started')
        time.sleep(12)
        print(remote('pilot', seconds, timeout=int(seconds) + 30)[-120:], flush=True)
        print(state(), flush=True)
    finally:
        remote('stop', timeout=120)


if __name__ == '__main__':
    main()
