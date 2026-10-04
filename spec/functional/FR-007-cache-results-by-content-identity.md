---
id: FR-007
title: "Cache results by content identity"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-007: Cache results by content identity

## Description

When the caller enables the cache for an eligible operation, the driver SHALL
key the entry by the canonical content identity of every input that can
change the result. The driver SHALL compute the provider contribution from
its manifest bytes. A recorded repository revision or version pin SHALL NOT
stand in for manifest content identity.

## Inputs

- The operation and subject `package_id`.
- The canonical request, including requested items, claims, domains, bounds,
  result-affecting options and deterministic budgets.
- The selected provider's manifest bytes and, for generation, the target and
  target options.
- The caller-named cache directory and size limit.

## Outputs

- A verified cached terminal-record result or generated artifact, or a cache
  miss followed by fresh execution.

## Behavior

- The cache SHALL store eligible `prove` and `analyze` terminal records and
  `generate` artifacts.
- The driver SHALL run parsing, checking, packaging, execution and monitoring
  on every request.
- The key SHALL be a canonical `quire.cache-key/v1` content identity over the
  JCS preimage of the operation, subject package identity, canonical request,
  provider manifest content identity and applicable generation target and
  options. It SHALL NOT use a recorded pin, path, timestamp, application
  version, wall-clock deadline or execution backend as a substitute for
  content inputs.
- When any meaning input changes, the cache SHALL produce a miss.
- Inputs outside the key SHALL NOT invalidate a matching entry.
- The cache reader SHALL accept an entry only when its name, the content
  identity of its stored preimage and the requested key agree and its value
  passes the same typed reader used for a fresh result.
- The cache reader SHALL treat an entry with an invalid name, preimage, key or
  typed-reader result as a miss.
- The cache writer SHALL use a temporary name and an atomic rename.
- When two writers store one key, each SHALL write identical bytes.
- The cache SHALL remain within the caller's size limit.
- The driver SHALL NOT store internal failures, cancelled or deadline
  results, time-limited incomplete results, wall-time-bearing values or plugin
  results.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | Equal canonical operation, subject, request, provider-manifest bytes and applicable generation options produce the same cache key; changing the manifest bytes changes the key without consulting a recorded pin. | Test |
| FR-007-AC-2 | Changing any meaning input causes a miss, while changing a path, timestamp, application version, wall-clock deadline or execution backend alone does not invalidate an otherwise matching entry. | Test |
| FR-007-AC-3 | A stored entry is accepted only when its name, stored-preimage identity and requested key agree and its value passes the fresh-result typed reader; each mismatch or parse failure is a miss. | Test |
| FR-007-AC-4 | Concurrent writes for one key publish identical bytes atomically, and eviction keeps the named store at or below its configured size limit. | Test |
| FR-007-AC-5 | The cache never stores internal failures, cancelled or deadline results, time-limited incomplete results, wall-time-bearing values or plugin results. | Test |
| FR-007-AC-6 | Parse, check, package, execute and monitor operations run on every request, while eligible prove, analyze and generate results can be served from a verified entry. | Test |

## Dependencies

- QSL ADR-029 CA-1 through CA-5 and QSpec FR-201 and FR-306.
- [TC-014](../test-cases/TC-014-cache-content-identity-and-validation.md).
