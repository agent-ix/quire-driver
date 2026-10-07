#!/usr/bin/env python3
"""Deterministic PL-4 failure and resource-bound fixture."""

import json
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
if mode == "descendant-hang":
    subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
    write_frame({"protocol": protocol, "id": request["id"], "kind": "result", "body": {"identity": "agent-ix/provider/test", "items": request["body"]["items"]}})
    shutdown = read_frame()
    if shutdown["kind"] != "shutdown":
        raise SystemExit(3)
    raise SystemExit(0)
if mode == "descendant-memory":
    subprocess.Popen([sys.executable, "-c", "import time; data=bytearray(67108864); data[0]=1; time.sleep(60)"])
    time.sleep(0.1)
    write_frame({"protocol": protocol, "id": request["id"], "kind": "result", "body": {"identity": "agent-ix/provider/test", "items": request["body"]["items"]}})
    raise SystemExit(0)
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
