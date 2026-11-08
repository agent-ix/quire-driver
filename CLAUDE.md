# quire-driver

Orchestrating driver: calls QSL (compile, emit, route), IR (E5 read) and CG (negotiate, generate) across repositories.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make tools          # install/check rustfmt, clippy, cargo-deny, and quire
make deny           # cargo deny check licenses sources
make spec           # quire validate 'spec/**/*.md'
make ci             # fmt-check + lint + test + deny + docs + spec
```

The `quire-cli` tests `signal_is_forwarded_after_active_cleanup_and_defaults_without_a_session`
and `plugin_session_signals_cancel_and_reap_each_active_lifecycle_stage` need
normal, unsandboxed cross-process signal and process-group permissions. A
sandbox failure can report `signal() == None` where SIGTERM/SIGINT was expected,
or exceed a bounded wait (observed in about 6–7 s in sandbox runs versus about
14 s with normal permissions). Rerun under normal, unsandboxed permissions.

## Safety scaffolding

- `clippy.toml` caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- Workspace Clippy `undocumented_unsafe_blocks` checks safety comments. Every workspace crate also forbids unsafe code at compile time.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.

## Layout

```
src/lib.rs             # quire-driver library root
quire-plugin-host/     # process-provider transport and lifecycle boundary
tests/drive.rs         # driver acceptance tests (real QSL and CG calls)
tests/dependency_edges.rs # driver package-graph assertion
quire-cli/src/cli.rs   # user-facing command parser and handlers
quire-cli/src/main.rs  # quire binary entry point
quire-cli/tests/       # CLI contract and binary process tests
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts
scripts/               # local tooling
```

## Place in the package graph

The `quire-driver` package depends on the QSL layer crates
and CG. Do not add a dependency from `quire-driver` to the downstream
`quire-cache` package. The separate `quire-cli` package depends on the driver
interface and cache package. Do not
add a dependency on the QSL root crate
`quire-spec-language`; `tests/dependency_edges.rs` fails if one appears.
