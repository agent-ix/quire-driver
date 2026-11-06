---
id: SR-2026110414560059
title: IR-1374 PR127 code-review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; IR-1374; spec/functional/FR-001-orchestrate-the-driver-steps.md, spec/test-cases/TC-001-steps-run-in-order.md, src/drive.rs, src/handoff.rs, src/upstream.rs, tests/drive.rs"
review_set: subset
---

## Summary

Independent review of frozen PR #127 head 53cacfe. No applicable AssuranceProfile exists in this repository's spec tree. The Rust call chain carries the caller package limit into E4 and its emit member into E5. The focused test uses real QSL and IR calls with an exact byte ceiling and a one-byte-lower refusal. No wrapper, copied API, default limit, new pin, or weakened gate appears in the diff.

## Verdict

PASS: no findings in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The Rust call chain carries the caller package limit into E4 and its emit member into E5. The focused test uses real QSL and IR calls with an exact byte ceiling and a one-byte-lower refusal. No wrapper, copied API, default limit, new pin, or weakened gate appears in the diff.

## Examined units

```yaml
{
  "scope": [
    {
      "id": "FR-001-AC-25",
      "path": "spec/functional/FR-001-orchestrate-the-driver-steps.md",
      "role": "examined",
      "excerpt": "The driver passes the caller's `SpineLimits.package.emit()` to QSL admission evidence at E5, preserving the exact `emit.output_bytes` value used for E4, including a caller-selected value."
    },
    {
      "id": "TC-001",
      "path": "spec/test-cases/TC-001-steps-run-in-order.md",
      "role": "examined",
      "excerpt": "Drive a fixture with a caller-selected `emit.output_bytes` ceiling equal to its actual canonical E4 byte length and confirm E5 admits it. Lower the ceiling by one byte and confirm QSL refuses at E4 before E5. Inspect the E5 call to `admission_evidence` to confirm it receives that same caller `SpineLimits.package.emit()` value."
    },
    {
      "id": "src/drive.rs",
      "path": "src/drive.rs",
      "role": "examined",
      "excerpt": "input.limits.package.emit(),"
    },
    {
      "id": "src/handoff.rs",
      "path": "src/handoff.rs",
      "role": "examined",
      "excerpt": "admission_evidence(models, checked, emitted, imports, emit)"
    },
    {
      "id": "src/upstream.rs",
      "path": "src/upstream.rs",
      "role": "examined",
      "excerpt": "package(&checked, limits.package, cancel)"
    },
    {
      "id": "tests/drive.rs",
      "path": "tests/drive.rs",
      "role": "examined",
      "excerpt": "tc_001_caller_emit_ceiling_bounds_e4_and_admits_at_e5"
    }
  ]
}
```
