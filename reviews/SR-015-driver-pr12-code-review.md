---
id: SR-015
title: "Code review (with Rust lane) of quire-driver PR #12"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@3d72fc6b148db691bc4d45b2e9f39ddffad90abc; Cargo.toml; Cargo.lock; src/drive.rs; src/handoff.rs; src/refusal.rs; src/registry.rs; tests/drive.rs; tests/dependency_edges.rs; deny.toml; README.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
---
# Code review (with Rust lane) of quire-driver PR #12

## Summary

Ticket: IR-479 (folds IR-478). PR: quire-driver#12, reviewed at 3d72fc6.

The code change is sound. Every removed CG/IR pin, attestation and
digest-evidence API use is gone from `src/` and `tests/` (grep for
`KaniToolPins`, `AttestationContext`, `IR_CANDIDATE_REVISION`,
`CheckedArtifactLocator`, `DigestDomain`, `insert_artifact_*`,
`diagnostics_catalog`, `UnpinnedBackend`, `InvalidAttestationContext`,
`pinned_tool`, `digest_wire`, CG `manifest_digest`: zero hits). No test was
removed and none is `#[ignore]`d (22 test fns at base, 23 at head, the new
one is `tc_008_an_unknown_header_profile_refuses_with_its_cause`). The two
changed harness assertions keep the same bounds, only CG's new
`integer("a")?` spelling.

Measured:

- `make ci` on 3d72fc6: green. fmt-check, clippy -D warnings, tests
  2 + 1 + 20 = 23 passed, 0 failed, 0 ignored; deny licenses+sources ok;
  audit-unsafe ok; docs ok; `quire validate` exit 0.
- `cargo tree -d`: one copy each of `quire-contract-model`, `qsl-package`,
  `quire-contract-ir`, every `qsl-*` and CG; no `quire-spec-language` root
  crate. Duplicates are third-party only (`jsonschema`, `syn`, ...).
- Spelling claim reproduced in a scratch copy: with `rev = <the same lock
  commits>` cargo resolves two copies of `quire-contract-model` and of every
  `qsl-*` (`?rev=` and `?branch=main` sources) and the build fails E0308
  "multiple different versions of crate `quire_contract_model`" at
  src/drive.rs:195 and src/upstream.rs:124. A `[patch]` to the same git URL is
  refused by cargo ("patches must point to different sources"). So
  `branch = "main"` plus Cargo.lock is the minimal sound fix; the lock holds
  the commits.
- Mutants: `compile_cause`'s Profile arm to `None` fails the new tc_008
  (`unknown_profile` vs `unknown_profile/unsupported-selection`); dropping
  `support_feature(root)` from E5 evidence fails 5+ drive tests; replacing the
  `unit()` header digest with a constant fails 6+ drive tests, so QSL checks
  it and reading it from `DefinitionLock::pinned()` (in-process, hermetic) is
  required, not a copied digest.
- `ProviderManifest.manifest_digest` / `qsl_route::ManifestDigest` stay
  because QSL's `Candidate::new(id, manifest_digest)` still requires it and
  QSL's registry keys conflicts by it (TC-004). That is adapting to QSL's API,
  not the driver tracking a digest.

## Verdict

**CHANGES REQUESTED (medium, comments only).** Code is correct; the
remaining findings are stale pin wording in files this PR touched, which
the repo's own CLAUDE.md says to remove when touched.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Stale pin wording in touched test files. tests/drive.rs:588 says CG is called "with ... the pinned tools", but no tools are passed any more. tests/dependency_edges.rs:181-186 cites "Cargo.toml's revision note" (deleted by this PR) and "QSL-255b PR 2 (this bump)", and :183,194,203 call the deps "pins" although they are now `branch = "main"`. CLAUDE.md: "When you touch code or specs that contain one, remove it". Failure scenario: a reader looks for a revision note or pinned-tool argument that does not exist. | tests/drive.rs:588; tests/dependency_edges.rs:181-204 |
| FND-002 | low | Pin wording outside the diff: deny.toml:12,14 "reached through the QSL and IR pins"; src/registry.rs:20 "The pinned tool identity" (QSL `ToolIdentity`). deny also reports `AGPL-3.0-only`, `ISC`, `CDLA-Permissive-2.0`, `BSD-3-Clause` allowances as not encountered. No behavior impact. | deny.toml:12-19; src/registry.rs:20 |
| FND-003 | low | The new whole-graph one-copy assertion subsumes the older qsl-package model-identity check above it; the earlier block is now redundant. | tests/dependency_edges.rs:186-213 |
| FND-004 | low | Style: doc line exceeds the repo's 100-char width (rustfmt does not wrap comments); README reflow left a long line. | src/refusal.rs:260; README.md:30 |
| FND-005 | low | The new one-copy assertion contradicts draft PR #11's AD-001 decision row (line 164), which decides the dependency-edge test should not assert model-copy equality. Whichever merges second must reconcile; owner decision. | tests/dependency_edges.rs:207-213 |
