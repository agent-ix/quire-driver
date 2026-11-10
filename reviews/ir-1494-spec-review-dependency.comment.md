<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14857 method=spec-review/dependency lang=spec pr=quire-driver#146 model=claude-opus-5-5 run=87752af5-6144-4b92-9817-ee41c4ca9989 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | exceeds | AC-11 tag->branch switch belongs in a sibling dependency ticket; as written it covers one of two manifests and reverses IR-672 |
| FND-002 | low | other | Slice (b) prerequisite has no live ticket (IR-304 is Done) |

Verdict: Needs changes. Prerequisites QSL-266/700/904 are Done and their shapes exist on QSL origin/main and at the driver's locked qsl-route commit; CG FR-022 is merged; CG PR #772 is open. Two dependency defects: the CG source-spelling change is out of FR-020's scope and under-specified, and the slice (b) prerequisite has no live ticket.

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: 'One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item''s `RoutedRequirement`, and the package''s `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), "Per-occurrence requirement ...'
- id: FR-020-AC-11
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: The driver's `Cargo.toml` entry for `quire-contract-codegen` names a branch like the `qsl-route` entry and carries no `rev`, and `Cargo.lock` content is the only record of the resolved commit.
- id: FR-020-AC-10
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard.
- id: FR-020#behavior-dependency
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- The driver''s dependency on CG SHALL name CG by branch, as it names `qsl-route`, with no `rev`, commit hash or digest. Taking the IR-304 types is a plain dependency update of that branch reference, not a carve-out.'
- id: Cargo.toml
  path: Cargo.toml
  role: context_only
  excerpt: quire-contract-codegen = { git = "https://github.com/agent-ix/quire-contract-codegen", tag = "v0.1.0" } -- with the comment 'The QSL layer crates and CG are tracked on `main` of their repositories ... The exact commits are in Cargo.lock.'
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  method: spec-review/dependency
  check: exceeds
  artifact_id: FR-020-AC-11
  related_ids:
  - FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: Behavior last bullet; AC-11
  excerpt: The driver's `Cargo.toml` entry for `quire-contract-codegen` names a branch like the `qsl-route` entry and carries no `rev`, and `Cargo.lock` content is the only record of the resolved commit.
  finding: 'The tag->branch switch is a repo-wide dependency-policy change, not projection behaviour, and it reverses IR-672 (#143, merged today), which moved BOTH Cargo.toml and quire-cli/Cargo.toml (dev-dependency) from branch=main to tag=v0.1.0 while leaving the root comment saying CG is tracked on main. AC-11 names only ''the driver''s Cargo.toml''; switching the root alone makes the workspace resolve two CG git sources (two incompatible copies of the IR-304 types in quire-cli tests). ''Cargo.lock content is the only record of the resolved commit'' is a repo-wide negative that is not inspectable as written, and ''not a carve-out'' is policy prose in a Behavior bullet. The switch IS a hard prerequisite (v0.1.0 = a15be8eb predates PR #772), so carry it in a sibling ticket owning both manifests and the comment (FR-001/TC-005 own dependency edges), list it in FR-020 Dependencies, and drop AC-11 and the bullet from FR-020.'
- fnd: FND-002
  severity: low
  confidence: medium
  method: spec-review/dependency
  check: other
  artifact_id: FR-020-AC-10
  related_ids:
  - FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: Dependencies; AC-10 Verification
  excerpt: In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard.
  finding: 'FR-020 cites ''IR-304 slice (b)'' as a prerequisite, but IR-304 is Done in Linear while slice (a) PR #772 is still open and no separate slice (b) ticket is named; CG FR-022 also says the driver projection ''has no ticket yet''. Name the ticket that owns slice (b) (or file it) so AC-10''s guard-Assume branch has a trackable prerequisite.'
bindings: []
```

+++
