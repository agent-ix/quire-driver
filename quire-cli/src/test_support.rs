// SPDX-License-Identifier: AGPL-3.0-or-later
//! Helpers for tests that must exercise the emitted-package path.

use std::collections::BTreeMap;

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::SpineLimits;
use quire_driver::{build_registry, drive, CompileInput, GenerationInput, QslCg};

pub fn emitted_package() -> qsl_package::EmittedPackage {
    emitted_package_with_source_name("process-provider-test")
}

/// A real emitted package whose bytes exercise URL-safe base64 substitutions.
pub fn emitted_package_with_urlsafe_bytes() -> qsl_package::EmittedPackage {
    // Five tildes ensure the serialized source identity contains an aligned
    // standard-base64 `/` and make the resulting byte length non-divisible by 3.
    emitted_package_with_source_name("process-provider-~~~~~")
}

fn emitted_package_with_source_name(source_name: &str) -> qsl_package::EmittedPackage {
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\nfunction f using v(): Boolean pure {{ true }}\n",
        "quire.value.complete/v1"
    );
    let packages = BTreeMap::new();
    let input = CompileInput {
        source: SourceIdentity::new("agent-ix", source_name),
        package_name: "unit-test",
        path: "unit-test.native",
        bytes: source.as_bytes(),
        packages: &packages,
        limits: SpineLimits::default(),
        cancel: quire_exact::Cancel::new(),
    };
    drive(
        &QslCg,
        input,
        &build_registry(&[]).expect("empty linked registry is admitted"),
        None,
        GenerationInput::default(),
    )
    .expect("fixture compiles and emits a real package")
    .emitted()
    .clone()
}
