# IR-528 G5b gate evidence plan

## Purpose and boundary

This plan defines the evidence the IR-528 gate will collect. The cross-repo
tests belong in `agent-ix/quire-integration`; the driver repository owns the
CLI/library contract and the gate decision. Existing integration coverage
includes `tests/integration.rs`, `tests/qsl_codegen_composition.rs`,
`tests/qsl_kani_exemplar.rs` and `tests/qsl_model_owner_admission.rs`. The new
G5b driver-binary coverage below is planned; it is not claimed to exist. Reuse
or extend the existing composition, owner-admission and Kani-exemplar tests
where their evidence matches the G5b criterion.

Run the gate only after the IR-527 specification and the implementation
tickets it blocks have merged, and after QSL LC7 removes the old binary. Use
the built integration executable and library with the actual admitted inputs
and generated artifacts under test. Preserve per-case stdout, stderr, exit
status, typed record and test logs as the gate evidence bundle.

## Evidence groups

| Group | Evidence and procedure | Pass condition | Owning test location |
|---|---|---|---|
| CLI and library parity | For one request per O-16 outcome kind, call the library entrypoint and run the `quire` binary with the same typed inputs. Capture canonical outcome bytes and process status for both paths. Include success, violation, undefined, refusal, unsupported, incomplete, inconclusive and internal failure. Exercise both proof `inconclusive` (exit 22) and supplied-trace pending `inconclusive` (exit 0). | Each pair emits byte-equal `quire-outcome/1` documents and the same QSL `Category::exit_code`. Expected codes: success 0; violation and non-proof undefined 10; refusal 20; unsupported 21; incomplete 22; proof/analyze inconclusive 22; pending trace inconclusive 0; internal failure 30. | `quire-integration` CLI/library composition tests; small CLI tests in `quire-driver` may provide the process harness. |
| Provider and plugin install | Build a first-party `Provider` into the test binary. Separately launch a fixture plugin only through its caller-named path, exchange canonical hello/request/result frames, and verify that its manifest reaches the shared registry conversion. Exercise missing executable, malformed hello and failed process outcomes. | Linked provider and installed plugin use the same manifest admission and routing contract. A plugin is not discovered from an ambient search path, loaded as a dynamic library or trusted to mark a counterexample refuted without replay. | `quire-integration` provider/plugin host tests; provider manifest fixtures are authored there. |
| Cache identity | Execute the same eligible prove request twice with unchanged request and admitted `ManifestDigest`; then supply a different admitted manifest whose content identity changes. Mutate each other meaning input independently. Corrupt an entry's name, preimage and typed result. Attempt to store plugin, internal-failure, cancelled, deadline, time-limited incomplete and wall-time-bearing outcomes. | Unchanged meaning inputs hit; a changed admitted manifest content identity and every changed meaning input miss. Corrupt entries miss. Prohibited outcomes and plugin results are never stored. Hit/miss records identify the tested canonical input set without exposing a second cache-key policy. | `quire-integration` cache composition tests; key construction tests belong to downstream `quire-cache`. |
| AOT parity | Run one checked-package corpus through the interpreter and AOT with identical entries, environments, arguments and accounting limits. Include completed, undefined, refused and incomplete results, source locations, loss records, a boundary limit and one unsupported entry. Build with the toolchain named in the request. | AOT and interpreter return equal typed evaluations and causes for every supported entry. Unsupported and preparation-limit results remain typed; neither silently selects another engine. The test also records that generated code read only the package from the current checked request. | `quire-integration` AOT/interpreter conformance tests, across the public driver-to-QSL API. |
| Prove, Kani, replay and terminal records | Extend the existing real-Kani exemplar (`tests/qsl_kani_exemplar.rs`, IT-011) as the execution template. It passed 5/5 with real Kani in 530 seconds serially on the measured Linux run. For G5b, use one generated harness that proves and one that falsifies. Feed the falsifying result through QSL replay. Include a replay mismatch and a second requested item so the test checks item-local records. Capture generated artifact identity, Kani output, replay output and the final `quire-outcome/1` document. | The Kani run is real (not a fabricated provider response). A reproduced counterexample settles `refuted`; a mismatch settles `inconclusive`; CG's FR-331 mapping is the only terminal mapping. The output contains exactly one FR-331 record per requested item and the binary and library paths serialize it identically. | Extend `tests/qsl_kani_exemplar.rs` for the G5b CLI/library composition; retain the real-Kani run as an explicit long gate. The measured 530-second exemplar shows draco's Kani 0.68 setup can run this lane, subject to Linux remaining the authoritative `make ci` gate. |
| Cross-platform process-tree budgets and cleanup | Run native host tests on Linux, macOS and Windows. On each platform, use synchronized fixtures where plugin and descendant are individually below a memory or CPU-time limit but their aggregate crosses it. Include a short-lived descendant that completes CPU work between monitor samples, an unavailable/incomplete-tree measurement case, and a sustained child that remains alive during cleanup. On Windows, also force Job Object setup failure before plugin execution. Record per-platform command, host version, configured limits, sampled observations, typed result, request-frame count, and surviving process-tree members. | Each platform enforces memory and CPU-time limits across the process tree and returns the corresponding typed budget failure while terminating descendants. A short-lived descendant must be accounted for; if the implementation cannot observe its completed CPU use, the evidence records the gap and final G5b remains blocked pending IR-668 resolution. An unavailable complete-tree measurement refuses before sending a request. On Windows, the job is assigned before plugin code executes, closing it terminates remaining members, and setup failure sends no request and cleans up any child already created. A Linux-only result is partial evidence and does not satisfy final IR-528 acceptance. | Native Linux, macOS and Windows runs of `quire-plugin-host` TC-026 and TC-028; preserve separate raw logs and process observations for each OS. |

## Gate assembly

1. Verify that paired cases use the same built executable and library, typed
   request, admitted provider descriptors and manifests, and generated
   artifacts wherever the criterion compares those inputs.
2. Keep unit and fast composition tests in the normal local test gate. Run the
   real Kani prove/replay cases as an explicitly identified slow phase; a
   timeout is a gate result to classify, not a passing or skipped case.
3. Attach case-level outcomes and raw command exit statuses. Do not summarize
   a failed or timed-out case as a green aggregate.
4. Gate G5b passes only when every required group has evidence, the parity
   and cache checks pass, and every requested item has its expected FR-331
   terminal record. The cross-platform process-tree group must pass natively
   on Linux, macOS and Windows; a Linux-only run is partial evidence.

## Dependencies and current gaps

- IR-527 defines the CLI, exit, provider, cache, AOT, proof/replay and
  abstraction-relation requirements that this plan covers.
- IR-607 through IR-610 provide the binary, options, provider/cache and
  execution implementations.
- IR-609's final process-provider negotiation is gated on CG IR-629's public
  origin-aware descriptor conversion, as amended by QSL-637 PV-4. QSL owns
  `ProviderOrigin` and the registration rules; the driver preserves the QSL
  descriptor origin and supplies that descriptor to CG. CG constructs
  `BackendKind::Process` from `ProviderOrigin::Process`; the driver does not
  map plugin `BackendId` values to CG kinds, consistent with FR-005-AC-17.
- QSL LC7 removes the old `quire-spec` binary so the single `quire` CLI can
  own these verbs.
- QSL AR2 is a prerequisite for the SP-D9 bound-item check; QSL AR3 and
  QSL-514 own its cross-repo acceptance test.
- Existing `quire-integration` tests provide reusable evidence; the G5b-
  specific binary parity, provider/plugin, cache, AOT and expanded terminal-
  record assertions remain planned and are not claimed to exist or pass.
