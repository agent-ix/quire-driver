---
id: TC-020
title: "Derive cache identity from meaning inputs"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-012
    type: verifies
---
# TC-020: Derive cache identity from meaning inputs

## Description

Verify the cache key binds every meaning input and uses the QSL-admitted
manifest content identity instead of a recorded revision or version pin.

Scope: FR-012-AC-1 through FR-012-AC-5.

## Test Procedure

Derive keys for prove, analyze and generate requests. Then vary each meaning key member in turn. For
generate, vary the target and an output-affecting option separately, and verify
the target member is absent for prove/analyze. Vary the explicitly excluded
top-level path, timestamp, wall-clock deadline, recorded pin, revision,
application version, wall time and execution-backend label without changing
meaning inputs. Also vary a nested request field named `path` and verify that
it remains a meaning input.

## Expected Results

- Each meaning-input change, including a generate target or option, changes the
  key; the target member occurs only for generate.
- Changes to excluded top-level path, timestamp, deadline, pin, revision,
  application version, wall time and execution-backend labels do not change the
  key; a nested request field named `path` changes the key.
