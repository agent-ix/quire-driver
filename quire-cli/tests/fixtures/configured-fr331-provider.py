#!/usr/bin/env python3
"""FR-331 configured-provider fixture that records real request dispatch."""

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
identity = sys.argv[1]
marker = sys.argv[2]
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
                    "advertises": [{"kind": "value-validity", "mode": "bounded"}],
                    "domains": ["integer"],
                }
            ],
        },
    }
)

while True:
    envelope = read_frame()
    if envelope["kind"] == "shutdown":
        break
    if envelope["kind"] != "request":
        raise SystemExit(3)
    request = envelope["body"]
    items = request["request"]["items"]
    with open(marker, "wb") as dispatched:
        dispatched.write(json.dumps(request, sort_keys=True).encode())
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
