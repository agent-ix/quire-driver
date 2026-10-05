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

Scope: FR-012-AC-1 through FR-012-AC-2.

## Test Procedure

Derive keys for prove, analyze and generate requests. Hold the operation,
package identity, request, provider identity and recorded pin fixed while
changing manifest bytes and the resulting `ManifestDigest`; then vary each key
member in turn. For generate, vary the target and an output-affecting option
separately, and verify the target member is absent for prove/analyze. Vary path,
timestamp, wall-clock deadline, recorded pin and execution-backend label without
changing meaning inputs. Attempt registry admission with a manifest whose
identity does not match the selected `BackendId`, then verify that no
`CacheRequest` is constructed.

## Expected Results

- Changed manifest content changes the admitted `ManifestDigest` member and
  cache key even when identity and pin remain unchanged.
- Each meaning-input change, including a generate target or option, changes the
  key; the target member occurs only for generate.
- Changes to excluded path, timestamp, deadline, pin and execution-backend
  labels do not change the key.
- A selected `BackendId`/manifest identity mismatch is rejected during
  registry admission before a `CacheRequest` is constructed.
