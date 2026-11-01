#!/usr/bin/env python3
"""Run an idle-app binary (bench/idle.m31 or bench/ratatui/src/bin/idle.rs) on a pty and measure it.

    python3 -I bench/idle.py <label> <runs> <binary> [ENV=VALUE ...]

For each run: start-up latency (spawn to the first frame on the pty), the latency from a key (`x`) to the frame
that shows it (5 keys), the latency from a window resize (TIOCSWINSZ + SIGWINCH, as a terminal emulator does it)
to the frame in the new size (4 resizes), the CPU time the process uses while idle for 1 s (user + system, from
/proc), and its peak RSS (wait4). One line per run: `RUN <label> startup_ms=... rss_kb=... idle_cpu_ms=...
key_ms=a,b,c,d,e resize_ms=a,b,c,d`. Linux only (it reads /proc); nothing is asserted.
"""
import fcntl, os, pty, select, signal, struct, subprocess, sys, termios, time

def set_size(fd, cols, rows):
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))

def wait_for(fd, needle, buf, timeout):
    end = time.monotonic() + timeout
    while needle not in buf:
        left = end - time.monotonic()
        if left <= 0:
            return None
        r, _, _ = select.select([fd], [], [], left)
        if r:
            try:
                data = os.read(fd, 65536)
            except OSError:
                return None
            if not data:
                return None
            buf += data
    return time.monotonic()

def cpu_ms(pid):
    with open("/proc/%d/stat" % pid) as f:
        parts = f.read().rsplit(")", 1)[1].split()
    ticks = int(parts[11]) + int(parts[12])
    return ticks * 1000.0 / os.sysconf("SC_CLK_TCK")

def one_run(binary, env_extra):
    master, slave = os.openpty()
    set_size(slave, 200, 50)
    env = dict(os.environ, TERM="xterm-256color")
    env.update(env_extra)
    def child():
        os.setsid()
        fcntl.ioctl(0, termios.TIOCSCTTY, 0)
    t0 = time.monotonic()
    p = subprocess.Popen([binary], stdin=slave, stdout=slave, stderr=slave, env=env, preexec_fn=child, close_fds=True)
    os.close(slave)
    buf = b""
    t = wait_for(master, b"READY-p-200x50", buf, 10)
    if t is None:
        p.kill(); p.wait()
        raise SystemExit("no first frame from " + binary)
    startup = (t - t0) * 1000
    time.sleep(0.3)
    keys = []
    for k in range(1, 6):
        buf = b""
        s = time.monotonic()
        os.write(master, b"x")
        t = wait_for(master, "pstuvw"[k:k + 1].encode(), buf, 5)
        keys.append((t - s) * 1000 if t else -1)
        time.sleep(0.15)
    resizes = []
    for cols, rows in ((120, 40), (200, 50), (90, 30), (200, 50)):
        buf = b""
        s = time.monotonic()
        set_size(master, cols, rows)
        os.kill(p.pid, signal.SIGWINCH)
        t = wait_for(master, ("-%dx%d" % (cols, rows)).encode(), buf, 5)
        resizes.append((t - s) * 1000 if t else -1)
        time.sleep(0.15)
    # drain, then idle for one second
    while select.select([master], [], [], 0.3)[0]:
        os.read(master, 65536)
    c0 = cpu_ms(p.pid)
    time.sleep(1.0)
    idle = cpu_ms(p.pid) - c0
    os.write(master, b"q")
    _, status, usage = os.wait4(p.pid, 0)
    os.close(master)
    return startup, usage.ru_maxrss, idle, keys, resizes

def main():
    label, runs, binary = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    env_extra = dict(a.split("=", 1) for a in sys.argv[4:])
    for _ in range(runs):
        startup, rss, idle, keys, resizes = one_run(binary, env_extra)
        print("RUN %s startup_ms=%.1f rss_kb=%d idle_cpu_ms=%.0f key_ms=%s resize_ms=%s" % (
            label, startup, rss, idle, ",".join("%.1f" % k for k in keys), ",".join("%.1f" % r for r in resizes)))
        sys.stdout.flush()

main()
