#!/usr/bin/env python3
"""Drives a `remote` build of the game for automated tests (src/remote.rs).

    remote.py build [--handheld]         builds the binary with the `remote` feature
    remote.py start [--gpu] [--fresh]    starts it (PC: on a copy of var and savedata)
    remote.py key Return [ms]            a key press (SDL's scancode names)
    remote.py down W / up W              a key held
    remote.py pad a [ms]                 a gamepad button (menus)
    remote.py shot name                  the next frame to <folder>/shots/name.png
    remote.py time n                     the arcade's countdown set to n
    remote.py state                      frames, race flag, menu, item, speed, time, place
    remote.py pilot seconds              drives along the stage line, printing a trace
    remote.py log [n]                    the game's log, last n lines
    remote.py stop                       quits, then ends it

--handheld (or RC2K_REMOTE_HOST=root@address) drives the handheld over SSH. There var and
savedata are put back on stop, as the folder is the real one.
"""
import math, os, shutil, struct, subprocess, sys, time, zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOME = os.path.expanduser('~/.cache/rc2k_remote')
GAME = HOME + '/game'
INSTALLED = os.path.expanduser('~/.local/share/rally-championship')
SHOTS = HOME + '/shots'
DEVICE_GAME = '/mnt/mmc/Roms/APPS/rally'
DEVICE_REMOTE = '/tmp/rc2k_remote'
KEY = os.path.expanduser('~/.ssh/rg34xxsp')


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


def running():
    if HOST:
        return ssh('pgrep -x rc2k_remote').returncode == 0
    try:
        return open(f"/proc/{open(HOME + '/pid').read()}/stat").read().split(') ')[1][0] != 'Z'
    except OSError:
        return False


def ask(command, timeout=20):
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
    rows = []
    for y in range(0, height, step):
        row = bytearray([0])
        at = y * width * 4
        line = raw[at:at + width * 4]
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
        env = dict(os.environ, CARGO_TARGET_DIR='target/linux-remote')
        subprocess.run(['cargo', 'build', '--release', '--target', 'x86_64-unknown-linux-gnu', '--features', 'remote',
                        '--bin', 'rc2k'], cwd=ROOT, env=env, check=True)
        os.makedirs(GAME, exist_ok=True)
        shutil.copy(ROOT + '/target/linux-remote/x86_64-unknown-linux-gnu/release/rc2k', GAME + '/rc2k.new')
        os.replace(GAME + '/rc2k.new', GAME + '/rc2k')


def start(gpu, fresh):
    renderer = 'RC2K_RENDERER=gpu ' if gpu else ''
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
        if gpu:
            env['RC2K_RENDERER'] = 'gpu'
        with open(HOME + '/rc2k.out', 'w') as out:
            process = subprocess.Popen(['./rc2k'], cwd=GAME, env=env, stdout=out, stderr=out, start_new_session=True)
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
        time.sleep(3)
        try:
            pid = int(open(HOME + '/pid').read())
            if open(f'/proc/{pid}/comm').read().strip() == 'rc2k':
                os.kill(pid, 15)
        except (OSError, ValueError):
            pass
    print('stopped')


def pilot(seconds):
    """Holds the stage line: heading from the car's matrix, Right turns it up, Left down."""
    held = set()

    def hold(key, wanted):
        if wanted != (key in held):
            ask(('down ' if wanted else 'up ') + key)
            (held.add if wanted else held.discard)(key)

    end = time.time() + seconds
    count = 0
    still = time.time() + 5
    backing = 0
    try:
        while time.time() < end:
            if os.environ.get('RC2K_PILOT_TIME'):  # keeps the arcade's countdown up
                ask('time ' + os.environ['RC2K_PILOT_TIME'])
            fields = dict(f.split('=', 1) for f in ask('state').split() if '=' in f)
            if 'race' not in fields:
                continue
            if fields['race'] != '1':
                print('race over', fields)
                break
            if 'speed' not in fields:  # a state read while the game was busy
                time.sleep(0.1)
                continue
            speed = float(fields['speed'])
            x, y = map(float, fields['map'].split(','))
            tx, ty = map(float, fields['target' if speed > 10 else 'near'].split(','))
            matrix = [float(v) for v in fields['matrix'].split(',')]
            heading = math.atan2(-matrix[10], matrix[8])
            error = math.atan2(ty - y, tx - x) - heading
            error = (error + math.pi) % (2 * math.pi) - math.pi
            # Stuck: back out, wheels the other way.
            now = time.time()
            if speed >= 3:
                still = now
            if now - still > 2.5:
                backing = now + 2.5
                still = backing + 2.5
            if now < backing:
                hold('Up', False)
                hold('Down', True)
                hold('Right', error < 0)
                hold('Left', error > 0)
                continue
            hold('Right', error > 0.06)
            hold('Left', error < -0.06)
            braking = abs(error) > 0.35 and speed > 12
            hold('Down', braking)
            hold('Up', not braking)
            count += 1
            if count % 120 == 0:
                print(fields['time'], 'cp', fields['checkpoint'], 'speed', speed,
                      'splits', fields['splits'], 'error', round(error, 2), flush=True)
    finally:
        for key in list(held):
            hold(key, False)


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
        start('--gpu' in rest, '--fresh' in rest)
    elif command == 'stop':
        stop()
    elif command == 'shot':
        shot(rest[0])
    elif command == 'pilot':
        pilot(float(rest[0]))
    elif command == 'log':
        log(int(rest[0]) if rest else 30)
    elif command in ('key', 'down', 'up', 'pad', 'state', 'time'):
        print(ask(' '.join([command] + rest)))
    else:
        sys.exit(__doc__)


main()
