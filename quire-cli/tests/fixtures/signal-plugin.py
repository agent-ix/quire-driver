#!/usr/bin/env python3
"""Signal lifecycle fixture: hello, request, and shutdown can each block."""

import json
import os
import signal
import struct
import select
import subprocess
import sys
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


mode, ready_path, descendant_path, plugin_path = sys.argv[1:]
protocol = "quire.plugin-wire/v1"

if mode == "hello":
    open(ready_path, "wb").close()
    os.kill(os.getppid(), signal.SIGTERM)
    time.sleep(2)
    open(plugin_path, "wb").close()

write_frame(
    {
        "protocol": protocol,
        "id": 0,
        "kind": "hello",
        "body": {"identity": "agent-ix/provider/signal-test", "advertises": []},
    }
)

while True:
    request = read_frame()
    if request["kind"] == "shutdown":
        if mode == "shutdown":
            subprocess.Popen(
                [
                    sys.executable,
                    "-c",
                    "import pathlib,time,sys; time.sleep(0.6); pathlib.Path(sys.argv[1]).write_text('leaked')",
                    descendant_path,
                ]
            )
            open(ready_path, "wb").close()
            os.kill(os.getppid(), signal.SIGTERM)
            time.sleep(2)
            open(plugin_path, "wb").close()
        break
    if request["kind"] != "request":
        raise SystemExit(3)
    if mode in ("request", "request-int"):
        subprocess.Popen(
            [
                sys.executable,
                "-c",
                "import pathlib,time,sys; time.sleep(0.6); pathlib.Path(sys.argv[1]).write_text('leaked')",
                descendant_path,
            ]
        )
        open(ready_path, "wb").close()
        os.kill(os.getppid(), signal.SIGINT if mode == "request-int" else signal.SIGTERM)
        readable, _, _ = select.select([sys.stdin.buffer], [], [], 2)
        if readable:
            control = read_frame()
            if control["kind"] == "cancel":
                time.sleep(2)
        open(plugin_path, "wb").close()
    write_frame(
        {
            "protocol": protocol,
            "id": request["id"],
            "kind": "result",
            "body": {"identity": "agent-ix/provider/signal-test", "items": []},
        }
    )
