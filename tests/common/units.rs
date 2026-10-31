// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared native source fixtures for the driver and CLI integration tests.

/// Build a source unit with QSL's pinned root profile and the supplied body.
pub fn unit(body: &str) -> String {
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"quire.value.complete/v1\";\n{body}"
    )
}

/// Return the exact check-stage result-bound refusal fixture used by TC-006.
pub fn overflow_unit() -> String {
    unit("function unreachable_result using v(x: Int[0, 9]): Int[0, 10] pure { x + 100 }\n")
}
