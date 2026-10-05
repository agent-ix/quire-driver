#!/usr/bin/env python3
"""Small PL-4 process fixture used by the quire-plugin-host tests."""

import json
import struct
import sys


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
write_frame(
    {
        "protocol": protocol,
        "id": 0,
        "kind": "hello",
        "body": {"identity": "agent-ix/provider/test", "advertises": []},
    }
)
request = read_frame()
write_frame(
    {
        "protocol": protocol,
        "id": request["id"],
        "kind": "result",
        "body": {
            "identity": "agent-ix/provider/test",
            "items": request["body"]["items"],
        },
    }
)
shutdown = read_frame()
if shutdown["kind"] != "shutdown":
    raise SystemExit(3)
sys.stderr.write("opaque provider diagnostic\n")
