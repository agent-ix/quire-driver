// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `quire` executable.

mod cli;
#[allow(
    dead_code,
    reason = "process routing awaits the CG IR-629 process BackendKind mapping"
)]
mod process_provider;
mod signals;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}
