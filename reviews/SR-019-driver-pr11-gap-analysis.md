---
id: SR-019
title: "Gap analysis of quire-driver PR #11: AD-001 claims against the driver's code"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@85ecacb5e1159f265a9249d64c131e4ce4d76e95; spec/assurance/AD-001-driver-architecture.md; src/drive.rs; src/handoff.rs; src/refusal.rs; src/registry.rs; src/lib.rs; tests/dependency_edges.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/FR-001, type: references }
---
# Gap analysis of quire-driver PR #11: AD-001 claims against the driver's code

## Summary

Ticket: IR-324. PR: quire-driver#11, reviewed at 85ecacb (base origin/main c0b2f31). The PR is
spec-only and changes no code, so this gap analysis checks each code claim the AD makes against
origin/main. It also checks those claims against the catch-up PR quire-driver#12 (IR-479, head
3d72fc6). Plan completion: not assessed.

Re-verified true at origin/main:
- src/drive.rs:14 imports `AttestationContext` and `KaniToolPins`, and `KaniToolPins::pinned()`
  is called at src/drive.rs:205. Neither name exists in CG main `src/`.
- src/handoff.rs:63, :78 and :93 call `insert_artifact_bytes` and `insert_artifact_digest`.
- The quoted doc is at src/handoff.rs:53-59: "informational constants, never checked" and the
  vacuous `/diagnostics/catalog` check.
- IR main `evidence.rs` holds domain packages, dependency packages and supported features. Its
  doc says the lock's raw-artifact digests are not checked against any evidence.
- E5 runs with `CheckedPackageReadLimits::bounded()` (src/handoff.rs:27). `HandoffLimit` is a
  limit refusal (src/refusal.rs:37, :196). The `package_id` equality is at src/handoff.rs:38-44.
- The other cited identifiers match the code: `VocabularyMismatch` (src/drive.rs:259),
  `invalid_capability/unknown-backend` (src/refusal.rs:156), `UnknownNode` as
  `runtime_invariant` (src/refusal.rs:162), the empty-`DependencyInput` refusal (FR-001:21-24)
  and the crate doc's step order (src/lib.rs).
- TC-005 asserts that the driver's model is CG's IR's model (tests/dependency_edges.rs:163-178),
  and #12 adds the one-copy assertion. D-7 at this head states both correctly.
- No code implements execute, replay or a terminal record. The AD's "not implemented" is true.

Since 218f51e, the pre-catch-up bullets are scoped "Before IR-479", so they no longer go stale
silently when #12 merges.

## Verdict

**CHANGES REQUESTED (medium).** The AD's invariant D-8 and its "single digest" sentence would be
false after #12, because #12 keeps two pin- and digest-shaped uses. The failure view and the
after-catch-up evidence sentence are incomplete or off.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | D-8 says "The driver uses no pin, attestation or digest-evidence API". After #12 the driver still calls `DefinitionLock::pinned()` (src/handoff.rs:76 on main; #12 keeps it for the root feature), and src/registry.rs:5/:19/:47 still carries `ManifestDigest` and `ToolIdentity` into `qsl_route` (#12 does not touch it). So D-8 fails even once IR-479 is done. Also, the current-state list omits the envelope's `manifest_digest`, `pinned_tool` and `digest_wire` (src/drive.rs:11, :268, :270, :364, :371), which #12 deletes and IR-479 names. Scope D-8 to what IR-479 removes, or route the rest. | spec/assurance/AD-001-driver-architecture.md:131 | wrong-requirement |
| FND-002 | medium | "That is the single canonical content-identity digest on this seam" overclaims. The FR-056 domain package documents cross E5 keyed by a `sha256-jcs` digest that IR recomputes (src/handoff.rs:72-74; IR evidence.rs doc), and the registry carries a `ManifestDigest` per provider manifest (src/registry.rs:47). Both survive #12. True statement: `package_id` is the one digest the driver itself checks. | spec/assurance/AD-001-driver-architecture.md:71-73 | wrong-requirement |
| FND-003 | low | The failure view says it states "who reports each failure", but it omits four driver faults: `SettlementCount`, `SupportedOutsideCandidates`, `RoutingTarget` and `VocabularyMismatch` (src/refusal.rs:52-81). `VocabularyMismatch` is named in prose at AD:64 but has no row. | spec/assurance/AD-001-driver-architecture.md:95-105 | missing-requirement |
| FND-004 | low | "After the catch-up the evidence is the unit's domain packages, supported features and any dependency packages." In #12 the evidence is the caller's domain package documents and QSL's root feature only. The driver supplies no dependency packages, because it compiles with an empty `DependencyInput` (AD:153). | spec/assurance/AD-001-driver-architecture.md:148-149 | wrong-requirement |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | bb4bf6e |
| FND-002 | fixed | bb4bf6e |
| FND-003 | fixed | bb4bf6e |
| FND-004 | fixed | bb4bf6e |
