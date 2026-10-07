#!/usr/bin/env python3
"""Small FR-331 manifest-stage fixture for the quire-cli adapter."""

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


identity = "agent-ix/provider/fr331-fixture"
write_frame(
    {
        "protocol": "quire.plugin-wire/v1",
        "id": 0,
        "kind": "hello",
        "body": {
            "contract_version": "quire.backend-provider/v1",
            "capability_vocabulary": "quire.capability-kind/v1",
            "manifest": [
                {
                    "backend": identity,
                    "advertises": [
                        {"kind": "operation-contract", "mode": "bounded"}
                    ],
                    "domains": ["integer"],
                    "bounds": {"max_states": 8},
                }
            ],
        },
    }
)

while True:
    request = read_frame()
    if request["kind"] == "shutdown":
        break
    if request["kind"] != "request":
        raise SystemExit(3)
    write_frame(
        {
            "protocol": "quire.plugin-wire/v1",
            "id": request["id"],
            "kind": "result",
            "body": {
                "identity": identity,
                "items": request["body"].get("items", []),
            },
        }
    )
sys.stderr.write("FR-331 fixture diagnostic\n")
