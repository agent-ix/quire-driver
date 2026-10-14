// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `quire` executable.

#[allow(
    dead_code,
    reason = "FR-331 request construction awaits routed Process items"
)]
mod backend_provider;
mod cli;
#[allow(
    dead_code,
    reason = "process routing awaits the CG IR-629 process BackendKind mapping"
)]
mod process_provider;
mod signals;
#[cfg(test)]
mod test_support;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}
