# =============================================================================
# quire-driver Makefile
# =============================================================================

CARGO ?= cargo
RUSTUP ?= rustup
NPM ?= npm

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make tools            - Install/check rustfmt, clippy, cargo-deny, and quire"
	@echo "  make deny             - cargo deny check licenses sources"
	@echo "  make spec             - quire validate over spec/"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make docs             - cargo doc, denying missing/broken doc links"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + audit-unsafe + docs + spec)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: test
test:
	$(CARGO) test

.PHONY: build
build:
	$(CARGO) build --release

# Install the host tools used by the repository's Make targets. rustup, Cargo,
# npm, and make themselves must already be available; this target bootstraps the
# project toolchain, cargo-deny, and Quire on macOS and Linux.
.PHONY: tools
tools:
	@set -eu; \
	case "$$(uname -s)" in Darwin|Linux) ;; *) echo "make tools supports macOS and Linux" >&2; exit 1 ;; esac; \
	for tool in "$(RUSTUP)" "$(CARGO)" bash; do \
		if ! command -v "$$tool" >/dev/null 2>&1; then \
			echo "make tools requires $$tool on PATH; install rustup and bash first" >&2; exit 1; \
		fi; \
	done; \
	toolchain="$$(sed -n 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' rust-toolchain.toml | head -n 1)"; \
	if [ -z "$$toolchain" ]; then echo "could not read channel from rust-toolchain.toml" >&2; exit 1; fi; \
	$(RUSTUP) toolchain install "$$toolchain" --component rustfmt --component clippy; \
	if ! $(CARGO) deny --version >/dev/null 2>&1; then \
		$(CARGO) install --locked cargo-deny; \
	fi; \
	if ! command -v quire >/dev/null 2>&1; then \
		if ! command -v "$(NPM)" >/dev/null 2>&1; then \
			echo "make tools needs npm on PATH to install @agent-ix/quire-cli" >&2; exit 1; \
		fi; \
		$(NPM) install --global @agent-ix/quire-cli; \
	fi; \
	$(CARGO) deny --version; \
	if ! command -v quire >/dev/null 2>&1; then \
		echo "quire was installed but its npm global bin directory is not on PATH" >&2; exit 1; \
	fi; \
	quire --version

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses sources

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

# =============================================================================
# Documentation
# =============================================================================

# cargo doc is a separate lint pass from clippy: rustdoc-only lints such as
# rustdoc::broken_intra_doc_links only fire here, never under `make lint`.
.PHONY: docs
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features

# =============================================================================
# Specification
# =============================================================================

.PHONY: spec
spec:
	quire validate 'spec/**/*.md'

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test deny audit-unsafe docs spec
