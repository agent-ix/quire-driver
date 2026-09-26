# quire-driver

Orchestrating driver: calls QSL (compile, emit, route), IR (E5 read) and CG (negotiate, generate) across repositories (ADR-011 T-13).

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses sources
make spec           # quire validate 'spec/**/*.md'
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # fmt-check + lint + test + deny + audit-unsafe + docs + spec
```

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.98.1` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to 1.98.1 (QSL's and CG's toolchain) + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
tests/drive.rs         # FR-001 acceptance tests (real QSL and CG calls)
tests/dependency_edges.rs # FR-001-AC-1: cargo metadata edge check
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (FR-001, TC-001..TC-007)
scripts/               # local tooling
```

## Place in the crate graph

QSL ADR-011 §6.1 driver row: this crate depends on the QSL layer crates and on
CG, and nothing depends on it. QSL's and CG's gates never build it. Do not add
a dependency on the QSL root crate `quire-spec-language`; `tests/dependency_edges.rs`
fails if one appears.
