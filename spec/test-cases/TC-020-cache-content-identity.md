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

Verify the cache key binds every meaning input and uses the exact provider
manifest content instead of a recorded revision or version pin.

Scope: FR-012-AC-1 through FR-012-AC-2.

## Test Procedure

Derive keys for prove, analyze and generate requests using canonical manifests.
Hold the operation, package identity, request, provider identity and recorded
pin fixed while changing one manifest byte; then vary each key member in turn.
For generate, vary the target and an output-affecting target option separately.
Finally vary a path, timestamp, wall-clock deadline, recorded pin and execution
backend label without changing any meaning input.

## Expected Results

- A changed manifest byte changes the provider-manifest digest and cache key
  while the recorded pin remains unchanged.
- Each meaning-input change, including a generate target or option change,
  changes the cache key.
- Changes to excluded path, timestamp, deadline, pin and execution-backend
  labels do not change the key.
- A key request whose selected provider identity differs from the identity in
  its manifest is rejected before the key is returned.
