#!/usr/bin/env python3
"""Drives a `remote` build of the game for automated tests (src/remote.rs).

    remote.py build [--handheld]         builds the binary with the `remote` feature
    remote.py start [--software] [--fresh] [--profile]    starts it (PC: on a copy of var and savedata);
                                         the GPU renderer is the default (--gpu is kept: the same),
                                         --software forces the rasteriser (RC2K_RENDERER=software)
    remote.py key Return [ms]            a key press (SDL's scancode names)
    remote.py down W / up W              a key held
    remote.py pad a [ms]                 a gamepad button (menus)
    remote.py joy attach|detach          a plain stand-in stick plugged in or out
    remote.py joy axis n v / button n 0|1   its axes and buttons
    remote.py shot name                  the next frame to <folder>/shots/name.png
    remote.py time n                     the arcade's countdown set to n
    remote.py state                      frames, race flag, menu, item, speed, time, place, cars
    remote.py wait COND.. [secs]         polls state until every COND holds (menu!=Some(MenuId(5)), item=3,
                                         race=1, screen_frames>=20; ops = != >= <= > < ; default 10 s);
                                         exit 0 when met, 1 with the last state on timeout
    remote.py press KEY [secs]           a key press, then waits (default 5 s) for menu / item / race to change
                                         and the new screen to draw SETTLE frames; exit 1 when nothing changed
    remote.py pilot seconds              the game's computer driver drives the player's car, with a trace
    remote.py drive on|off               the game's computer driver at the player's wheel, or not
    remote.py log [n]                    the game's log, last n lines
    remote.py stop                       quits, then ends it

--handheld (or RC2K_REMOTE_HOST=root@address) drives the handheld over SSH. There var and
savedata are put back on stop, as the folder is the real one. On a Mac it runs there as on the
PC (the game's folder ~/rally, or RC2K_REMOTE_INSTALLED).
"""
import math, os, shutil, struct, subprocess, sys, time, zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOME = os.path.expanduser(os.environ.get('RC2K_REMOTE_HOME', '~/.cache/rc2k_remote'))  # a second copy: another folder
GAME = HOME + '/game'
# Run on a Mac itself (over SSH from elsewhere: `ssh mac rc2k_src/tools/remote.py ..`): the
# game's folder there is RC2K_REMOTE_INSTALLED (default ~/rally, as deploy_standalone.sh mac).
MAC = sys.platform == 'darwin'
INSTALLED = os.path.expanduser(os.environ.get('RC2K_REMOTE_INSTALLED',
                                              '~/rally' if MAC else '~/.local/share/rally-championship'))
SHOTS = HOME + '/shots'
# The Mac build in Darling on Linux (RC2K_REMOTE_DARLING=its prefix; a test build linked with
# RC2K_MAC_WEAK_FRAMEWORKS): started by `darling shell`, it sees this machine's files under
# /Volumes/SystemRoot. Its game folder needs its own files (a symlink's target would be
# Darling's).
DARLING = os.environ.get('RC2K_REMOTE_DARLING')
# A device over SSH: the handheld by default; RC2K_REMOTE_GAME / RC2K_REMOTE_KEY for another
# (the Steam Deck: the game started from Steam by its run.sh, which sets RC2K_REMOTE).
DEVICE_GAME = os.environ.get('RC2K_REMOTE_GAME', '/mnt/mmc/Roms/APPS/rally')
DEVICE_REMOTE = '/tmp/rc2k_remote'
KEY = os.path.expanduser(os.environ.get('RC2K_REMOTE_KEY', '~/.ssh/rg34xxsp'))


def host():
    if '--handheld' in sys.argv:
        sys.argv.remove('--handheld')
        return os.environ.get('RC2K_REMOTE_HOST', 'root@192.168.1.207')
    return os.environ.get('RC2K_REMOTE_HOST')


HOST = host()
FOLDER = DEVICE_REMOTE if HOST else HOME + '/ctl'


def ssh(command, data=None):
    return subprocess.run(['ssh', '-i', KEY, '-o', 'BatchMode=yes', HOST, command], input=data,
                          capture_output=True, check=False)


def put_cmd(line):
    if HOST:
        ssh(f'cd {FOLDER} && cat > cmd.new && mv cmd.new cmd', (line + '\n').encode())
    else:
        with open(FOLDER + '/cmd.new', 'w') as f:
            f.write(line + '\n')
        os.rename(FOLDER + '/cmd.new', FOLDER + '/cmd')


def replies():
    if HOST:
        return ssh(f'cat {FOLDER}/reply 2>/dev/null').stdout.decode(errors='replace')
    try:
        return open(FOLDER + '/reply').read()
    except FileNotFoundError:
        return ''


def process(pid):
    """A local process's state letter and program name; None when it is gone (/proc on Linux,
    ps on macOS)."""
    if MAC:
        out = subprocess.run(['ps', '-o', 'stat=,comm=', '-p', str(pid)], capture_output=True, text=True).stdout
        if not out.strip():
            return None
        state, comm = out.split(None, 1)
        return state[0], os.path.basename(comm.strip())
    try:
        stat = open(f'/proc/{pid}/stat').read()
        return stat.split(') ')[1][0], open(f'/proc/{pid}/comm').read().strip()
    except OSError:
        return None


def running():
    if HOST:
        return ssh('pgrep -x rc2k_remote').returncode == 0
    if DARLING:
        return darling_game() is not None
    try:
        found = process(int(open(HOME + '/pid').read()))
    except (OSError, ValueError):
        return False
    return found is not None and found[0] != 'Z'


def darling_game():
    """The program under `darling shell` (named `main` there, its starter gone at once): found
    by its command line and folder; None when it is gone."""
    for entry in os.listdir('/proc'):
        try:
            line = open(f'/proc/{entry}/cmdline', 'rb').read()
            if line.startswith(b'./rc2k\0') and os.readlink(f'/proc/{entry}/cwd') == GAME:
                return int(entry)
        except (OSError, ValueError):
            pass
    return None


def ask(command, timeout=20):
    # Checked before the command is written: a copy that is running but not the one the pid
    # file names would otherwise take a command reported as not run.
    if not running():
        sys.exit(f'the game has ended ("{command}" not run): remote.py log')
    number = str(int(time.time() * 1000) % 100000000)
    put_cmd(f'{number} {command}')
    end = time.time() + timeout
    while time.time() < end:
        if not running():
            sys.exit(f'the game has ended ("{command}" not run): remote.py log')
        for line in replies().splitlines():
            if line.startswith(number + ' '):
                return line[len(number) + 1:]
        time.sleep(0.02)
    sys.exit(f'no answer to "{command}" in {timeout} s: is the game running? (remote.py log)')


def png(raw, width, height, out):
    step = 2 if width > 1000 else 1
    size = len(raw) // (width * height)
    rows = []
    for y in range(0, height, step):
        row = bytearray([0])
        at = y * width * size
        line = raw[at:at + width * size]
        if size == 2:  # a 16-bit screen (the installer): 5, 6, 5
            for x in range(0, width, step):
                v = line[2*x] | line[2*x+1] << 8
                row += bytes(((v >> 8) & 0xF8, (v >> 3) & 0xFC, (v << 3) & 0xF8))
        else:
            row += bytes(b for x in range(0, width, step) for b in (line[4*x+2], line[4*x+1], line[4*x]))
        rows.append(bytes(row))
    def chunk(kind, body):
        return struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body))
    with open(out, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width // step, len(rows), 8, 2, 0, 0, 0))
                + chunk(b'IDAT', zlib.compress(b''.join(rows), 6)) + chunk(b'IEND', b''))


def shot(name):
    answer = ask(f'shot {name}')
    if answer.startswith('error'):
        sys.exit(answer)
    width, height = map(int, answer.split())
    if HOST:
        raw = ssh(f'cat {FOLDER}/{name}.raw; rm -f {FOLDER}/{name}.raw').stdout
    else:
        path = f'{FOLDER}/{name}.raw'
        raw = open(path, 'rb').read()
        os.unlink(path)
    os.makedirs(SHOTS, exist_ok=True)
    out = f'{SHOTS}/{name}.png'
    png(raw, width, height, out)
    print(out)


def build():
    if HOST:
        subprocess.run(['tools/anbernic/run.sh', 'cargo', 'build', '--release', '--target', 'aarch64-unknown-linux-gnu',
                        '--features', 'remote', '--bin', 'rc2k'], cwd=ROOT, check=True)
        data = open(ROOT + '/target/a64-linux/aarch64-unknown-linux-gnu/release/rc2k', 'rb').read()
        ssh(f'cat > {DEVICE_GAME}/rc2k_remote; chmod +x {DEVICE_GAME}/rc2k_remote', data)
    else:
        target, folder = ('x86_64-unknown-linux-gnu', 'target/linux-remote')
        if MAC:
            target, folder = (os.uname().machine.replace('arm64', 'aarch64') + '-apple-darwin', 'target/mac-remote')
        env = dict(os.environ, CARGO_TARGET_DIR=folder)
        subprocess.run(['cargo', 'build', '--release', '--target', target, '--features', 'remote',
                        '--bin', 'rc2k'], cwd=ROOT, env=env, check=True)
        os.makedirs(GAME, exist_ok=True)
        shutil.copy(f'{ROOT}/{folder}/{target}/release/rc2k', GAME + '/rc2k.new')
        os.replace(GAME + '/rc2k.new', GAME + '/rc2k')


def start(software, fresh, profile=False):
    renderer = 'RC2K_RENDERER=software ' if software else ''
    if os.environ.get('RC2K_FAIL_RENDERER'):  # start-up steps made to fail, to test the fallbacks
        renderer += f"RC2K_FAIL_RENDERER={os.environ['RC2K_FAIL_RENDERER']} "
    if profile:
        renderer += 'RC2K_PROFILE=1 '
    if os.environ.get('RC2K_GL_BENCH'):  # the GL driver's costs, every n-th big part (handheld)
        renderer += f"RC2K_GL_BENCH={int(os.environ['RC2K_GL_BENCH'])} "
    if os.environ.get('RC2K_SLOW_FRAMES'):  # log each frame in a race slower than this many ms
        renderer += f"RC2K_SLOW_FRAMES={int(os.environ['RC2K_SLOW_FRAMES'])} "
    if os.environ.get('RC2K_SHOT_CLOCKS'):  # a replay's frames saved after these ticks (src/remote.rs)
        renderer += f"RC2K_SHOT_CLOCKS={','.join(str(int(c)) for c in os.environ['RC2K_SHOT_CLOCKS'].split(','))} "
    if os.environ.get('RC2K_DUMP'):  # the n-th big flush's commands to rc2k_frame_dump.bin
        renderer += f"RC2K_DUMP={int(os.environ['RC2K_DUMP'])} "
    for switch in ('RC2K_MIRROR_BESIDE', 'RC2K_MIRROR_ALWAYS', 'RC2K_DRAW_SERIAL', 'RC2K_PLAYERS_BESIDE', 'RC2K_CAR_PART', 'RC2K_CANVAS_CHECK'):  # the mirror beside (0: after), shown, parts serial, the players' views beside, the car on a helper, the canvas rows' check
        if os.environ.get(switch):
            renderer += f'{switch}={int(os.environ[switch] != "0")} '
    if HOST:
        ssh(f'rm -rf {FOLDER}; mkdir -p {FOLDER}; cd {DEVICE_GAME}; '
            '[ -d .remote_kept ] || { mkdir .remote_kept; cp -r var savedata .remote_kept/; }; '
            'pgrep -f loadapp.sh | head -1 > /tmp/rc2k_paused; pgrep -f dmenu_ln | head -1 >> /tmp/rc2k_paused; '
            'kill -STOP $(cat /tmp/rc2k_paused); pkill dmenu.bin; sleep 1; '
            f'RC2K_REMOTE={FOLDER} {renderer} nohup ./rc2k_remote > rc2k.out 2>&1 < /dev/null & echo started')
    else:
        shutil.rmtree(FOLDER, ignore_errors=True)
        os.makedirs(FOLDER)
        if not os.path.exists(GAME + '/files'):
            os.symlink(INSTALLED + '/files', GAME + '/files')
        for part in ('var', 'savedata'):
            if fresh:
                shutil.rmtree(f'{GAME}/{part}', ignore_errors=True)
            if not os.path.exists(f'{GAME}/{part}'):
                shutil.copytree(f'{INSTALLED}/{part}', f'{GAME}/{part}')
        env = dict(os.environ, RC2K_REMOTE=FOLDER)
        if software:
            env['RC2K_RENDERER'] = 'software'
        if os.environ.get('RC2K_FAIL_RENDERER'):
            env['RC2K_FAIL_RENDERER'] = os.environ['RC2K_FAIL_RENDERER']
        command = ['./rc2k']
        if DARLING:
            env['DPREFIX'] = DARLING
            # SDL's controller back ends lean on frameworks Darling only stubs.
            passed = {k: v for k, v in env.items() if k.startswith(('RC2K_', 'SDL_'))}
            passed.update(RC2K_REMOTE='/Volumes/SystemRoot' + FOLDER, SDL_JOYSTICK_MFI='0',
                          SDL_JOYSTICK_HIDAPI='0', SDL_JOYSTICK_IOKIT='0', SDL_HAPTIC='0')
            command = ['darling', 'shell', 'env'] + [f'{k}={v}' for k, v in passed.items()] + command
        with open(HOME + '/rc2k.out', 'w') as out:
            process = subprocess.Popen(command, cwd=GAME, env=env, stdout=out, stderr=out, start_new_session=True)
        open(HOME + '/pid', 'w').write(str(process.pid))
    print('started')


def stop():
    if HOST:
        try:
            put_cmd('0 quit')
        finally:
            time.sleep(3)
            ssh(f'pkill rc2k_remote; sleep 1; cd {DEVICE_GAME}; [ -d .remote_kept ] && '
                '{ rm -rf var savedata; mv .remote_kept/var .remote_kept/savedata .; rmdir .remote_kept; }; '
                'kill -CONT $(cat /tmp/rc2k_paused)')
    else:
        put_cmd('0 quit')
        end = time.time() + 3  # up to 3 s for it to quit by itself
        while time.time() < end and running():
            time.sleep(0.05)
        try:
            pid = int(open(HOME + '/pid').read())
            found = process(pid)
            if found and found[1] == 'rc2k':
                os.kill(pid, 15)
        except (OSError, ValueError):
            pass
        if DARLING and (pid := darling_game()):
            os.kill(pid, 15)
    print('stopped')


def pilot(seconds):
    """The game's own computer driver at the player's car's wheel (`drive on`: the physics as
    for the player, only the controls the computer's), a trace every two seconds."""
    ask('drive on')
    end = time.time() + seconds
    raced = False  # a race seen: until then the stage may still be loading
    try:
        while time.time() < end:
            if os.environ.get('RC2K_PILOT_TIME'):  # keeps the arcade's countdown up
                ask('time ' + os.environ['RC2K_PILOT_TIME'])
            fields = dict(f.split('=', 1) for f in ask('state').split() if '=' in f)
            raced = raced or fields.get('race') == '1'
            if raced and fields.get('race') not in (None, '1'):
                print('race over', fields.get('time'), 'splits', fields.get('splits'), flush=True)
                break
            if 'speed' in fields:
                print(fields['time'], 'cp', fields['checkpoint'], 'of', fields.get('checkpoints'), 'speed', fields['speed'], 'finished', fields.get('finished'),
                      'splits', fields['splits'], flush=True)
            time.sleep(2)
    finally:
        ask('drive off')


def fields_of(text):
    return dict(f.split('=', 1) for f in text.split() if '=' in f)


def holds(fields, cond):
    for op in ('!=', '>=', '<=', '=', '>', '<'):
        if op in cond:
            name, want = cond.split(op, 1)
            break
    else:
        sys.exit(f'wait: no operator in "{cond}"')
    have = fields.get(name)
    if have is None:
        return False
    if op in ('=', '!='):
        return (have == want) == (op == '=')
    try:
        a, b = float(have), float(want)
    except ValueError:
        return False
    return {'>=': a >= b, '<=': a <= b, '>': a > b, '<': a < b}[op]


def wait(conds, seconds):
    """Polls `state` (every 50 ms) until every condition holds; returns the state, or None on
    timeout (the last state is printed to stderr)."""
    end = time.time() + seconds
    while True:
        text = ask('state')
        fields = fields_of(text)
        if all(holds(fields, c) for c in conds):
            return text
        if time.time() >= end:
            print('wait: timed out at', text[:120], file=sys.stderr)
            return None
        time.sleep(0.05)


# Frames a screen has drawn before it takes keys (the menus may ignore keys during a fade).
SETTLE = int(os.environ.get('RC2K_SETTLE', '8'))


def press(name, seconds, ms='100'):
    """A key, then until the screen reports another menu, item or race flag and has drawn SETTLE
    frames; True when it changed (a key the game ignores waits the whole time)."""
    before = fields_of(ask('state'))
    ask(f'key {name} {ms}')
    end = time.time() + seconds
    changed = False
    while time.time() < end:
        now = fields_of(ask('state'))
        if any(now.get(k) != before.get(k) for k in ('menu', 'item', 'race')):
            changed = True
            if int(now.get('screen_frames', 0)) >= SETTLE:
                break
        time.sleep(0.05)
    print(ask('state')[:60])
    return changed


def log(lines):
    if HOST:
        text = ssh(f'tail -n {lines} {DEVICE_GAME}/rc2k.log').stdout.decode(errors='replace')
    else:
        text = ''.join(open(GAME + '/rc2k.log', errors='replace').readlines()[-lines:])
    print(text, end='')


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    command, rest = args[0], args[1:]
    if command == 'build':
        build()
    elif command == 'start':
        start('--software' in rest, '--fresh' in rest, '--profile' in rest)
    elif command == 'stop':
        stop()
    elif command == 'shot':
        shot(rest[0])
    elif command == 'pilot':
        pilot(float(rest[0]))
    elif command == 'drive' and rest in (['on'], ['off']):
        print(ask('drive ' + rest[0]))
    elif command == 'wait':
        seconds = float(rest.pop()) if rest and rest[-1].replace('.', '').isdigit() else 10.0
        text = wait(rest, seconds)
        print(text[:80] if text else '')
        sys.exit(0 if text else 1)
    elif command == 'press':
        sys.exit(0 if press(rest[0], float(rest[1]) if len(rest) > 1 else 5.0) else 1)
    elif command == 'log':
        log(int(rest[0]) if rest else 30)
    elif command in ('key', 'down', 'up', 'pad', 'joy', 'state', 'time'):
        print(ask(' '.join([command] + rest)))
    else:
        sys.exit(__doc__)


main()
