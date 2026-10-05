---
id: FR-012
title: "Implement the downstream content-addressed cache"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-driver/FR-007
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-012: Implement the downstream content-addressed cache

## Description

When a result is eligible for reuse, the `quire-cache` component SHALL derive
and verify one content key from every meaning input required by QSL ADR-029
CA-2 and SHALL store only values permitted by CA-5.

## Inputs

- The operation, package identity and canonical typed request.
- The selected backend identity and its exact canonical FR-331 manifest bytes.
- For generation, the target and every target option that can change output.
- The caller-named store directory, entry-size limit and total-store limit.
- A typed value reader matching the reader used for a fresh result.

## Outputs

- A verified typed cache hit or a miss.
- A stored, never-stored or over-capacity insertion outcome.

## Behavior

- The cache SHALL compute the backend manifest content digest from the exact
  canonical manifest bytes under `quire.tool-manifest.jcs/v1` and bind that
  digest and the backend identity into `quire.cache-key/v1`.
- The cache SHALL include the operation, package identity, canonical request
  and, for generation, the target and its options in the key preimage.
- The cache SHALL NOT use a recorded revision, version, path, timestamp,
  wall-clock deadline, application version or execution backend as a key
  input.
- The cache SHALL treat an absent, malformed, noncanonical, size-exceeding,
  mismatched or incorrectly typed entry as a miss.
- The cache SHALL write an eligible entry to a temporary name and atomically
  rename it into the caller-named store.
- The cache SHALL keep stored entries within the caller's total size limit.
- The cache SHALL NOT store internal failures, cancellations, deadlines,
  time-limited incomplete results, plugin results or values that record wall
  time.
- The `quire-cache` package SHALL remain downstream of and separate from the
  `quire-driver` package; `quire-driver` SHALL NOT depend on `quire-cache`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-012-AC-1 | Two keys with the same operation, package identity, canonical request, provider identity and recorded pin but different exact manifest bytes have different `quire.cache-key/v1` identities; the preimage binds the `quire.tool-manifest.jcs/v1` digest computed from those bytes. | Test |
| FR-012-AC-2 | Changing any included meaning input changes the key, while changing a path, timestamp, deadline, recorded revision or execution-backend label that is outside the key leaves it unchanged; generate target and options are included only for generate. | Test |
| FR-012-AC-3 | A missing, malformed, noncanonical, mismatched-name, mismatched-preimage, wrong-digest or typed-reader-rejected entry is a miss and is never returned as a hit. | Test |
| FR-012-AC-4 | Internal-failure, cancelled, deadline, time-limited incomplete, plugin and wall-time-bearing values return never-stored and leave no readable cache entry. | Test |
| FR-012-AC-5 | A write is published by atomic rename from a temporary file; lookups never accept temporary names, and a later write removes abandoned temporary files without following symlinks. | Test |
| FR-012-AC-6 | Concurrent writers of the same key leave a valid typed entry and do not fail solely because a competing writer completed first. | Test |
| FR-012-AC-7 | Insertion and eviction keep the named entry set within the caller's total byte limit; an entry that cannot fit is not stored. | Test |

## Dependencies

- QSL ADR-029 CA-1 through CA-5 and QSpec FR-201 and FR-306 define cache
  identity, typed-read verification and storage policy.
- [FR-007](./FR-007-cache-results-by-content-identity.md) defines the driver
  interface composition; this component does not become a dependency of the
  driver core.
- [TC-020](../test-cases/TC-020-cache-content-identity.md),
  [TC-021](../test-cases/TC-021-cache-entry-integrity-and-policy.md) and
  [TC-022](../test-cases/TC-022-cache-storage-bounds-and-concurrency.md)
  verify this component.
