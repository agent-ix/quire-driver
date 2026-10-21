<!-- reviewer repo=agent-ix/quire-driver visibility=private quoin=0.28.3 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14790 method=code-review lang=rust pr=quire-driver#93 model=gpt-6.1-sol run=7F689A1D-89E7-4335-AA30-8BD9709231A1 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | code-bug | Identical admitted identities are idempotently registered, after which every matching session receives the selected request instead of failing closed. |
| FND-002 | high | code-bug | The production entrypoint never starts the signal listener or registers active sessions, so SIGINT/SIGTERM can terminate the CLI while leaving the isolated plugin process group running. |

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-006-AC-1, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "When negotiation selects a process provider, the driver invokes its process-provider seam with the configuration entry associated with that provider's once-admitted hello identity, one `EmittedPackage` value, that entry's caller-supplied deadline, the registered-candidate manifest snapshot and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry."}
  - {id: FR-006-AC-4, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal."}
  - {id: FR-006-AC-8, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged."}
  - {id: FR-006-AC-15, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "A validated result without a specified QSL terminal mapping remains typed and unmapped in the driver's shared settlement path."}
  - {id: FR-006-AC-26, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a QSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when QSL treats repeated identical admission as idempotent."}
  - {id: FR-006-AC-27, path: spec/functional/FR-006-host-provider-plugins.md, role: examined, excerpt: "If a configured provider fails to start or admit its hello, the composition begins no negotiation, closes all earlier active sessions, and preserves the original typed host failure."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: code-review
    check: code-bug
    artifact_id: FR-006-AC-26
    related_ids: [TC-028]
    path: quire-cli/src/cli.rs
    lines: "881-887, 927-957"
    excerpt: "If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a QSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when QSL treats repeated identical admission as idempotent."
    finding: "The composition passes all active descriptors to `register_admitted` and only handles an error. The registry treats identical repeated descriptors as idempotent, so two sessions with the same admitted identity proceed to negotiation. The dispatch loop then matches both sessions by that identity and sends the selected request to both executables, violating the fail-closed requirement."
  - fnd: FND-002
    severity: high
    confidence: high
    method: code-review
    check: code-bug
    artifact_id: FR-006-AC-4
    related_ids: [FR-013-AC-14, FR-013-AC-15]
    path: quire-cli/src/cli.rs
    lines: "1131-1143, 1250-1267"
    excerpt: "On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal."
    finding: "The production entrypoint passes a default `SignalContext`; `dispatch` ignores it, and the configured run also receives a default context. Nothing invokes `signals::run` or activates session tracking. Since the host starts the plugin in an isolated process group, SIGINT/SIGTERM can terminate the CLI without bounded host cleanup and leave the plugin tree running."
```

+++
