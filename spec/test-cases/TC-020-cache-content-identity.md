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

Verify the cache key (a structural preimage, no hash) binds every meaning input
and records no revision, version pin or manifest digest.

Scope: FR-012-AC-1 through FR-012-AC-5.

## Test Procedure

Derive keys for prove, analyze and generate requests. Vary each meaning key member in turn. For
generate, vary the target and an output-affecting option separately, and verify
the target member is absent for prove/analyze. Vary the explicitly excluded
top-level path, timestamp, wall-clock deadline, recorded pin, revision,
application version, wall time and execution-backend label without changing
meaning inputs. Also vary a nested request field named `path` and verify that
it remains a meaning input.

## Expected Results

- The key holds no digest, hash or version member.
- Each meaning-input change, including a generate target or option, changes the
  key; the target member occurs only for generate.
- Changes to excluded top-level path, timestamp, deadline, pin, revision,
  application version, wall time and execution-backend labels do not change the
  key; a nested request field named `path` changes the key.
