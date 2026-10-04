# IR-528 G5b gate evidence plan

## Purpose and boundary

This plan defines the evidence the IR-528 gate will collect. The cross-repo
tests belong in `agent-ix/quire-integration`; the driver repository owns the
CLI/library contract and the gate decision. This is a plan only: the current
`quire-integration` checkout describes itself as a scaffold and does not yet
claim the driver CLI, plugin host, cache, AOT runner or G5b gate are
implemented.

Run the gate only after the IR-527 specification and the implementation
tickets it blocks have merged, and after QSL LC7 removes the old binary. Use
one exact dependency lock for the composed run. Preserve all per-case stdout,
stderr, exit status, typed record and test logs as the gate evidence bundle.

## Evidence groups

| Group | Evidence and procedure | Pass condition | Owning test location |
|---|---|---|---|
| CLI and library parity | For one request per O-16 outcome kind, call the library entrypoint and run the `quire` binary with the same typed inputs. Capture canonical outcome bytes and process status for both paths. Include success, violation, undefined, refusal, unsupported, incomplete, inconclusive and internal failure. Exercise both proof `inconclusive` (exit 22) and supplied-trace pending `inconclusive` (exit 0). | Each pair emits byte-equal `quire-outcome/1` documents and the same QSL `Category::exit_code`. Expected codes: success 0; violation and non-proof undefined 10; refusal 20; unsupported 21; incomplete 22; proof/analyze inconclusive 22; pending trace inconclusive 0; internal failure 30. | `quire-integration` CLI/library composition tests; small CLI tests in `quire-driver` may provide the process harness. |
| Provider and plugin install | Build a first-party `Provider` into the test binary. Separately launch a fixture plugin only through its caller-named path, exchange canonical hello/request/result frames, and verify that its manifest reaches the shared registry conversion. Exercise missing executable, malformed hello and failed process outcomes. | Linked provider and installed plugin use the same manifest admission and routing contract. A plugin is not discovered from an ambient search path, loaded as a dynamic library or trusted to mark a counterexample refuted without replay. | `quire-integration` provider/plugin host tests; provider manifest fixtures are authored there. |
| Cache identity | Execute the same eligible prove request twice with unchanged request and manifest bytes; then change one manifest byte without changing any recorded revision pin. Mutate each other meaning input independently. Corrupt an entry's name, preimage and typed result. Attempt to store plugin, internal-failure, cancelled, deadline, time-limited incomplete and wall-time-bearing outcomes. | Unchanged meaning inputs hit; changed manifest content and every changed meaning input miss even with the same pin. Corrupt entries miss. Prohibited outcomes and plugin results are never stored. Hit/miss records identify the tested canonical input set without exposing a second cache-key policy. | `quire-integration` cache composition tests; key construction unit tests remain in `quire-driver`. |
| AOT parity | Run one checked-package corpus through the interpreter and AOT with identical entries, environments, arguments and accounting limits. Include completed, undefined, refused and incomplete results, source locations, loss records, a boundary limit and one unsupported entry. Build with the toolchain named in the request. | AOT and interpreter return equal typed evaluations and causes for every supported entry. Unsupported and preparation-limit results remain typed; neither silently selects another engine. The test also records that generated code read only the package from the current checked request. | `quire-integration` AOT/interpreter conformance tests, across the public driver-to-QSL API. |
| Prove, Kani, replay and terminal records | Use the real Kani runner on one generated harness that proves and one that falsifies. Feed the falsifying result through QSL replay. Include a replay mismatch and a second requested item so the test checks item-local records. Capture generated artifact identity, Kani output, replay output and the final `quire-outcome/1` document. | The Kani run is real (not a fabricated provider response). A reproduced counterexample settles `refuted`; a mismatch settles `inconclusive`; CG's FR-331 mapping is the only terminal mapping. The output contains exactly one FR-331 record per requested item and the binary and library paths serialize it identically. | `quire-integration` Kani/replay composition tests; actual Kani execution remains an explicit long gate. |

## Gate assembly

1. Verify that every evidence group ran against the same locked QSL, IR, CG,
   RT and `quire-driver` revisions.
2. Keep unit and fast composition tests in the normal local test gate. Run the
   real Kani prove/replay cases as an explicitly identified slow phase; a
   timeout is a gate result to classify, not a passing or skipped case.
3. Attach case-level outcomes and raw command exit statuses. Do not summarize
   a failed or timed-out case as a green aggregate.
4. Gate G5b passes only when every required group has evidence, the parity
   and cache checks pass, and every requested item has its expected FR-331
   terminal record.

## Dependencies and current gaps

- IR-527 defines the CLI, exit, provider, cache, AOT, proof/replay and
  abstraction-relation requirements that this plan covers.
- IR-607 through IR-610 provide the binary, options, provider/cache and
  execution implementations.
- QSL LC7 removes the old `quire-spec` binary so the single `quire` CLI can
  own these verbs.
- QSL AR2 is a prerequisite for the SP-D9 bound-item check; QSL AR3 and
  QSL-514 own its cross-repo acceptance test.
- Current `quire-integration` code and tests are a scaffold. Every test named
  above is planned, not claimed to exist or pass.
