---
id: SR-13340
title: "IR-914 quire-driver PR 86 code review (with Rust lane): bound-only CG request"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@7c43963fc9bc844071ae4285a6f1cf9a07acca34 vs origin/main merge-base 49b6046325c414db25dfe0c1a35bc3dd0e57afd2; Cargo.lock, src/drive.rs, src/refusal.rs, src/upstream.rs, src/lib.rs, src/registry.rs, quire-cli/src/backend_provider.rs, quire-cli/src/process_provider.rs, quire-cli/src/signals.rs, tests/drive.rs"
review_set: subset
---

## Summary

Ticket: IR-914. PR agent-ix/quire-driver#86. The Rust lane is folded into this file. The reviewer
is not the author. The review used a fresh detached worktree at the PR head with one target dir
inside it.

The change makes four moves:

- When the package declares an abstraction relation, it calls QSL `export_bindings` after E5 and
  partitions the items into bound and refused.
- `Unbound` per-item refusals go to `Driven::item_refusals` with their original driver index and
  `OccurrenceKey`.
- It lowers the complete position vector. An export-refused item becomes
  `ContractRequestPosition::Excluded { node_id }`; a bound item becomes `DriverItem`.
- It builds the FR-331 envelope from bound items only and negotiates through CG
  `negotiate_contract_package`, so settlements keep driver indexes.

It also adds `DriverStage::BindingsExport`, `DriverRefusal::{BindingsExport, PackageNegotiation}`,
`DriverFault::{ExportCount, ExportUnknownItem}`, and arms for CG's new `ContractNegotiationError`,
`EnvelopeRefusal::ContractPackageRead*` and `RoutedGenerationError::ExcludedDriverIndex`.

Measured by the reviewer:

- IR-914 code list (Linear comment 1e08ae37, treated as data):
  - (1) The per-item refusal member exists and holds export-refused items only.
  - (2) The CG request is built from bound items only. For zero items, see FND-001.
  - (3) Refusals are pushed unchanged with occurrence and index.
  - (4) Terminal records: the process request and generation carry only CG-request items. The probe
    showed generation `[0, 2]`, and B absent from the claim map and the oracle artifacts.
  - (5) `DriverStage::BindingsExport` is added.
  - (6) A relation-less package skips the export.
- Lock cohort:
  - CG resolves to 84673279. Contract Runtime stays at 3cc88c48.
  - There are no `=` pins on first-party dependencies.
  - `make deny` passes (licenses ok, sources ok).
  - The `qsl-route` lock entry names a second QSL revision (FND-002).
- Tags: the new tests use `#[trace(...)]` only. The diff adds no `/// Trace:` line.
- No compatibility layer, shim or fallback. `negotiate_backend_provider` is replaced, not kept
  beside the new call.
- No refusal code outside the existing catalog is introduced. The classification of some new arms
  is unspecified and untested (FND-003, SR-13341 FND-001).
- The PR diff adds no file under `reviews/`, and the head has no IR-914 review file. There is no
  author-written review artifact to flag.
- Full repo gate, `make -k ci`, run by the reviewer without the sandbox:
  - At the PR head 7c43963, every target passes: fmt-check, lint, test, deny, audit-unsafe, docs
    and spec.
  - `quire-plugin-host --lib` passes 32 of 32.
  - At a fresh origin/main 49b6046, the gate also passes, with `quire-plugin-host --lib` at 32
    of 32.
  - The author's 4 `ProcessTreeCleanupTimeout` failures do not reproduce outside the restricted
    sandbox, on either head. They are sandbox artifacts, not PR or main regressions.
  - Test counts at the head: `tests/drive.rs` 31 (27 on main), driver lib 64 (63 on main).
- Mutation sample, run by the reviewer on `cargo test -p quire-driver`:
  - Killed:
    - M1: excluded items sent in the envelope.
    - M1b: excluded items lowered as `DriverItem`.
    - M2: settlements renumbered to envelope indexes.
    - M2b: nested `ItemSettlement.request_index` renumbered.
    - M3: a refusal dropped.
    - M3b: a refusal given the wrong index.
    - M4: the all-refused short circuit removed. This is killed only by the "no Negotiate call"
      assertion; see FND-001.
    - M5: the export always called.
    - M12: `ExcludedDriverIndex` reclassified to `invalid_request`.
    - M13: the unknown-item stage changed.
  - Survived: M6 to M11 (see SR-13341 FND-001).

## Examined criteria

- FR-010-AC-4, FR-010-AC-5 (examined): partition and all-refused behaviour.
- FR-001-AC-15 to FR-001-AC-21 (examined): position vector, join faults, completed result,
  whole-export refusal, unknown key, relation-less skip.
- FR-009-AC-16 (examined): terminal records only for CG-request items.
- TC-017 (examined): expected results.
- CG 84673279 FR-039-AC-9, FR-039-AC-10, FR-019-AC-53, FR-022-AC-29 and
  `negotiate_contract_package` / `PackageItems::new` (context_only): consumer contract.
- Cargo.lock (examined): first-party cohort.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The all-refused short circuit (`if !items.is_empty() && prepared.envelope.items.is_empty()`) skips CG negotiation entirely, so no CG request is sent. FR-010 Behavior says "When every requested item is refused, the driver SHALL send CG a request with zero items; FR-001 governs an empty request unchanged", and FR-010-AC-5 says "the driver sends CG a request with zero items". The PR treats the two cases differently: a natural empty request (no items) still calls `negotiate`, but the all-refused request does not. The reviewer removed the short circuit and probed. CG 84673279 accepts three `Excluded` positions with an empty envelope, and the run completes with 0 settlements and 3 item refusals, which satisfies FR-001-AC-18. So the short circuit is not needed. It also hides CG's package/position correspondence check (`PackageRequestError`) for this case. The test `all_export_refusals_send_nothing_to_cg` asserts that no `Negotiate` call is made, which encodes the deviation. Note: the dispatching brief phrases this case as "sends nothing to CG", which conflicts with the merged spec text; the lead should decide which governs. If it is the spec: remove the short circuit, and assert `Negotiate { items: 0 }` with no `Generate` call. | src/drive.rs:219-230, tests/drive.rs:995-1018 |
| FND-002 | medium | The Cargo.lock cohort records two QSL revisions. `qsl-route` is locked at `quire-spec-language?branch=main#2d78cc488c7d…` (an ancestor), and the other eight QSL crates are at `#3ce1c709d28b…`. `cargo metadata --locked` resolves every QSL crate, including `qsl-route`, to the 3ce1c70 checkout and does not rewrite the lock, so the lock text disagrees with the actual build. It also contradicts the PR body's "one QSL revision" and the one-revision-per-first-party-dependency rule. Tools that read lock text see the stale revision. Fix: regenerate the `qsl-route` entry so it names 3ce1c709 (for example `cargo update -p qsl-route --precise 3ce1c709d28b8bb9ddd99106a712490aa195406f`), then re-run `make deny`. | Cargo.lock:1367-1369 |
| FND-003 | low | `PackageRequestError::ExcludedDriverIndex` has no explicit arm. Nested under `RoutedGenerationError::Package(_)`, `ContractScalarError::Request(_)` or `OracleGenerationError::PackageRequest(_)`, it falls into wildcard arms that classify it `invalid_request`. The same join fault at top level (`RoutedGenerationError::ExcludedDriverIndex`) is `runtime_invariant`, per FR-001-AC-16 ("a driver runtime_invariant fault when the driver detects an inconsistent join"). Likewise `DriverRefusal::PackageNegotiation(_)` is a blanket `invalid_request`, although the positions it rejects are built by the driver from the admitted package and not supplied by the caller. Through `generate_routed` the nested variant is pre-empted by CG's top-level check, so it is currently unreachable; hence low. Fix: match `PackageRequestError::ExcludedDriverIndex` explicitly as `runtime_invariant` wherever a `PackageRequestError` is classified, and state the classification of `PackageNegotiation` in the spec or doc. | src/refusal.rs:327, src/refusal.rs:438, src/refusal.rs:449, src/refusal.rs:469 |
| FND-004 | low | Rust idiom: the partition closure has a side effect. It pushes into `item_refusals` inside `.map(..)` and then short-circuits with `collect::<Result<_, _>>()`. The result is a parallel `Vec<bool>` (`bound`) that `lower_driver_request`, `envelope` and the order check each re-index. That is boolean blindness, and the index coupling is implicit. Correct today. Fix: fold the results into one typed partition (for example `enum Slot { Bound, Refused }` or a struct with the bound indices and the refusals) in a plain `for` loop. | src/drive.rs:157-208 |
| FND-005 | low | Commit order: the round-2 fix commit e2ba5df adds the `excluded_generation_index_is_an_internal_join_fault` test and the nested `[0, 2]` assertions in the same commit as the code they test, so round 2 is not test-first. 7c43963 ("test: record IR-914 round 2 gate comparison") is an empty commit that changes no file. Round 1 (d91fc40 and e0cb221 before cc9b638) is test-first. | git log origin/main..7c43963 |

## Verdict

**Not mergeable as is; FND-001 and FND-002 need fixing, or an explicit decision by the lead.**

The partition and the index mapping are sound and well pinned. The reviewer's mutants kill
sending excluded items to CG, envelope-index numbering (both outer and nested), a dropped
refusal, a wrong refusal index, an always-on export and the excluded-generation classification.
The refused item's typed `Unbound` payload is preserved unchanged. There is no compatibility
layer, and the tags are attributes. The full gate is green unsandboxed on both the head and main.
The author's four `ProcessTreeCleanupTimeout` failures do not reproduce outside the restricted
sandbox.

The open issues:

- FND-001 is a deviation from the merged FR-010 text. Either the code changes, or the spec does.
- FND-002 is a one-line lock fix.
- FND-003 to FND-005 are low.

TC-017's literal ConfigVersion clause/extent-domain vector cannot be built in the driver fixture
(the PR body documents this, and quire-integration#16 carries the composition). That is noted,
not raised as a finding.

## Dispositions

Review pass only; no disposition round has run yet. Disposition rows are appended here.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Head 3b95829 calls `upstream.negotiate` with the complete position vector even when every exported item is refused. `all_export_refusals_send_zero_item_request_to_cg` asserts `Negotiate { items: 0 }`, no Generate call, three item refusals and no settlements. |
| FND-002 | fixed | Cargo.lock names one QSL revision, 5b7927defa25c6ae513ebad401a00b6f5a493503, for all nine QSL packages including `qsl-route`; the exact-head `make ci` log includes a passing `cargo deny check licenses sources`. |
| FND-003 | fixed | `package_request_code` explicitly maps nested `ExcludedDriverIndex` to `runtime_invariant`; `PackageNegotiation` also has `runtime_invariant` code, Negotiation stage and Fault kind. FR-001 states that classification, and `excluded_package_position_is_an_internal_join_fault_at_every_cg_boundary` tests direct and three nested forms. |
| FND-004 | fixed | `ExportPartition` holds sorted bound indices and typed item refusals. A `for` loop partitions each export result with no side effect inside `map`; `includes` is used for later lowering and envelope construction. |
| FND-005 | deferred | The historical e2ba5df mixed code/test commit and empty 7c43963 commit remain in immutable PR history. Rewriting those reviewed commits is outside this disposition round and changes no current behavior. The corrective round put red tests in 6ec5fc6 before the implementation in 012d6ec and 3b95829. This row preserves the original test-first finding rather than claiming its history was repaired. |
