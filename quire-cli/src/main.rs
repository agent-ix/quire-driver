// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entry point for the user-facing `quire` executable.

mod cli;

fn main() -> std::process::ExitCode {
    cli::entrypoint()
}
