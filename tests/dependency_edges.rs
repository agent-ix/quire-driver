// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001-AC-1 (TC-005): the driver's dependency edges match ADR-011 §6.1's
//! driver row and §7.1's crate graph, read from `cargo metadata`'s resolved
//! graph. QSL packages are identified by their source URL, and normal and
//! build edges both count.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

/// The source of every package built from the QSL repository.
const QSL_SOURCE: &str = "git+https://github.com/agent-ix/quire-spec-language";
/// The QSL root crate, which is not a layer crate (ADR-011 §6.1 crate map).
const QSL_ROOT: &str = "quire-spec-language";

struct Package {
    name: String,
    source: Option<String>,
}

impl Package {
    fn is_qsl(&self) -> bool {
        self.source
            .as_deref()
            .is_some_and(|source| source.starts_with(QSL_SOURCE))
    }
}

struct Graph {
    packages: BTreeMap<String, Package>,
    /// Package id -> the package ids it reaches by a normal or build edge.
    deps: BTreeMap<String, Vec<String>>,
    root: String,
}

impl Graph {
    fn load() -> Self {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
        let output = Command::new(cargo)
            .args(["metadata", "--format-version", "1", "--locked", "--offline"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("cargo metadata runs");
        assert!(
            output.status.success(),
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let metadata: Value = serde_json::from_slice(&output.stdout).expect("metadata is JSON");
        let packages = metadata["packages"]
            .as_array()
            .expect("packages")
            .iter()
            .map(|p| {
                (
                    p["id"].as_str().expect("id").to_owned(),
                    Package {
                        name: p["name"].as_str().expect("name").to_owned(),
                        source: p["source"].as_str().map(str::to_owned),
                    },
                )
            })
            .collect();
        let deps = metadata["resolve"]["nodes"]
            .as_array()
            .expect("resolve nodes")
            .iter()
            .map(|node| {
                let edges = node["deps"]
                    .as_array()
                    .expect("deps")
                    .iter()
                    .filter(|dep| {
                        dep["dep_kinds"]
                            .as_array()
                            .expect("dep_kinds")
                            .iter()
                            .any(|k| k["kind"].is_null() || k["kind"] == "build")
                    })
                    .map(|dep| dep["pkg"].as_str().expect("pkg").to_owned())
                    .collect();
                (node["id"].as_str().expect("node id").to_owned(), edges)
            })
            .collect();
        let root = metadata["resolve"]["root"]
            .as_str()
            .expect("root")
            .to_owned();
        Self {
            packages,
            deps,
            root,
        }
    }

    fn package(&self, id: &str) -> &Package {
        &self.packages[id]
    }

    fn deps_of(&self, id: &str) -> impl Iterator<Item = &Package> {
        self.deps[id].iter().map(|dep| self.package(dep))
    }

    fn ids_named(&self, name: &str) -> Vec<&str> {
        self.packages
            .iter()
            .filter(|(_, p)| p.name == name)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

/// FR-001-AC-1 (TC-005)
#[test]
fn tc_005_driver_edges_match_the_adr_011_allow_list() {
    let graph = Graph::load();
    assert_eq!(graph.package(&graph.root).name, "quire-driver");

    // §6.1 driver row: the driver's normal and build edges are QSL layer
    // crates, CG, IR's model crate (E5's reader and `CheckedNodeId`) and one
    // error-derive crate. Never the QSL root crate. `serde_json` is a
    // dev-dependency (this crate's own tests use it), so it is not one of
    // these normal/build edges.
    let direct: BTreeSet<&str> = graph
        .deps_of(&graph.root)
        .map(|p| p.name.as_str())
        .collect();
    let expected: BTreeSet<&str> = [
        "qsl-foundation",
        "qsl-package",
        "qsl-replay",
        "qsl-route",
        "qsl-semantics",
        "quire-contract-codegen",
        "quire-contract-model",
        "thiserror",
    ]
    .into_iter()
    .collect();
    assert_eq!(direct, expected);
    let driver_qsl: Vec<&Package> = graph.deps_of(&graph.root).filter(|p| p.is_qsl()).collect();
    assert_eq!(driver_qsl.len(), 5, "five QSL edges, by source");
    assert!(driver_qsl.iter().all(|p| p.name != QSL_ROOT));

    // Nothing in this lock depends on the driver.
    assert_eq!(graph.ids_named("quire-driver").len(), 1);
    for id in graph.deps.keys() {
        assert!(
            graph.deps_of(id).all(|p| p.name != "quire-driver"),
            "`{}` depends on quire-driver",
            graph.package(id).name
        );
    }

    // §7.1 / FB-05: CG's one normal or build edge into QSL is `qsl-replay`.
    let cg = graph.ids_named("quire-contract-codegen");
    assert_eq!(cg.len(), 1, "one CG revision");
    let cg_qsl: Vec<&Package> = graph.deps_of(cg[0]).filter(|p| p.is_qsl()).collect();
    let cg_qsl_names: Vec<&str> = cg_qsl.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(cg_qsl_names, ["qsl-replay"]);

    // The driver's model crate is the one CG's IR crate re-exports, so a
    // `CheckedPackageV2` the driver reads is the type CG generates from.
    let driver_model: Vec<&String> = graph.deps[&graph.root]
        .iter()
        .filter(|id| graph.package(id).name == "quire-contract-model")
        .collect();
    assert_eq!(driver_model.len(), 1);
    let cg_ir: Vec<&String> = graph.deps[cg[0]]
        .iter()
        .filter(|id| graph.package(id).name == "quire-contract-ir")
        .collect();
    assert_eq!(cg_ir.len(), 1, "CG has one IR edge");
    assert!(
        graph.deps[cg_ir[0]].contains(driver_model[0]),
        "the driver's `quire-contract-model` ({}) is not CG's IR's",
        driver_model[0]
    );

    // The driver's own QSL layer crates (its direct edges, at the revision
    // Cargo.toml's revision note names) now resolve the same
    // `quire-contract-model` package id as the driver's own direct pin:
    // `qsl-package`'s one model edge is identical to `driver_model[0]`, not
    // just same-revision. QSL-255b PR 2 (this bump) is what removes the
    // second, divergent copy QSL's layer crates used to bring.
    let driver_qsl_package: Vec<&String> = graph.deps[&graph.root]
        .iter()
        .filter(|id| graph.package(id).name == "qsl-package")
        .collect();
    assert_eq!(
        driver_qsl_package.len(),
        1,
        "one qsl-package edge from the driver's own QSL pin"
    );
    let qsl_model: Vec<&String> = graph.deps[driver_qsl_package[0]]
        .iter()
        .filter(|id| graph.package(id).name == "quire-contract-model")
        .collect();
    assert_eq!(qsl_model.len(), 1, "qsl-package has one model edge");
    assert_eq!(
        qsl_model[0], driver_model[0],
        "the driver's own QSL pin and its `quire-contract-model` pin diverge: {} vs {}",
        qsl_model[0], driver_model[0]
    );

    // Two more copies remain, both outside this bump's control:
    //
    // - CG's own `qsl-replay` pin (Cargo.toml's revision note) has not
    //   caught up to the driver's QSL revision, so CG's `qsl-replay` still
    //   resolves its own, older `qsl-package`, which still brings a
    //   `quire-contract-model` copy at the older revision that older
    //   `qsl-package` pinned. That is CG's own pin to move, not this
    //   crate's; it is informational, not gated here, the same way the
    //   revision string a QSL crate resolves to has never been gated
    //   independent of the package-id identity checks above.
    // - `quire-contract-ir` (CG's IR edge) depends on the QSL root crate
    //   `quire-spec-language` at a revision of its own, which pins its own
    //   `quire-contract-model` copy. That copy stays until IR's root crate
    //   drops its QSL edge (agent-ix/quire-contract-ir#141).
    //
    // Whether an IR type crosses between any of these copies is a
    // compile-time question this crate's own build already answers, not one
    // `cargo metadata` can.
}
