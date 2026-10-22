<!-- reviewer repo=agent-ix/quire-driver visibility=private quoin=0.28.3 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14792 method=spec-review/base lang=spec pr=quire-driver#93 model=gpt-6.1-sol run=7F689A1D-89E7-4335-AA30-8BD9709231A1 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | test-intent | FR-006 and its test cases still describe production dispatch/config coverage as planned under IR-672 although the changed matrix reports the production tests. |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-006-AC-1, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "When negotiation selects a process provider, the driver invokes its process-provider seam with the configuration entry associated with that provider's once-admitted hello identity, one `EmittedPackage` value, that entry's caller-supplied deadline, the registered-candidate manifest snapshot and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry."}
  - {id: FR-006-AC-8, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged."}
  - {id: FR-006-AC-24, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `--provider-config` file accepts only the documented strict JSON shape; missing/unreadable files, malformed JSON, missing or wrongly typed fields, a `deadline_ms` that is missing, non-integer, outside 1..=9,007,199,254,740,991, `null` or wrongly typed optional process budgets, and unknown fields at any object level are rejected before any provider starts."}
  - {id: FR-006-AC-8, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged."}
  - {id: FR-006-AC-24, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `--provider-config` file accepts only the documented strict JSON shape; missing/unreadable files, malformed JSON, missing or wrongly typed fields, a `deadline_ms` that is missing, non-integer, outside 1..=9,007,199,254,740,991, `null` or wrongly typed optional process budgets, and unknown fields at any object level are rejected before any provider starts."}
  - {id: TC-013, path: spec/test-cases/TC-013-plugin-host-protocol-and-failures.md, role: examined, excerpt: "This step checks field ownership and serialization only; it does not imply that the driver dispatches negotiated work through the Process seam, which remains IR-672 implementation work."}
  - {id: TC-028, path: spec/test-cases/TC-028-cli-package-boundary.md, role: examined, excerpt: "This end-to-end routed case remains planned until IR-672 implements production dispatch."}
  - {id: spec/tests.md, path: spec/tests.md, role: examined, excerpt: "Partial: `quire-cli` tests cover explicit-option parsing, the empty configuration, strict invalid-file/deadline outcomes, and a production two-provider run that admits both providers before negotiation, dispatches only the selected request, preserves its typed result and captures both sessions' stderr."}
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: spec-review/base
    check: test-intent
    artifact_id: TC-013
    related_ids: [FR-006-AC-8, FR-006-AC-24, TC-028]
    path: spec/test-cases/TC-013-plugin-host-protocol-and-failures.md
    lines: "36"
    excerpt: "This step checks field ownership and serialization only; it does not imply that the driver dispatches negotiated work through the Process seam, which remains IR-672 implementation work."
    finding: "The updated `spec/tests.md` reports the production two-provider dispatch and strict configuration/deadline tests, while FR-006-AC-1/8/24 verification cells and TC-013/TC-028 still say those behaviors remain planned under IR-672. This contradicts the current coverage index and can cause maintainers to treat implemented behavior as unimplemented. Update the stale statuses and test-case statements while keeping settlement and untested lifecycle cases marked pending."
```

+++
