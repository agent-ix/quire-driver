# quire-driver

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)
[![Quire](https://img.shields.io/badge/Quire-Specification%20Language-1F6FEB)](https://github.com/agent-ix/quire-spec-language)

quire-driver runs a QSL specification end to end and builds the `quire`
command-line tool.

It reads the source, has QSL check and package the rule, passes the checked
package to Contract IR and the code generators, and reports the result with one
exit code. It is the only component that calls both QSL and the code
generators.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="The quire command takes QSL source through parse, check and compile to a checked package, and its run verb evaluates a function to an outcome with an exit code, all working today. The driver library also reads the package back, routes it, negotiates with backends and generates code, but no command exposes that yet. The prove and generate commands and their verdict are planned." src="docs/images/hero-light.svg">
</picture>

> **Status: prerelease, private repository.** Nothing here is published
> (`publish = false`). Today the `quire` binary parses, formats, checks,
> compiles and runs QSL. The route, negotiate and generate steps work as a
> library and are tested, but no command runs them yet. Expect breaking
> changes.

## Commands

The binary is built from the `quire-cli` package. Its verbs follow the QSL
command-line design:

| Verb | State | What it does |
| --- | --- | --- |
| `parse`, `check`, `compile`, `run` | works | Run the QSL stage of that name on one source file and print QSL's result document. Use `--format json`; text rendering is not available yet and is refused as unsupported. |
| `format` | works | Print the source in QSL's canonical layout. |
| `analyze`, `monitor`, `prove`, `replay`, `generate`, `inspect`, `render` | planned | Recognized, but they end as `unsupported` (exit 21) until a handler is written. |

`run` accepts `--engine interpreter|aot|jit` and `--limit` overrides. Only the
interpreter is linked, so the other engines are refused before anything runs;
the driver does not substitute one engine for another. The command supplies an
empty object environment because it has no way to pass one in yet.

Every command ends with QSL's own exit code, passed through unchanged:

| Exit code | Meaning |
| --- | --- |
| 0 | success |
| 10 | violation or undefined |
| 20 | refused |
| 21 | unsupported |
| 22 | incomplete or inconclusive |
| 30 | internal failure |

### Example

This source is `quire-cli/tests/fixtures/ir608-run.qsl`, the file the
`run_without_engine_executes_with_the_interpreter` test in
`quire-cli/tests/binary_process.rs` runs:

```
language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function yes using v(): Boolean pure { true }
```

The same test runs it like this and expects exit code 0:

```
quire run quire-cli/tests/fixtures/ir608-run.qsl yes --format json
```

## Library

The `quire-driver` library crate exposes `drive`, which runs one request
through every stage below. QSL knows nothing about code generation, and the
generators know nothing about source files. The tests in `tests/drive.rs` call
real QSL and code-generator crates.

1. Parse, select, check and package the source with QSL, sharing one `Cancel`
   handle. The result is the `checked-package/v2` bytes and their identity.
2. Read those bytes back through Contract IR's reader. The package it admits
   must carry the identity QSL just emitted.
3. Write the items to negotiate from the package's own checked requirement
   records, one per record. The caller does not supply them. A unit with no
   scalar operation application has no records, so nothing is routed and
   nothing is generated.
4. Convert every registered backend to the code generator's backend kind. An
   id with no kind refuses the run before any negotiation, naming each one.
5. Negotiate with the backend providers, then route only the items settled as
   supported.
6. Ask the code generator to generate for each routed item and return its
   settlements, claim map and generated artifacts.

Kani execution of the generated proofs is not wired up, and the generated
oracle crate is returned but not yet written to disk. The registry of
backends is a value the caller builds from provider manifests and passes in. It is
not global state.

`execute` is the smaller entry point behind `quire run`: it compiles one
source and calls one function on the selected engine, returning QSL's typed
outcome unchanged.

## Crates

| Crate | Role |
| --- | --- |
| `quire-driver` (root) | The library: `drive`, `execute`, the registry, limits and the cache interface. Depends on QSL and the code generator, and on neither the cache nor the CLI. |
| `quire-cli` | The `quire` binary: argument parsing and the command handlers. Depends on the driver and the cache. |
| `quire-plugin-host` | The process boundary for out-of-process backend providers: framing, wire types and lifecycle for the `quire.plugin-wire/v1` protocol. |
| `quire-cache` | A content-addressed on-disk cache that implements the driver's cache interface. The CLI does not call it yet. |

The driver defines the cache interface and the cache implements it, so the
library never depends on its own cache. `tests/dependency_edges.rs` fails if
the driver gains a dependency on the cache or on the QSL root crate.

## Ecosystem

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-dark.svg">
  <img alt="The Quire repositories. The main chain runs from the standard, quire-specification, to the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and quire-contract-runtime. quire-spec-language also feeds quire-protocol, and quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and quire-verification plans verification. Below it, shared libraries used throughout: quire-exact, quire-semantic-value, quire-walk, quire-canonical and quire-verification-contracts." src="docs/images/ecosystem-light.svg">
</picture>

quire-driver sits above the main chain and runs it. It
depends on the QSL layer crates from
[quire-spec-language](https://github.com/agent-ix/quire-spec-language), on
[quire-contract-codegen](https://github.com/agent-ix/quire-contract-codegen),
on the model crate of
[quire-contract-ir](https://github.com/agent-ix/quire-contract-ir), and on
[quire-canonical](https://github.com/agent-ix/quire-canonical) and
[quire-exact](https://github.com/agent-ix/quire-exact). No QSL, IR or
code-generator crate depends on it. It also uses `ix-cli-kit` for the
command-line exit and stream conventions.

## Building and testing

Requires a Rust toolchain at least the `rust-version` in `Cargo.toml`.

```
make fmt-check   # formatting
make lint        # clippy, warnings are errors
make test        # cargo test
make deny        # license and source policy
make spec        # quire validate over spec/
make ci          # every gate above plus unsafe-comment and doc checks
```

`make build` makes a release build. Two `quire-cli` signal tests need normal,
unsandboxed process-signal permissions; rerun them outside a sandbox if they
fail there. Requirements live in [`spec/`](spec/spec.md); `CLAUDE.md` has the
contributor notes and the crate layout.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) and
[CONTENT_RIGHTS.md](CONTENT_RIGHTS.md).
