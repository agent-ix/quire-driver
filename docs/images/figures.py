# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-driver's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The solid and dashed
boxes in the hero follow the verbs the `quire` binary handles today
(quire-cli/src/cli.rs) and the steps `drive` runs (src/drive.rs).
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: from source to verdict, what runs today and what is planned."""
    f = Figure(
        "hero",
        980,
        470,
        "The quire command takes QSL source through parse, check and compile to a checked package, "
        "and its run verb evaluates a function to an outcome with an exit code, all working today. "
        "The driver library also reads the package back, routes it, negotiates with backends and "
        "generates code, but no command exposes that yet. The prove and generate commands and "
        "their verdict are planned.",
    )
    top = 24
    f.text(0, 12, "QUIRE COMMAND · WORKS TODAY", "muted", "mono", SMALL)
    f.box(0, top, 140, 56, "code", "QSL source", ".native file")
    f.box(180, top, 110, 56, "logic", "parse", "quire parse")
    f.box(330, top, 110, 56, "logic", "check", "quire check")
    f.box(480, top, 130, 56, "logic", "compile", "quire compile")
    f.box(650, top, 190, 56, "model", "checked package", "checked-package/v2")
    for a, b in ((140, 177), (290, 327), (440, 477), (610, 647)):
        f.line([(a, top + 28), (b, top + 28)], "logic")

    run = 140
    f.box(180, run, 130, 56, "logic", "run", "interpreter")
    f.box(360, run, 220, 56, "evidence", "outcome", "result · exit code")
    f.line([(70, top + 56), (70, run + 28), (177, run + 28)], "code")
    f.line([(310, run + 28), (357, run + 28)], "logic")

    lib = 262
    f.text(0, lib - 12, "DRIVER LIBRARY · TESTED, NO COMMAND YET", "muted", "mono", SMALL)
    f.line([(745, top + 56), (745, lib - 20)], "model", arrow=False)
    f.line([(370, lib - 20), (915, lib - 20)], "model", arrow=False)
    names = [
        ("read back", "IR admission"),
        ("route", "candidates"),
        ("negotiate", "backends"),
        ("generate", "code · claims"),
    ]
    xs = [370, 530, 690, 850]
    for x, (name, sub) in zip(xs, names):
        f.line([(x + 65, lib - 20), (x + 65, lib - 3)], "model")
        f.box(x, lib, 130, 56, "logic", name, sub)
    for x in xs[:3]:
        f.line([(x + 130, lib + 28), (x + 160, lib + 28)], "logic")

    plan = 380
    f.text(0, plan - 12, "PLANNED", "muted", "mono", SMALL)
    f.box(780, plan, 200, 56, "logic", "quire prove · generate", "command verbs", dashed=True)
    f.line([(915, lib + 56), (915, plan - 3)], "logic", dashed=True)
    f.box(480, plan, 240, 56, "evidence", "verdict", "evidence from a backend", dashed=True)
    f.line([(780, plan + 28), (723, plan + 28)], "evidence", dashed=True)
    f.legend(0, 464, [("logic", "works today", "box"), ("logic", "planned", "dash")])
    return f


REPOS = [
    (1, 0, "quire-driver", "the quire command", True),
    (2, 0, "quire-integration", "tests the whole chain", True),
    (3, 0, "quire-verification", "plans verification", True),
    (0, 1, "quire-specification", "the standard", True),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", True),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and "
        "quire-verification plans verification. Below it, shared libraries used throughout: "
        "quire-exact, quire-semantic-value, quire-walk, quire-canonical and "
        "quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("ink", "private", "dash"),
                      ("model", "feeds", "fill")])
    return f


def ecosystem_here():
    """The ecosystem figure with quire-driver highlighted."""
    return ecosystem(here="quire-driver")


FIGURES = [hero, ecosystem_here]
