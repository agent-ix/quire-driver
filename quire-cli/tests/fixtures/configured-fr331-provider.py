#!/usr/bin/env python3
"""FR-331 configured-provider fixture that records real request dispatch."""

import json
import os
import select
import signal
import struct
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


protocol = "quire.plugin-wire/v1"
identity = sys.argv[1]
marker = sys.argv[2]
capability = sys.argv[3]
mode = sys.argv[4] if len(sys.argv) > 4 else "normal"
ready_path = sys.argv[5] if len(sys.argv) > 5 else None
release_path = sys.argv[6] if len(sys.argv) > 6 else None
descendant_path = sys.argv[7] if len(sys.argv) > 7 else None
shutdown_path = sys.argv[8] if len(sys.argv) > 8 else (sys.argv[5] if mode == "normal" and len(sys.argv) > 5 else None)
write_frame(
    {
        "protocol": protocol,
        "id": 0,
        "kind": "hello",
        "body": {
            "contract_version": "quire.backend-provider/v1",
            "capability_vocabulary": "quire.capability-kind/v1",
            "manifest": [
                {
                    "backend": identity,
                    "advertises": [{"kind": capability, "mode": "bounded"}],
                    "domains": ["integer"],
                }
            ],
        },
    }
)

while True:
    envelope = read_frame()
    if envelope["kind"] == "shutdown":
        if shutdown_path is not None:
            with open(shutdown_path, "wb") as shutdown:
                shutdown.write(b"closed")
        break
    if envelope["kind"] != "request":
        raise SystemExit(3)
    request = envelope["body"]
    items = request["request"]["items"]
    with open(marker, "wb") as dispatched:
        dispatched.write(json.dumps(request, sort_keys=True).encode())
    if mode in ("signal", "signal-int"):
        subprocess.Popen(
            [
                sys.executable,
                "-c",
                "import pathlib,sys,time; ready,release,done=map(pathlib.Path,sys.argv[1:]); "
                "ready.touch(); deadline=time.monotonic()+10; "
                "\nwhile not release.exists() and time.monotonic()<deadline: time.sleep(0.002) "
                "\nif release.exists(): done.write_text('leaked')",
                ready_path,
                release_path,
                descendant_path,
            ]
        )
        deadline = time.monotonic() + 5
        while not os.path.exists(ready_path) and time.monotonic() < deadline:
            time.sleep(0.002)
        if not os.path.exists(ready_path):
            raise SystemExit(4)
        selected_signal = signal.SIGINT if mode == "signal-int" else signal.SIGTERM
        os.kill(os.getppid(), selected_signal)
        readable, _, _ = select.select([sys.stdin.buffer], [], [], 2)
        if readable:
            control = read_frame()
            if control["kind"] == "cancel":
                time.sleep(2)
        raise SystemExit(0)
    write_frame(
        {
            "protocol": protocol,
            "id": envelope["id"],
            "kind": "result",
            "body": {
                "contract_version": "quire.backend-provider/v1",
                "capability_vocabulary": "quire.capability-kind/v1",
                "results": [
                    {
                        "request_index": item["request_index"],
                        "backend": identity,
                        "kind": item["kind"],
                        "result": "tested",
                    }
                    for item in items
                ],
                "dispositions": [],
            },
        }
    )

sys.stderr.write("configured provider completed\n")
