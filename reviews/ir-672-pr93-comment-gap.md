<!-- reviewer repo=agent-ix/quire-driver visibility=private quoin=0.28.3 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14791 method=gap-analysis lang=rust pr=quire-driver#93 model=gpt-6.1-sol run=7F689A1D-89E7-4335-AA30-8BD9709231A1 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | coverage | FR-006-AC-26 has no trace-tagged test, and production proceeds when duplicate sessions have identical idempotent admissions, then dispatches to both. |
| FND-002 | high | coverage | FR-006-AC-4 is untagged and the production command path does not activate signal handling, so a handled signal can bypass session cleanup. |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-006-AC-1, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "When negotiation selects a process provider, the driver invokes its process-provider seam with the configuration entry associated with that provider's once-admitted hello identity, one `EmittedPackage` value, that entry's caller-supplied deadline, the registered-candidate manifest snapshot and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry."}
  - {id: FR-006-AC-4, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal."}
  - {id: FR-006-AC-8, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged."}
  - {id: FR-006-AC-15, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "A validated result without a specified QSL terminal mapping remains typed and unmapped in the driver's shared settlement path."}
  - {id: FR-006-AC-24, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `--provider-config` file accepts only the documented strict JSON shape; missing/unreadable files, malformed JSON, missing or wrongly typed fields, a `deadline_ms` that is missing, non-integer, outside 1..=9,007,199,254,740,991, `null` or wrongly typed optional process budgets, and unknown fields at any object level are rejected before any provider starts."}
  - {id: FR-006-AC-25, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `quire-cli` composition starts and admits every configured provider before negotiation, and uses each admitted hello descriptor as the sole provider identity for subsequent selection and dispatch."}
  - {id: FR-006-AC-26, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a QSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when QSL treats repeated identical admission as idempotent."}
  - {id: FR-006-AC-27, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "If a configured provider fails to start or admit its hello, the composition begins no negotiation, closes all earlier active sessions, and preserves the original typed host failure."}
  - {id: TC-013, path: spec/test-cases/TC-013-plugin-host-protocol-and-failures.md, role: context_only, excerpt: "Verify that the driver passes negotiated process-provider work through its process-provider seam and returns typed provider results through its proof path; TC-028 verifies that `quire-cli` composes this seam with `quire-plugin-host`."}
  - {id: TC-028, path: spec/test-cases/TC-028-cli-package-boundary.md, role: context_only, excerpt: "Exercise the production `quire run --provider-config <path>` command with a strict JSON file containing at least two entries for distinct fixture executables."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-006-AC-26
    related_ids: [TC-028]
    path: spec/functional/FR-006-host-provider-plugins.md
    lines: "220"
    excerpt: "If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a QSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when QSL treats repeated identical admission as idempotent."
    finding: "The computed matrix has no test binder for AC-26. The implementation also violates the criterion: identical repeated admissions are idempotent in the registry, then the production dispatch loop sends matching work to both sessions."
  - fnd: FND-002
    severity: high
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-006-AC-4
    related_ids: [TC-028]
    path: spec/functional/FR-006-host-provider-plugins.md
    lines: "198"
    excerpt: "On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal."
    finding: "The computed matrix has no binder for AC-4, and the production command path passes an inert default `SignalContext` without calling `signals::run`. A signal during configured dispatch can therefore bypass cancellation and leave the isolated plugin process group running."
```

+++
