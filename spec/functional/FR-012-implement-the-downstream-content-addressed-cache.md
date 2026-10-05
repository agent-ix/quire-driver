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

The downstream `quire-cache` package SHALL implement the required cache
interface owned by `quire-driver` and SHALL follow QSL ADR-029 CA-1 through
CA-5. It is a separate package and SHALL depend on the driver interface; the
`quire-driver` package SHALL NOT depend on `quire-cache`.

## Inputs

- `CacheRequest`: operation (`prove`, `analyze` or `generate`), package
  identity, canonical typed request, selected `qsl_route::BackendId`, the
  registry-admitted `qsl_route::ManifestDigest`, and for `generate` the target
  and output-affecting options.
- A caller-named storage directory, maximum entry size and total stored-byte
  limit, plus the typed value reader for the requested result.
- An eligible typed result for insertion.

## Outputs

- `CacheLookup::Hit(TypedResult)` after key, entry and typed-reader validation,
  or `CacheLookup::Miss`.
- A typed cache I/O error for operational filesystem failures (for example,
  permission denied or corrupt directory operation); it SHALL NOT be disguised
  as a miss.
- `CacheStore::Stored`, `CacheStore::NeverStore` for QSL CA-5 exclusions, or
  `CacheStore::NotStored` when an entry exceeds the per-entry or total-store
  limit. NeverStore/NotStored do not fail the provider operation.

## Behavior

- The cache key SHALL be SHA-256 over JCS of an object with members `operation`,
  `package_id`, `request`, `backend`, `manifest`, and `target` for `generate`
  only. `backend` is the identity string from `BackendId`; `manifest` is the
  admitted `ManifestDigest`, using QSL's canonical serialization for that
  typed digest. The cache SHALL use the `ManifestDigest` value supplied by the
  registry descriptor and SHALL NOT rehash the manifest or invent a separate
  digest spelling. QSL-636 amends QSpec FR-201/FR-306 and ADR-029 CA-2 to
  require this content identity in the key.
- The cache represents generation target and output-affecting options in the
  `target` member. For other operations, the cache omits that member. When
  deriving a key, the cache SHALL exclude paths, timestamps, deadlines,
  recorded pins/revisions, application version, wall time and execution-backend
  labels.
- A lookup SHALL return a hit only when the named entry is present, canonical,
  within the per-entry limit, has a filename matching the requested key, has a
  preimage and digest matching the requested `CacheRequest`, and is accepted by
  the same typed reader used for a fresh value. An invalid entry is a miss.
- The cache stores only values eligible under ADR-029 CA-5. The cache stores
  deterministic budget-incomplete results when within size limits; time-limited
  incomplete values are not eligible.
- The cache writes each entry to a unique temporary file in the caller's store,
  flushes and closes it, then atomically renames it to its final key name. The
  cache never writes directly to the final name. A process-local lock
  serializes local eviction and insertion. If another process has already
  removed an eviction candidate, the cache ignores `NotFound` and continues
  insertion. Temporary files count toward total-store bytes until removed. The
  cache removes its own abandoned temporary files and inspects entries without
  following symlinks. These filesystem policies are this component's storage
  choices under ADR-029 CA-3 and CA-4; they include the per-entry size bound
  and `NotStored` on an over-capacity insertion.
- The cache returns `NotStored` for an entry above the per-entry limit or one
  that cannot fit the configured total-store limit after eviction. The cache
  uses no index or ledger; it derives keys from entries and request input
  (ADR-029 CA-3).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-012-AC-1 | The key preimage is JCS with operation, package identity, canonical request, backend identity and the QSL-serialized `ManifestDigest`; `generate` also has a target member. | Test (TC-020) |
| FR-012-AC-2 | Given identical other key members, different admitted `ManifestDigest` values produce different cache keys. | Test (TC-020) |
| FR-012-AC-3 | Changing a meaning input changes the derived key. | Test (TC-020) |
| FR-012-AC-4 | Changing excluded path, time, deadline, pin, revision, application version or execution-backend label leaves the key unchanged. | Test (TC-020) |
| FR-012-AC-5 | The `target` member occurs for generate and is absent for prove and analyze. | Test (TC-020) |
| FR-012-AC-6 | A valid present entry accepted by the requested typed reader is returned as a hit; absent or invalid entries are misses. | Test (TC-021) |
| FR-012-AC-7 | ADR-029 CA-5 excluded values return `NeverStore`. | Test (TC-021) |
| FR-012-AC-8 | A deterministic-budget incomplete result within size limits is stored and readable. | Test (TC-021) |
| FR-012-AC-9 | An entry above the per-entry limit returns `NotStored`. | Test (TC-022) |
| FR-012-AC-10 | Atomic rename from a temporary file publishes each entry; direct final-name writes are absent, temporary files count toward capacity, and cleanup removes abandoned temporary files without following symlinks. | Test (TC-022) |
| FR-012-AC-11 | Concurrent same-key writers leave a valid typed entry, and cross-process eviction treats `NotFound` for an already-removed entry as success. | Test (TC-022) |
| FR-012-AC-12 | The cache keeps stored bytes within configured per-entry and total limits, returns `NotStored` when an entry cannot fit, and uses no index or ledger. | Test (TC-022) |
| FR-012-AC-13 | `quire-cache` is separate from `quire-driver`, depends on its interface, and is absent from the driver's transitive normal/build dependency closure. | Test (TC-005) |

## Dependencies

- QSL ADR-029 CA-1 through CA-5 and QSL-636 (QSpec FR-201/FR-306 and ADR-029
  CA-2 cache-key clarification).
- [FR-007](./FR-007-cache-results-by-content-identity.md) defines the required
  driver-owned interface; this package implements it downstream.
- [TC-020](../test-cases/TC-020-cache-content-identity.md),
  [TC-021](../test-cases/TC-021-cache-entry-integrity-and-policy.md),
  [TC-022](../test-cases/TC-022-cache-storage-bounds-and-concurrency.md) and
  [TC-005](../test-cases/TC-005-dependency-edges.md) verify the component and
  architecture.
