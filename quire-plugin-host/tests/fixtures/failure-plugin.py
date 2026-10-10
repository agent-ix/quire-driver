#!/usr/bin/env python3
"""Deterministic PL-4 failure and resource-bound fixture."""

import json
import os
import struct
import subprocess
import sys
import threading
import time


def read_frame():
    prefix = sys.stdin.buffer.read(8)
    if len(prefix) != 8:
        raise SystemExit(2)
    length = struct.unpack(">Q", prefix)[0]
    payload = sys.stdin.buffer.read(length)
    if len(payload) != length:
        raise SystemExit(2)
    return json.loads(payload)


def write_frame(value):
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    sys.stdout.buffer.write(struct.pack(">Q", len(payload)))
    sys.stdout.buffer.write(payload)
    sys.stdout.buffer.flush()


protocol = "quire.plugin-wire/v1"
mode = sys.argv[1]
if mode == "wrong-protocol":
    protocol = "quire.plugin-wire/v0"
if mode == "thread-tasks":
    for _ in range(4):
        threading.Thread(target=lambda: time.sleep(60), daemon=True).start()
if mode == "descendant-memory":
    ready = sys.argv[2]
    helper = """import os, sys, time
first = os.fork()
if first:
    os.waitpid(first, 0)
    sys.exit(0)
second = os.fork()
if second:
    os._exit(0)
retained = bytearray(32 * 1024 * 1024)
for offset in range(0, len(retained), 4096):
    retained[offset] = 1
with open(sys.argv[1], 'wb') as marker:
    marker.write(b'ready')
time.sleep(60)
"""
    for index in range(3):
        child_ready = f"{ready}.{index}"
        subprocess.Popen([sys.executable, "-c", helper, child_ready], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    deadline = time.monotonic() + 3
    while not all(os.path.exists(f"{ready}.{index}") for index in range(3)) and time.monotonic() < deadline:
        time.sleep(0.005)
    if not all(os.path.exists(f"{ready}.{index}") for index in range(3)):
        raise SystemExit(5)
if mode == "escaped-descendant-memory":
    ready, writing, stop, done = sys.argv[2:6]
    helper = """import json, os, sys, time
host_pgid = int(sys.argv[5])
retained = bytearray(32 * 1024 * 1024)
for offset in range(0, len(retained), 4096):
    retained[offset] = 1
payload = json.dumps({'pgid': os.getpgrp(), 'host_pgid': host_pgid}).encode()
temporary = sys.argv[1] + '.tmp-' + str(os.getpid())
with open(temporary, 'wb') as marker:
    marker.write(payload[:8])
    marker.flush()
    os.fsync(marker.fileno())
    with open(sys.argv[2], 'wb') as progress:
        progress.write(b'partial')
    time.sleep(0.2)
    marker.write(payload[8:])
    marker.flush()
    os.fsync(marker.fileno())
os.replace(temporary, sys.argv[1])
deadline = time.monotonic() + 15
while not os.path.exists(sys.argv[3]) and time.monotonic() < deadline:
    time.sleep(0.01)
with open(sys.argv[4], 'wb') as marker:
    marker.write(b'done')
"""
    subprocess.Popen(
        [sys.executable, "-c", helper, ready, writing, stop, done, str(os.getpid())],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
if mode == "request-marker":
    with open(sys.argv[2] + ".ready", "wb") as ready:
        ready.write(b"waiting")
if mode == "fork-during-request":
    helper = "import time; time.sleep(0.2)"
if mode == "eof-before-hello":
    raise SystemExit(17)
if mode == "truncated-hello":
    sys.stdout.buffer.write(b"\x00\x00")
    sys.stdout.buffer.flush()
    raise SystemExit(17)

write_frame(
    {
        "protocol": protocol,
        "id": 0,
        "kind": "hello",
        "body": {"identity": "agent-ix/provider/test", "advertises": []},
    }
)

if mode == "blocked-write":
    with open(sys.argv[2], "wb") as ready:
        ready.write(b"ready")
    time.sleep(60)
    raise SystemExit(0)

if mode == "stderr":
    sys.stderr.buffer.write(b"x" * 4096)
    sys.stderr.flush()

request = read_frame()
if mode == "escaped-descendant-memory":
    with open(sys.argv[5], "wb") as marker:
        marker.write(b"request")
if mode in ("descendant-hang", "detached-stdio"):
    streams = {} if mode == "descendant-hang" else {"stdin": subprocess.DEVNULL, "stdout": subprocess.DEVNULL, "stderr": subprocess.DEVNULL}
    subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], **streams)
    write_frame({"protocol": protocol, "id": request["id"], "kind": "result", "body": {"identity": "agent-ix/provider/test", "items": request["body"]["items"]}})
    shutdown = read_frame()
    if shutdown["kind"] != "shutdown":
        raise SystemExit(3)
    raise SystemExit(0)
if mode == "request-marker":
    with open(sys.argv[2], "wb") as marker:
        marker.write(b"request")
if mode == "fork-during-request":
    for _ in range(3):
        subprocess.Popen([sys.executable, "-c", helper], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(0.12)
if mode == "descendant-cpu":
    subprocess.Popen([sys.executable, "-c", "import time; end=time.process_time()+0.2\nwhile time.process_time()<end: pass"])
    time.sleep(0.3)
    write_frame({"protocol": protocol, "id": request["id"], "kind": "result", "body": {"identity": "agent-ix/provider/test", "items": request["body"]["items"]}})
    raise SystemExit(0)
if mode == "nonzero":
    sys.stderr.buffer.write(b"plugin exited with failure\n")
    sys.stderr.flush()
    raise SystemExit(17)
if mode == "cpu":
    while True:
        pass
if mode == "memory":
    retained = bytearray(64 * 1024 * 1024)
    retained[0] = 1
if mode == "wrong-id":
    request["id"] += 1
if mode == "fault":
    write_frame(
        {
            "protocol": protocol,
            "id": request["id"],
            "kind": "fault",
            "body": {"message": "fixture fault"},
        }
    )
    raise SystemExit(0)
if mode == "fault-after-cancel":
    with open(sys.argv[2], "wb") as ready:
        ready.write(b"ready")
    cancel = read_frame()
    if cancel["kind"] != "cancel":
        raise SystemExit(4)
    write_frame(
        {
            "protocol": protocol,
            "id": request["id"],
            "kind": "fault",
            "body": {"message": "fault during cancellation grace"},
        }
    )
    raise SystemExit(0)
if mode == "ignore-cancel":
    subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
    with open(sys.argv[2], "wb") as ready:
        ready.write(b"ready")
    cancel = read_frame()
    if cancel["kind"] != "cancel":
        raise SystemExit(4)
    time.sleep(60)

result = {
    "identity": "wrong/provider" if mode == "invalid-result" else "agent-ix/provider/test",
    "items": request["body"]["items"],
}
write_frame(
    {
        "protocol": protocol,
        "id": request["id"],
        "kind": "result",
        "body": result,
    }
)
shutdown = read_frame()
if shutdown["kind"] != "shutdown":
    raise SystemExit(3)
