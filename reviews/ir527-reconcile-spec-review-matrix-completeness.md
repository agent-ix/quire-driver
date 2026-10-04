---
id: SR-038
title: spec-review/matrix-completeness review of IR-527 PR 19
type: SpecReview
analysis: base
scope: agent-ix/quire-driver@a3789aa0d4057c21d360cd1f734a10779be2ddd0; spec/assurance/AD-001-driver-architecture.md,
  spec/spec.md, spec/tests.md
review_set: subset
---

## Summary

Ticket: IR-527. Computed matrix has 37 FR criteria: eight tagged (FR-001), twenty-nine untagged (FR-002..FR-010), all with no run evidence. Authored TM-001 lists all 37 criteria and all seventeen TCs. Its explicit planned/test-present distinction agrees with computed status; it claims no implementation completion.

## Verdict

PASS — No defects found in this diff under this method; this is not an implementation or G5b verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

```yaml
scope:
- id: spec/assurance/AD-001-driver-architecture.md#unit-1
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: "---\nid: AD-001\ntitle: \"quire-driver architecture: one CLI and the checked\
    \ request lifecycle\"\ntype: ArchitectureDescription\nstatus: proposed\nowner:\
    \ kreneskyp\nsystem: quire-driver library and its user-facing quire binary, downstream\
    \ of QSL lifecycle operations, IR checked-package admission, CG generation and\
    \ terminal mapping, and providers\nrelationships:\n  - target: ix://agent-ix/quire-driver/US-001\n\
    \    type: references\n  - target: ix://agent-ix/quire-driver/FR-001\n    type:\
    \ references\n  - target: ix://agent-ix/quire-driver/FR-002\n    type: references\n\
    \  - target: ix://agent-ix/quire-driver/FR-00"
- id: spec/assurance/AD-001-driver-architecture.md#unit-2
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'IR-527 reconciles this seam description with QSL ADR-029 and the merged

    SP-D1 through SP-D9 requirements. It absorbs IR-351''s driver specification,

    architecture and verification-inventory work. [The master specification](../spec.md)

    and [TM-001](../tests.md) integrate FR-001 through FR-010 and TC-001 through

    TC-017. This architecture describes their composition and ownership; the

    discrete requirements supply the normative acceptance criteria.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-3
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'The driver repository owns the downstream orchestration library and the
    one

    user-facing `quire` binary. The binary parses arguments, reads explicitly

    named inputs, builds typed requests, calls the appropriate library operation,

    and renders its typed outcome. `prove` and `generate` are driver operations;

    QSL owns the other lifecycle operations and the lifecycle contract. The CLI

    performs no semantic inference from diagnostic text.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-4
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'The driver composes QSL, IR and CG. It owns call order, registry construction,

    provider dispatch, proof execution, counterexample replay orchestration and

    writing one FR-331 terminal record per requested item. QSL owns checking,

    execution and replay semantics and the outcome types. IR owns checked-package

    admission. CG owns negotiation, claim derivation, generation, atomic artifact

    publication and the mapping from provider execution plus replay to QSL''s

    terminal value. The driver defines no alternative to these contracts.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-5
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'QSL ADR-029 also defines the plugin, cache and execution-backend contracts.

    Its CB-3 places the host, cache, AOT adapter and CLI outside the qualified core

    in downstream driver-repository components. That placement does not transfer

    their policy to the driver: FR-006 through FR-008 specify dispatch and typed

    input/result preservation against QSL''s PL, CA and EB contracts. This

    reconciliation neither implements those components nor changes their APIs.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-6
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'The following views describe the request flow, dependency direction and

    failure ownership across the driver and its upstream libraries.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-7
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: '| Boundary | Contract owner | Driver responsibility | Verification |

    |---|---|---|---|

    | User command to typed operation | QSL ADR-029 LC-1, CB-1, CB-4, CB-5; driver
    frontend | Expose one `quire` command surface, dispatch to QSL or driver operations,
    preserve the library outcome and its category exit result | FR-002 / TC-009 |

    | Caller limits and engine choice | QSL ADR-029 LC-4, EB-8 | Pass explicit limits
    and cancellation to the operation; select only the requested engine, preserving
    typed failures | FR-003..FR-004 / TC-010..TC-011 |

    | Compile, check and E4 package emission | QSL typed lifec'
- id: spec/assurance/AD-001-driver-architecture.md#unit-8
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'The existing FR-001 sequence is compile/emit, E5 admission, candidate

    construction, backend conversion, negotiation, routing and generation.

    FR-010 qualifies the CG input: only relation-bound items enter negotiation

    and generation. Selection never discards the need to account for each

    requested item in FR-009''s final outcome. Proving extends generation with

    provider execution, QSL replay and CG settlement; a `generate` request does

    not imply that a proof was executed.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-9
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'Types cross the QSL/CG seam in their owned wire spellings (ADR-013 T-7).

    The driver treats an unknown capability spelling as `VocabularyMismatch`

    and looks up each routed `WireNodeId` in IR''s admitted package to obtain

    its `CheckedNodeId`. It re-derives no checked semantics or claim.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-10
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'FR-001''s existing compile path uses an empty `DependencyInput`, so an
    import

    without supplied selection refuses at intake. This reconciliation does not

    add library-source intake or expand that baseline contract.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-11
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: "- QSL, IR and CG do not depend on the driver. The driver library depends\
    \ on\n  their layer/model APIs; the CLI and `quire-integration` consume those\n\
    \  libraries downstream. CG's QSL seam remains `qsl-replay`; the driver may\n\
    \  call QSL layer crates directly (ADR-011 §6.1, FB-11).\n- IR admission and checked\
    \ types come from `quire-contract-model`, not a\n  driver-local representation.\
    \ The driver, CG and QSL use one copy of that\n  model type, enforced by compilation\
    \ and the existing TC-005 graph check.\n- FR-001 and TC-005 describe the existing\
    \ orchestrator's dependency graph.\n  The frontend and non-"
- id: spec/assurance/AD-001-driver-architecture.md#unit-12
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: '| Condition | Contract owner / driver action | Outcome |

    |---|---|---|

    | Invalid command, option, limit or engine name | Frontend invokes QSL''s argument/outcome
    contract before a lifecycle stage | QSL typed refusal |

    | Recognized engine is not linked | Driver preserves the engine-availability contract
    | Unsupported; no alternate engine runs |

    | Compile, package or replay refuses, reaches a limit, cancels or faults | QSL
    typed operation result passes through | Owner category and cause retained |

    | Manifest registration refuses or a backend has no CG conversion | Driver reports
    the registry/conv'
- id: spec/assurance/AD-001-driver-architecture.md#unit-13
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'QSL''s `quire-outcome/1` value is the machine outcome. QSL''s category/severity

    contract decides the process status, including multi-item outcomes and stream

    failures; the driver and ix-cli-kit introduce no additional mapping.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-14
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: '| Decision | Reason and observable tradeoff |

    |---|---|

    | One user-facing CLI in the driver repository | It can see every lifecycle stage
    without a QSL-to-CG dependency cycle. QSL supplies library operations; a separate
    CG CLI would duplicate the user contract. |

    | Policy remains with each domain owner | Driver tests can check sequence, inputs,
    dispatch and unchanged typed results. Plugin/cache/AOT semantic conformance stays
    with QSL, admission with IR, and generation/settlement with CG. |

    | Replay precedes counterexample settlement | `refuted` requires reproduction
    under QSL''s reference seman'
- id: spec/assurance/AD-001-driver-architecture.md#unit-15
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'The requirement/test inventory in TM-001 is the checkable expression of
    these

    decisions. FR-002..FR-010 correspond to SP-D1..SP-D9; FR-001 retains the

    existing generation seam. CG FR-005''s CLI-facing conformance and IR-337''s

    parity/bound-suite obligations feed G5b through the one binary. CG continues

    to own local atomic publication and generation checks.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-16
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'This is a proposed architecture and a specification integration, not a
    claim

    that the planned lifecycle is implemented. Existing source tests for

    TC-001..TC-008 are present; no source tests are run by this reconciliation.

    The merged FR-002..FR-010 and TC-009..TC-017 specify the planned extension.'
- id: spec/assurance/AD-001-driver-architecture.md#unit-17
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: '| Open work or risk | Owner | Required evidence / review point |

    |---|---|---|

    | Existing FR-001 text and source calls refer to the earlier `spine::compile`
    entry; current QSL uses typed lifecycle operations | IR-606 | Adapt call sites
    to current QSL/IR/CG APIs and establish the orchestration tests on the adapted
    implementation. No compatibility shim or implementation change is made here. |

    | CLI, caller limits/engine selection and provider/plugin/cache composition remain
    planned | IR-607, IR-608, IR-609 | Implement and run TC-009..TC-014 against their
    owning contracts. |

    | AOT and execute/rep'
- id: spec/assurance/AD-001-driver-architecture.md#unit-18
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'Review these seams when each owning implementation is proposed and again

    before G5b acceptance. IR-351''s shared spec work is absorbed by IR-527;

    implementation delivery and gate acceptance stay with their named tickets.'
- id: spec/spec.md#unit-1
  path: spec/spec.md
  role: examined
  excerpt: "---\ntype: master-requirements\nname: quire-driver\norg: agent-ix\ncomponent_type:\
    \ rust-lib\nimplementation_language: rust\nstandards_alignment:\n  - iso-iec-ieee-29148\n\
    relationships:\n  - target: ix://agent-ix/quire-driver/US-001\n    type: contains\n\
    \  - target: ix://agent-ix/quire-driver/AD-001\n    type: contains\n  - target:\
    \ ix://agent-ix/quire-driver/TM-001\n    type: contains\n  - target: ix://agent-ix/quire-driver/FR-001\n\
    \    type: contains\n  - target: ix://agent-ix/quire-driver/FR-002\n    type:\
    \ contains\n  - target: ix://agent-ix/quire-driver/FR-003\n    type: contains\n\
    \  - target: ix://agent-ix/quire-d"
- id: spec/spec.md#unit-2
  path: spec/spec.md
  role: examined
  excerpt: '`quire-driver` is the downstream driver library and the owner of the one

    user-facing `quire` binary for the checked request lifecycle. It coordinates

    QSL, IR and CG operations while leaving each upstream''s domain policy with

    that owner. This specification reconciles the driver scope formerly tracked

    under IR-351 as IR-527.'
- id: spec/spec.md#unit-3
  path: spec/spec.md
  role: examined
  excerpt: 'The driver orchestrates checked compile and package handoff, backend

    registration and negotiation, routing, generation, execution, replay, and

    terminal outcome recording. It provides one command line across the lifecycle

    operations defined by QSL ADR-029. Driver operations own `prove` and

    `generate`; the remaining lifecycle operations delegate to their typed QSL

    library entries.'
- id: spec/spec.md#unit-4
  path: spec/spec.md
  role: examined
  excerpt: 'QSL owns lifecycle policy and the `quire-outcome/1`, diagnostic, exit-category

    and operation contracts. The driver preserves those typed outcomes and uses

    QSL''s `Category::exit_code` as the sole category-to-process exit mapping.

    `ix-cli-kit` supplies shared CLI mechanics. FR-002 defines the direct numeric

    pass-through at the process boundary until the kit can carry QSL''s codes.'
- id: spec/spec.md#unit-5
  path: spec/spec.md
  role: examined
  excerpt: "- QSL syntax, checking, replay semantics, plugin protocol, cache policy\
    \ and\n  execution-backend semantics; this repository specifies their composition\n\
    \  according to QSL ADR-029, not replacement contracts.\n- IR checked-package\
    \ admission semantics, and CG negotiation, claim derivation,\n  generation, atomic\
    \ publication and terminal mapping semantics.\n- A competing QSL or CG user CLI,\
    \ a second outcome schema or exit mapping,\n  and automatic engine substitution.\n\
    - G5b test implementation or gate evidence in this repository. IR-528 owns\n \
    \ that evidence in `quire-integration`.\n- Implementation chang"
- id: spec/spec.md#unit-6
  path: spec/spec.md
  role: examined
  excerpt: 'A downstream Rust driver library and one user-facing `quire` binary. The

    binary builds typed requests, calls QSL or driver operations, and renders the

    returned outcome. The library sequences QSL, IR and CG calls and builds the

    registry value passed to consumers. QSL, IR and CG do not depend on the driver;

    the driver frontend and cross-repository integration suite are downstream

    consumers. CLI mechanics remain outside the qualified core (QSL ADR-029 CB-3).'
- id: spec/spec.md#unit-7
  path: spec/spec.md
  role: examined
  excerpt: '`quire-driver` owns the sequencing across QSL, IR and CG for the checked

    request lifecycle and supplies the sole user-facing `quire` binary. It sits

    downstream of those libraries. IR-527 absorbs the driver spec, architecture,

    requirements and test-matrix scope previously tracked by IR-351. The tables

    index the discrete normative artifacts; they do not replace their criteria.

    FR-002 through FR-010 specify SP-D1 through SP-D9 respectively.'
- id: spec/spec.md#unit-8
  path: spec/spec.md
  role: examined
  excerpt: '| ID | Title |

    |---|---|

    | [US-001](usecase/US-001-run-a-checked-request.md) | Run one checked request
    from source to routed backends |

    | [FR-001](functional/FR-001-orchestrate-the-driver-steps.md) | Orchestrate compile,
    backend conversion, negotiation, routing and generation |

    | [FR-002](functional/FR-002-expose-driver-cli-and-outcome-contract.md) | Expose
    the lifecycle through one driver CLI and outcome contract |

    | [FR-003](functional/FR-003-apply-caller-selected-limits.md) | Apply caller-selected
    lifecycle limits |

    | [FR-004](functional/FR-004-select-the-execution-engine.md) | Select the e'
- id: spec/spec.md#unit-9
  path: spec/spec.md
  role: examined
  excerpt: '[TM-001](tests.md) integrates FR-001 through FR-010 and TC-001 through
    TC-017.

    It distinguishes existing orchestration tests from planned driver tests and

    records their verification owners. Cross-backend and CLI/library conformance

    formerly described by CG FR-005 and IR-337 feed IR-528''s G5b gate exclusively

    in `quire-integration`; CG retains its local generation and atomic publication

    obligations. TC-017 is a QSL-owned cross-repository binding test in that suite

    and awaits QSL AR2 / QSL-514.'
- id: spec/spec.md#unit-10
  path: spec/spec.md
  role: examined
  excerpt: 'This reconciliation integrates the merged SP-D1–D9 specifications from
    driver

    PRs #15 and #17. It does not certify implementation: IR-606 adapts the existing

    orchestrator to current upstream lifecycle APIs; IR-607 through IR-610 deliver

    the CLI, options, providers, cache, AOT and proof execution chain. IR-351''s

    shared specification work is absorbed here, with no separate dependency

    revision ledger or policy artifact.'
- id: spec/spec.md#unit-11
  path: spec/spec.md
  role: examined
  excerpt: "- [QSL ADR-029](https://github.com/agent-ix/quire-spec-language/blob/main/spec/decisions/ADR-029-lifecycle-cli-provider-plugin-cache-codegen-boundaries.md),\
    \ with ADR-011, ADR-012 and ADR-013.\n- CG FR-005 CLI conformance and IR-337 parity\
    \ scope; the one CLI is specified\n  here, while CG retains codegen-specific atomic\
    \ publication requirements.\n- [IR-527](https://linear.app/agent-ix/issue/IR-527)\
    \ owns this reconciliation;\n  [IR-528](https://linear.app/agent-ix/issue/IR-528)\
    \ owns G5b gate evidence in\n  `quire-integration`.\n- [Driver architecture](assurance/AD-001-driver-architecture.md).\n\
    - [Drive"
- id: spec/tests.md#unit-1
  path: spec/tests.md
  role: examined
  excerpt: '---

    id: TM-001

    title: "Driver command and execution test matrix"

    type: TestMatrix

    ---

    # TM-001: Driver binary, provider and execution test matrix'
- id: spec/tests.md#unit-2
  path: spec/tests.md
  role: examined
  excerpt: 'This authored verification inventory integrates FR-001 and SP-D1 through

    SP-D9 (FR-002 through FR-010) from IR-527. It covers the frozen merged

    TC-001 through TC-017 set. Discrete FR and TC artifacts remain authoritative;

    this matrix is their index, not a computed implementation-coverage report.'
- id: spec/tests.md#unit-3
  path: spec/tests.md
  role: examined
  excerpt: '`✅` means the existing source contains the test implementations named
    by

    TC-001 through TC-008; it does not claim a fresh test run or compatibility

    with current upstream APIs. `🚧` means verification is planned. This spec-only

    change runs artifact validation, not source tests or the G5b gate.'
- id: spec/tests.md#unit-4
  path: spec/tests.md
  role: examined
  excerpt: '| Scope | Verification owner and location |

    |---|---|

    | FR-001 / TC-001..TC-008 | Driver orchestration tests in `tests/drive.rs`, `tests/dependency_edges.rs`
    and `src/refusal.rs`, as named by the TC artifacts; upstream API adaptation is
    IR-606. |

    | FR-002..FR-009 / TC-009..TC-016 | Planned driver CLI and composition tests at
    the locations named by each TC; they check dispatch, input fidelity, typed result
    preservation and single-owner mappings. |

    | Plugin protocol, cache policy, execution semantics and AOT/interpreter parity
    | QSL ADR-029 owner suites; driver TC-013..TC-015 check composition b'
- id: spec/tests.md#unit-5
  path: spec/tests.md
  role: examined
  excerpt: 'The driver supplies the one user-facing CLI. CG FR-005''s CLI-facing

    conformance is exercised through that binary; CG keeps its local generator

    and atomic publication tests. CG''s cross-backend parity and IR-337''s

    outcome-per-backend and bound-suite coverage feed the G5b evidence rather

    than a second driver or CG gate. The driver and QSL suites supply local

    verification; they do not substitute for IR-528 acceptance evidence.'
- id: spec/tests.md#unit-6
  path: spec/tests.md
  role: examined
  excerpt: '| Functional Req | Acceptance Criteria | Test Cases | Status |

    |---|---|---|---|

    | FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5, FR-001-AC-6,
    FR-001-AC-7, FR-001-AC-8 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007,
    TC-008 | ✅ Tests present |

    | FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-009
    | 🚧 Planned |

    | FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3 | TC-010 | 🚧 Planned |

    | FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-011 | 🚧 Planned
    |

    | FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3 | TC-012 | 🚧 Pla'
- id: spec/tests.md#unit-7
  path: spec/tests.md
  role: examined
  excerpt: '| Test ID | Title | Type | Traces To | Status |

    |---|---|---|---|---|

    | TC-001 | The steps run in ADR-011 T-13 order | Integration | FR-001-AC-4 | ✅
    Tests present |

    | TC-002 | An unknown BackendId refuses the run before negotiation | Integration
    | FR-001-AC-2 | ✅ Tests present |

    | TC-003 | route runs after negotiation, for supported items only | Integration
    | FR-001-AC-3 | ✅ Tests present |

    | TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5 | ✅ Tests
    present |

    | TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 | Integration | FR-001-AC-1
    | ✅ Tests present |

    | TC-0'
- id: FR-001
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: "The driver SHALL run one request through these steps, in this order (QSL\n\
    ADR-011 T-13, ADR-012 §7.2):\n\n1. S1 to S4 through `qsl_replay::spine::compile`,\
    \ with an empty\n   `DependencyInput` (ADR-015 D-1): the driver does not yet support\n\
    \   compiling a unit's `import`s against caller-supplied library source, so\n\
    \   any `import` refuses at the intake stage,\n   `missing_import`/`missing-selection`.\n\
    2. E4: the `quire.checked-package/v2` bytes and `package_id` that the same\n \
    \  call emits.\n3. The `route` candidate step for each requested item.\n4. The\
    \ pre-negotiation conversion. Every registered `Backen"
- id: FR-001-AC-1
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: 'The driver''s dependency edges match the QSL ADR-011 §6.1 allow-list and
    §7.1 crate graph: its normal and build dependencies are QSL layer crates, CG,
    `quire-contract-model` and `thiserror` (`serde_json` is a dev-dependency, not
    a normal one), the driver''s `quire-contract-model` is the one CG''s IR crate
    depends on, nothing in its lock depends on it, CG''s one QSL edge is `qsl-replay`,
    and the lock holds exactly one `quire-contract-model` package, so the driver''s
    direct edge, CG''s IR crate and QSL''s `qsl-package` all resolve the same package
    id.'
- id: FR-001-AC-2
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: A run whose registry holds a `BackendId` with no CG kind refuses before
    any negotiation, with `invalid_capability`/`unknown-backend` and the unconverted
    ids in bytewise order.
- id: FR-001-AC-3
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: '`route` is called only after negotiation, and only for `supported` items.'
- id: FR-001-AC-4
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: 'The steps run in the order of the Description: compile and emit, the E5
    read, the candidate step, conversion, negotiation, routing, generation.'
- id: FR-001-AC-5
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: Each command-level refusal carries its step, catalog code and outcome kind.
    A `Compile` refusal's code renders `code/cause` when the refusing stage names
    a cause (e.g. `missing_import`/`missing-selection`), not just the bare code.
- id: FR-001-AC-6
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: CG generation is called once, after routing, over the routed items only,
    each with its request index, its admitted `CheckedNodeId`, its routed backend
    and its kind; its output is keyed by request index, ascending; a missing kind
    context refuses the run, and a routed item naming a node the checked package does
    not hold is an internal fault.
- id: FR-001-AC-7
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: The E5 read admits the emitted v2 bytes at the emitted `package_id`; an
    admitted package at another `package_id`, or an IR refusal, is an internal fault
    at the hand-off before any backend conversion.
- id: FR-001-AC-8
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: CG generation's output carries the FR-014 claim map CG derived for the
    Kani group (IR-294), `Some` whenever one ran; a node whose operation has no derivable
    claim settles `Unsupported` with `no_derivable_claim` and does not affect its
    routed siblings' records or harnesses.
- id: FR-002
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: context_only
  excerpt: 'The driver SHALL build the one user-facing `quire` binary. Its lifecycle

    commands, output options, multi-item severity and process behavior SHALL

    conform to QSL ADR-029 LC-1, CB-1, CB-4 and CB-5.


    Each verb SHALL call its corresponding typed QSL or driver operation.

    The CLI SHALL not make semantic decisions based on diagnostic text.


    In machine mode, each verb SHALL write exactly one QSL-defined

    `quire-outcome/1` JSON value to stdout and preserve the typed library outcome

    without changing its schema or values. The driver SHALL use QSL''s single

    total `Category::exit_code` function as the sole c'
- id: FR-002-AC-1
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: context_only
  excerpt: The driver binary exposes the command and output surface defined by QSL
    ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation.
- id: FR-002-AC-2
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: context_only
  excerpt: For each O-16 category, the process status equals the value returned by
    QSL's `Category::exit_code`; the CLI has no second category-to-exit mapping.
- id: FR-002-AC-3
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: context_only
  excerpt: In machine mode, each command writes one parseable `quire-outcome/1` value
    equal to the library outcome under QSL's schema; the CLI adds, removes or changes
    no outcome data.
- id: FR-002-AC-4
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: context_only
  excerpt: The CLI uses ix-cli-kit for shared command/output, configuration and version
    mechanics, while preserving QSL's numeric exit result without the kit's typed
    `Outcome` remapping.
- id: FR-002-AC-5
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: context_only
  excerpt: The CLI reads only input paths named by its arguments and reads no unnamed
    configuration file.
- id: FR-003
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: context_only
  excerpt: 'When a caller supplies one or more `--limit` options, the driver SHALL
    apply

    those named values to the matching fields of the typed limits passed to the

    selected lifecycle operation. When a caller omits a limit field, the

    operation SHALL use that field''s published QSL default. Every QSL-enforced

    bound SHALL be caller-configurable. When an operation reaches a bound, it SHALL

    return QSL''s typed limit outcome and report no success.

    '
- id: FR-003-AC-1
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: context_only
  excerpt: Setting one valid `--limit` overrides only its named field; every omitted
    field retains its published default, and the effective limits passed to the operation
    equal those values.
- id: FR-003-AC-2
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: context_only
  excerpt: An unknown field, malformed value or invalid duplicate `--limit` is refused
    before any stage with QSL's refusal category and names the option value.
- id: FR-003-AC-3
  path: spec/functional/FR-003-apply-caller-selected-limits.md
  role: context_only
  excerpt: An input that reaches a caller-lowered bound returns typed `LimitExceeded`
    with the configured value and field name and QSL's incomplete category; the process
    status equals QSL's category exit function. Raising the same bound to fit the
    input allows the operation to complete.
- id: FR-004
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: context_only
  excerpt: 'For an execution request, the driver SHALL select the engine named by

    `--engine interpreter|aot|jit`; when the option is absent, it SHALL select

    `interpreter`. The driver SHALL run only the selected engine. It SHALL refuse

    before any lifecycle stage when the selected engine is not linked into the

    binary, and SHALL NOT substitute another engine.

    '
- id: FR-004-AC-1
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: context_only
  excerpt: An execution request without `--engine` runs the interpreter and returns
    its result.
- id: FR-004-AC-2
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: context_only
  excerpt: A value outside the closed engine set is refused before any stage with
    QSL's refusal category and names the supplied value.
- id: FR-004-AC-3
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: context_only
  excerpt: A recognized engine absent from the binary is refused before any stage
    with category unsupported; no other engine runs, and process status equals QSL's
    category exit function.
- id: FR-004-AC-4
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: context_only
  excerpt: When the selected engine returns unsupported or a typed preparation failure,
    that result is returned and no other engine is invoked.
- id: FR-005
  path: spec/functional/FR-005-register-first-party-providers.md
  role: context_only
  excerpt: 'The driver SHALL expose one `Provider` trait in its library for each first-party

    provider linked into the binary. The trait SHALL expose the provider''s FR-331

    manifest and run the routed request with the emitted package, provider limits

    and cancellation handle, returning typed provider results. The driver SHALL

    register each provider through its manifest and SHALL pass the resulting

    registry to the lifecycle operation that uses it.

    '
- id: FR-005-AC-1
  path: spec/functional/FR-005-register-first-party-providers.md
  role: context_only
  excerpt: A first-party provider implementing `Provider` returns its FR-331 manifest,
    which the driver registers with the backend identity equal to the manifest identity.
- id: FR-005-AC-2
  path: spec/functional/FR-005-register-first-party-providers.md
  role: context_only
  excerpt: The provider receives the same emitted package, routed request, limits
    and cancellation handle supplied by the driver, and its typed results are returned
    without semantic changes.
- id: FR-005-AC-3
  path: spec/functional/FR-005-register-first-party-providers.md
  role: context_only
  excerpt: Two providers with distinct manifest identities are registered as distinct
    backends; a run invokes only the provider selected for its routed items.
- id: FR-006
  path: spec/functional/FR-006-host-provider-plugins.md
  role: context_only
  excerpt: 'When backend negotiation selects an out-of-process provider, the driver
    SHALL

    dispatch only that provider''s negotiated items to the QSL plugin host, using

    the executable path named by the caller. The driver SHALL feed the returned

    typed provider results into the same proof settlement path used for

    first-party providers.

    '
- id: FR-006-AC-1
  path: spec/functional/FR-006-host-provider-plugins.md
  role: context_only
  excerpt: When negotiation selects a process provider, the driver invokes the QSL
    plugin host with the caller-named executable path, admitted manifest, current
    emitted package bytes and exactly that provider's negotiated items.
- id: FR-006-AC-2
  path: spec/functional/FR-006-host-provider-plugins.md
  role: context_only
  excerpt: Typed results returned by the QSL plugin host enter the same driver proof
    settlement path as first-party provider results, while unrelated routed items
    retain their own provider results.
- id: FR-007
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: context_only
  excerpt: 'When the driver enables caching for an eligible operation, it SHALL pass
    the

    selected provider''s manifest bytes as content input to QSL''s cache contract.

    The driver SHALL NOT substitute a recorded repository revision or version

    pin for those bytes.

    '
- id: FR-007-AC-1
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: context_only
  excerpt: For an eligible cached request, the driver supplies the typed operation
    request and exact selected provider manifest bytes to the QSL cache interface.
- id: FR-007-AC-2
  path: spec/functional/FR-007-cache-results-by-content-identity.md
  role: context_only
  excerpt: Changing manifest bytes while keeping any recorded revision or version
    pin fixed changes the content input supplied by the driver; the driver does not
    replace those bytes with the pin.
- id: FR-008
  path: spec/functional/FR-008-aot-execution-backend.md
  role: context_only
  excerpt: 'When an execution request selects AOT, the driver SHALL invoke QSL''s
    AOT

    backend with only the checked package and selected entries from that request.

    The driver SHALL return the backend''s typed result or failure without

    substituting another engine.

    '
- id: FR-008-AC-1
  path: spec/functional/FR-008-aot-execution-backend.md
  role: context_only
  excerpt: When a request selects AOT, the driver calls the backend with the checked
    package and selected entries from that request, its limits and cancellation input,
    and the caller-named toolchain.
- id: FR-008-AC-2
  path: spec/functional/FR-008-aot-execution-backend.md
  role: context_only
  excerpt: The driver returns the backend's typed results and preparation failures
    unchanged and does not invoke another engine after an AOT failure.
- id: FR-009
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: 'When the caller requests a proof, the driver SHALL run the generated provider

    request and replay each counterexample through QSL''s replay operation. The

    driver SHALL produce exactly one FR-331 terminal record for each requested

    item. The driver SHALL preserve the independent result and cause from

    execution and replay when settling the item.

    '
- id: FR-009-AC-1
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: The proof provider receives the generated request and the exact checked-package
    bytes emitted by the same request.
- id: FR-009-AC-2
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: A provider counterexample settles `refuted` only when QSL replay reproduces
    it; a disagreement or completed non-reproduction settles `inconclusive` with a
    replay-parity cause.
- id: FR-009-AC-3
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: A replay refusal retains its typed cause and a replay fault is an internal
    failure; neither is converted to `refuted`.
- id: FR-009-AC-4
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: The terminal result is composed by CG's FR-331 mapping from both the provider
    result and replay result; the driver introduces no second mapping.
- id: FR-009-AC-5
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: The outcome contains exactly one FR-331 terminal record for every requested
    item, including unsupported and incomplete items.
- id: FR-010
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: 'When an abstraction relation binds a requested item, the driver SHALL
    send

    only the bound items to CG''s negotiation and generation operations. The

    driver SHALL not send the package''s unbound item set to CG.

    '
- id: FR-010-AC-1
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: Given a package whose abstraction relation binds a subset of requested
    items, CG negotiation receives exactly that subset and receives no unbound item.
- id: FR-010-AC-2
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: CG generation receives only items that the same admitted relation bound
    and that negotiation settled for generation.
- id: FR-010-AC-3
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: When a binding required for emission is absent, the driver returns the
    QSL-defined typed unbound-element refusal and sends no unbound item to CG.
- id: TC-001
  path: spec/test-cases/TC-001-steps-run-in-order.md
  role: context_only
  excerpt: "- The call log is: compile, the E5 read, the conversion of `kani`, one\n\
    \  negotiation of one item, one routing call, one generation call over\n  one\
    \ item. (The candidate step calls no upstream\n  function the recorder sees.)\n\
    - The emitted v2 bytes are not empty.\n"
- id: TC-002
  path: spec/test-cases/TC-002-unknown-backend-refusal.md
  role: context_only
  excerpt: "- The run refuses with `invalid_capability`/`unknown-backend` at the\n\
    \  conversion step, outcome kind refusal.\n- It names `Zeta` then `alpha`, bytewise\
    \ order, which a case-insensitive\n  order would reverse.\n- All three ids went\
    \ through the conversion, and negotiation and routing\n  were never called.\n"
- id: TC-003
  path: spec/test-cases/TC-003-route-after-negotiation.md
  role: context_only
  excerpt: "- Step 4: `route` and CG generation each see one item, the `-` at request\n\
    \  index 1 -- not at list position 0, which the unsupported item 0 leaves\n  it\
    \ as. `route` is called with one disposition (`supported: 1,\n  dispositions:\
    \ 1`), and generation is called over one item. Both the\n  routed and the generated\
    \ item carry request index 1, not 0: keying by\n  list position instead of `settlement.request_index`\
    \ would put 0 there\n  instead, and fail this test.\n- Step 5: `route` is never\
    \ called, generation is never called, and the run\n  still completes (an empty\
    \ request negotiates zero items).\n"
- id: TC-004
  path: spec/test-cases/TC-004-command-level-refusals.md
  role: context_only
  excerpt: "- Step 1 refuses at the source stage with `invalid_syntax`, outcome kind\n\
    \  refusal, and only the compile call was made.\n- Step 2 refuses at the check\
    \ stage with\n  `stage_limit_exceeded`/`nesting-depth-exceeded`, outcome kind\
    \ limit.\n- Step 3 refuses at the registry with `invalid_capability`/`duplicate-backend`,\n\
    \  one refusal per distinct conflicting digest (two), ascending: the A, B, A\n\
    \  case still yields exactly those same two, not a third for the repeated\n  digest\
    \ -- `build_registry` reads `Registry::refusals()` once, after every\n  manifest\
    \ is registered, rather than accumulating what each `r"
- id: TC-005
  path: spec/test-cases/TC-005-dependency-edges.md
  role: context_only
  excerpt: "- The driver's normal and build dependencies are exactly `qsl-foundation`,\n\
    \  `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,\n  `quire-contract-codegen`,\
    \ `quire-contract-model` and `thiserror`. The QSL\n  root crate is not one. `serde_json`\
    \ is a dev-dependency (this crate's own\n  tests use it), not a normal one.\n\
    - The driver's `quire-contract-model` is the package CG's\n  `quire-contract-ir`\
    \ depends on.\n- One package is named `quire-driver`, and no edge in the lock\
    \ reaches a\n  package of that name: nothing in this lock depends on the driver.\n\
    - CG's one normal or build edge into QSL is "
- id: TC-006
  path: spec/test-cases/TC-006-generation-through-cg.md
  role: context_only
  excerpt: "- The admitted package's `package_id` digest is the emitted `package_id`,\n\
    \  and it equals step 1's package.\n- The driver's generation output equals step\
    \ 3's, with one item at request\n  index 0 and the `kani` wire pair. Its record\
    \ is `Supported` with a scalar\n  harness ranging its one argument over `z`'s\
    \ `[0, 9]` domain: QSL-261 is\n  resolved by IR-296 and IR-297; IR-298 then resolves\
    \ the result role from\n  what the node itself names, and a single-operand node\
    \ like this one has\n  only one reachable candidate, so no own-bound ambiguity\
    \ check ever runs.\n  The generation output's `claim_map` is `S"
- id: TC-007
  path: spec/test-cases/TC-007-generation-refusals.md
  role: context_only
  excerpt: "- Step 1 refuses with CG's `MissingKindContext` for Kani, at the generation\n\
    \  step, code `invalid-request`, outcome kind refusal, after one generation\n\
    \  call over one item.\n"
- id: TC-008
  path: spec/test-cases/TC-008-missing-import-refusal.md
  role: context_only
  excerpt: "- The run refuses at `DriverStage::Compile(SpineStage::Intake)`.\n- The\
    \ refusal's code is `missing_import/missing-selection`, not the bare\n  `missing_import`.\n\
    - The outcome kind is a refusal, not a limit or a fault.\n- For step 2, the refusal\
    \ is `CompileRefusal::Profile` at\n  `DriverStage::Compile(SpineStage::Assembly)`,\
    \ its code is\n  `unknown_profile/unsupported-selection`, and the outcome kind\
    \ is a\n  refusal.\n"
- id: TC-009
  path: spec/test-cases/TC-009-cli-outcome-and-exit-contract.md
  role: context_only
  excerpt: "- Valid verbs and format options reach their corresponding operations.\n\
    - Invalid arguments are refused before any stage with QSL's refusal category\n\
    \  and name the refused argument.\n- Each category and multi-item result exits\
    \ with the exact code selected by\n  QSL's single exit function; no second mapping\
    \ changes the result.\n- Shared CLI mechanics use ix-cli-kit without mapping QSL\
    \ categories through\n  the kit's typed `Outcome` values.\n- JSON mode emits one\
    \ QSL-defined `quire-outcome/1` value equal to the typed\n  library outcome. A\
    \ stream write failure returns the internal-failure\n  category thr"
- id: TC-010
  path: spec/test-cases/TC-010-caller-selected-limits.md
  role: context_only
  excerpt: "- The explicit value replaces only its field; omitted fields retain defaults.\n\
    - Invalid options are refused before any stage with QSL's refusal category.\n\
    - The low limit returns `LimitExceeded` with the configured value and field,\n\
    \  category incomplete and the process status selected by QSL's category exit\n\
    \  function. The raised limit allows the operation to complete.\n"
- id: TC-011
  path: spec/test-cases/TC-011-execution-engine-selection.md
  role: context_only
  excerpt: "- The omitted option selects the interpreter.\n- An unknown name is refused\
    \ before any stage with QSL's refusal category.\n- A recognized but unavailable\
    \ engine is refused before any stage with exit\n  status selected by QSL's category\
    \ exit function and category unsupported.\n- A selected engine's unsupported or\
    \ preparation result is returned without\n  invoking another engine.\n"
- id: TC-012
  path: spec/test-cases/TC-012-first-party-provider-interface.md
  role: context_only
  excerpt: "- Each manifest's identity becomes the corresponding registered backend\n\
    \  identity.\n- Each provider receives the request package, routed items, limits\
    \ and\n  cancellation handle as supplied; typed results pass through unchanged.\n\
    - Each provider runs only for its selected items, and distinct manifest\n  identities\
    \ remain distinct backends.\n"
- id: TC-013
  path: spec/test-cases/TC-013-plugin-host-protocol-and-failures.md
  role: context_only
  excerpt: "- The driver gives the QSL plugin host the configured executable path,\n\
    \  admitted manifest, package bytes from the current request and only the\n  process\
    \ provider's negotiated items.\n- The host's typed result reaches the driver's\
    \ proof settlement input, and\n  the first-party provider's result remains associated\
    \ with its own items.\n- Plugin frame validation, lifecycle and typed failure\
    \ behavior match the\n  authoritative QSL ADR-029 PL-1 through PL-7 contract.\n"
- id: TC-014
  path: spec/test-cases/TC-014-cache-content-identity-and-validation.md
  role: context_only
  excerpt: "- Each cache invocation receives the typed request and exact manifest\
    \ bytes\n  selected for that invocation.\n- The second invocation receives the\
    \ changed bytes while the recorded pin is\n  unchanged; the driver has not replaced\
    \ manifest content with pin metadata.\n- Cache key construction, miss/hit behavior,\
    \ validation and storage follow\n  QSL ADR-029 CA-1 through CA-5 and are verified\
    \ by QSL-owned tests.\n"
- id: TC-015
  path: spec/test-cases/TC-015-aot-matches-interpreter.md
  role: context_only
  excerpt: "- The AOT backend receives only the checked package and entries produced\
    \ for\n  the current request, with its exact limits, cancellation handle and named\n\
    \  toolchain.\n- The driver's result equals the backend's typed evaluation or\
    \ typed\n  preparation failure, including the typed fault when the prepared\n\
    \  executable is unavailable.\n- The driver makes no interpreter or alternate-engine\
    \ call after either\n  failure.\n- AOT/interpreter semantic parity is verified\
    \ by the QSL ADR-029 EB-5\n  conformance evidence, not re-specified by this driver-boundary\
    \ test.\n"
- id: TC-016
  path: spec/test-cases/TC-016-execute-replay-and-terminal-records.md
  role: context_only
  excerpt: "- The provider receives the emitted package from the same request.\n-\
    \ A counterexample is `refuted` only when replay reproduces it; parity\n  disagreement\
    \ is `inconclusive`, replay refusal keeps its cause, and replay\n  fault is an\
    \ internal failure.\n- CG's mapping is used without a second driver mapping.\n\
    - The outcome has exactly one terminal record for every requested item,\n  including\
    \ unsupported and incomplete items.\n"
- id: TC-017
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: context_only
  excerpt: "- CG negotiation receives exactly the bound subset; generation receives\n\
    \  only bound items subsequently routed for generation.\n- No unbound item reaches\
    \ CG. Where a required binding is absent, the\n  driver returns QSL's typed unbound-element\
    \ refusal.\n"
```

## Analysis

Computed matrix has 37 FR criteria: eight tagged (FR-001), twenty-nine untagged (FR-002..FR-010), all with no run evidence. Authored TM-001 lists all 37 criteria and all seventeen TCs. Its explicit planned/test-present distinction agrees with computed status; it claims no implementation completion.
