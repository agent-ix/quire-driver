---
id: TC-020
title: "Derive cache identity from meaning inputs and manifest content"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-012
    type: verifies
---
# TC-020: Derive cache identity from meaning inputs and manifest content

## Description

Verify the cache key binds every meaning input and uses the QSL-admitted
manifest content identity instead of a recorded revision or version pin.

Scope: FR-012-AC-1 through FR-012-AC-5.

## Test Procedure

Derive keys for prove, analyze and generate requests. Hold all other key
members and recorded pin fixed while supplying different admitted
`ManifestDigest` values. Then vary each meaning key member in turn. For
generate, vary the target and an output-affecting option separately, and verify
the target member is absent for prove/analyze. Vary path, timestamp, wall-clock
deadline, recorded pin, application version and execution-backend label without
changing meaning inputs.

## Expected Results

- Different admitted `ManifestDigest` values change the cache key even when
  identity and pin remain unchanged; this test treats the digest as QSL input.
- Each meaning-input change, including a generate target or option, changes the
  key; the target member occurs only for generate.
- Changes to excluded path, timestamp, deadline, pin, revision, application
  version, wall time and execution-backend labels, including nested request
  fields, do not change the key.
