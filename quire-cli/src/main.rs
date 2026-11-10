// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `quire` executable.

#[allow(
    dead_code,
    reason = "FR-331 request members await the remaining QSL projection support"
)]
mod backend_provider;
mod cli;
#[allow(
    dead_code,
    reason = "typed process results remain unmapped pending QSL settlement APIs"
)]
mod process_provider;
mod provider_config;
mod signals;
#[cfg(test)]
mod test_support;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}
