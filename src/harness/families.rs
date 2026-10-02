//! The six harness decision-point families (Issue 004, from Research 579 —
//! the "Jev Engineering for Coding Agents" six-per-turn-question map):
//! `harness_visibility` · `harness_permissions` · `harness_tool_fit` ·
//! `harness_routing` · `harness_sensitivity` · `harness_cache_reuse`
//! (all six modelless DEFAULT — Issue 045 lifted the LLM-only carve-out:
//! the family is text-decidable like the other five, and the LLM lane's
//! published 0.5000 row had no winner to protect).
//!
//! Pure data + builders (ungated), the `code_fixtures` shape: in-process
//! synthetic fixtures with programmatic gold, self-split into three
//! PAIRWISE-DISJOINT slices — corpus (the routing/reference texts), cal
//! (the calibration slice, ≥16 so the fused-gate thresholds fit), eval —
//! because a cal case scoring cos 1.0 against ITSELF inflates every
//! cal-slice quantile (the measured self-inclusion leak, runner docs).
//!
//! Fixture-design law (the degenerate-lane lesson, 2026-09-22): the first
//! draft's one-line states were swamped by the shared per-family prompt in
//! the hashed-bag context, routing collapsed to one domain per family, and
//! 4 of 5 families emitted a CONSTANT pick. The states below are multi-
//! sentence and class-distinctive by VOCABULARY — and they deliberately do
//! NOT contain the gold label token (no "— hide.", ": allow.", "level 4"),
//! which would make any future accuracy claim a label-leak claim. The
//! corpus docs DO carry the label words: they are what teaches the
//! corpus-conditioned drafter which option string fits.
//!
//! No engine imports here; the runner consumes [`SynthData`].

use serde_json::{Map, Value};

use super::families_eval_wide;
use crate::harness::suites::{GoldAnswer, QKind, Suite, SuiteCase, SuiteQuestion, TrainDoc};

/// One in-process suite, prepared for the runner: the eval suite, its
/// calibration slice, the per-label corpus docs, and the domain labels
/// (domain order = label order = option-key order for Choice).
#[derive(Debug, Clone)]
pub struct SynthData {
    pub suite: Suite,
    pub cal_cases: Vec<SuiteCase>,
    pub docs: Vec<TrainDoc>,
    pub labels: Vec<String>,
}

/// One authored fixture: the state text and its programmatic gold index
/// (the option/level index, in `labels` order).
#[derive(Debug)]
pub struct FamilyText {
    pub text: &'static str,
    pub gold: usize,
}

/// One family definition: question shape, option universe, and the three
/// authored slices.
pub struct FamilyDef {
    pub name: &'static str,
    pub qid: &'static str,
    pub kind: QKind,
    /// Domain labels = option keys (Choice) = level indices (Score).
    pub labels: &'static [&'static str],
    pub instructions: &'static str,
    /// Score-level display strings (Score families; empty for Choice).
    pub score_levels: &'static [&'static str],
    pub note: &'static str,
    /// (gold index, text) — the routing/reference corpus.
    pub corpus: &'static [(usize, &'static str)],
    pub cal: &'static [FamilyText],
    pub eval: &'static [FamilyText],
}

// ── visibility (T2): the query-aware chunk visibility ladder ───────────────

const VIS_INSTR: &str = "How visible should this context chunk be for the current query? \
hide = irrelevant to the question; short = one-line summary; long = detailed \
summary; full = verbatim.";

static VIS: FamilyDef = FamilyDef {
    name: "harness_visibility",
    qid: "visibility",
    kind: QKind::Choice,
    labels: &["hide", "short", "long", "full"],
    instructions: VIS_INSTR,
    score_levels: &[],
    note: "authored chunk fixtures; gold = the visibility level by construction (Issue 004 T2); states are label-token-free",
    corpus: &[
        (
            0,
            "Chunks irrelevant to the question get hide. An unrelated changelog entry from another project has no bearing on the current query, so hide costs nothing and saves the window.",
        ),
        (
            0,
            "Plans superseded weeks ago belong in hide. Nothing in a dead plan answers the live question, and the window it frees is worth more than the text.",
        ),
        (
            0,
            "Build artifact listings and lockfile dumps are hide material. They are noise for any query about behavior, history, or design.",
        ),
        (
            1,
            "Tangential hits compress to a short one-line summary. Twelve grep matches reduce to a headline naming the two plausible files.",
        ),
        (
            1,
            "Dependency version bumps need only a short note. The version number is the whole story; everything else is boilerplate.",
        ),
        (
            1,
            "Directory listings compress to a short line naming the folder that matters. The rest of the tree adds tokens, not information.",
        ),
        (
            2,
            "Diagnostic material earns a long detailed summary. Keep the stack trace, the failing test name, and the probable cause; drop only the repetition.",
        ),
        (
            2,
            "A function body suspected in the regression deserves a long excerpt. The suspicious branch stays intact with its surrounding match arms.",
        ),
        (
            2,
            "Config files under investigation show long. Every flag stays visible until the bug is found, then the survivor compresses.",
        ),
        (
            3,
            "The edited function at the center of the task shows full and verbatim. Every line the change touches stays in the window exactly as written.",
        ),
        (
            3,
            "The user-reported error message is the anchor of the task and shows full. It is never paraphrased, because the exact wording is the evidence.",
        ),
        (
            3,
            "The API contract the patch must preserve reads full. Signature, docs, and guarantees stay complete for the whole session.",
        ),
    ],
    cal: &[
        FamilyText {
            text: "A vendored fork's internal release notes, describing changes this repository never picked up. Nothing here touches the task.",
            gold: 0,
        },
        FamilyText {
            text: "Yesterday's abandoned scratch experiment, kept only until the branch deletes. Its approach was rejected before review.",
            gold: 0,
        },
        FamilyText {
            text: "Continuous-integration logs from an unrelated pipeline, green runs for someone else's service.",
            gold: 0,
        },
        FamilyText {
            text: "The retired authentication flow that the current code replaced last quarter. No live path references it.",
            gold: 0,
        },
        FamilyText {
            text: "A duplicated paragraph already present elsewhere in the window. The copy adds zero new facts.",
            gold: 0,
        },
        FamilyText {
            text: "A dependency audit table spanning forty rows, every row the same verdict. The whole table reduces to one compliant line.",
            gold: 1,
        },
        FamilyText {
            text: "Twenty search hits across the tree, three of them plausible. The rest share one import line.",
            gold: 1,
        },
        FamilyText {
            text: "Release notes for a tool this repository merely transitively depends on, no API touched.",
            gold: 1,
        },
        FamilyText {
            text: "The folder inventory of a docs tree, where a single subdirectory carries everything the task needs.",
            gold: 1,
        },
        FamilyText {
            text: "Benchmark history across ten versions, flat since the third. The delta since then is one number.",
            gold: 1,
        },
        FamilyText {
            text: "The failing integration test's output: setup, the assertion that broke, and the fixture path that seeded it.",
            gold: 2,
        },
        FamilyText {
            text: "The parser branch now suspected of the mis-compile, with its guard conditions and the fall-through it takes.",
            gold: 2,
        },
        FamilyText {
            text: "The migration diff while review is open, every hunk annotated with the reviewer's open questions.",
            gold: 2,
        },
        FamilyText {
            text: "Cache eviction logs from the window when the regression reproduces, rates and sizes intact.",
            gold: 2,
        },
        FamilyText {
            text: "The protocol handshake capture, each frame annotated with the field it decodes to.",
            gold: 2,
        },
        FamilyText {
            text: "The function this patch rewrites, every line the diff touches, exactly as the working tree holds it.",
            gold: 3,
        },
        FamilyText {
            text: "The panic message the user pasted, word for word — the exact wording is what the issue-tracker search needs.",
            gold: 3,
        },
        FamilyText {
            text: "The schema contract the migration must satisfy, every field and its guarantee.",
            gold: 3,
        },
        FamilyText {
            text: "The negotiated wire-format table, byte offsets and all, as the two sides signed it.",
            gold: 3,
        },
        FamilyText {
            text: "The security policy the change implements, clauses intact, with no summarization permitted.",
            gold: 3,
        },
    ],
    eval: &families_eval_wide::VIS_WIDE_EVAL,
};

// ── permissions (T4): allow / ask / deny over commands ─────────────────────

const PERM_INSTR: &str = "Should this command run without prompting? allow = safe \
read-only or the expected build/test invocation; ask = writes or side effects \
outside the obvious scope; deny = touches secrets, credentials, or destructive \
paths.";

static PERM: FamilyDef = FamilyDef {
    name: "harness_permissions",
    qid: "permission",
    kind: QKind::Choice,
    labels: &["allow", "ask", "deny"],
    instructions: PERM_INSTR,
    score_levels: &[],
    note: "authored command fixtures; gold = the policy verdict by construction (Issue 004 T4); states are label-token-free",
    corpus: &[
        (
            0,
            "Read-only inspection is pre-approved: allow git status, grep over the sources, and cat of tracked files.",
        ),
        (
            0,
            "The expected build and test invocations are allow-by-default — cargo test and cargo check in the working tree.",
        ),
        (
            0,
            "Any command that only reads the tree and writes nothing outside target/ is allow.",
        ),
        (
            1,
            "Side effects beyond the obvious scope ask first: global installs, publishes, deploys, and anything reaching an external host.",
        ),
        (
            1,
            "Writes outside the repository ask: config in the user's home, registry pushes, and network egress need an explicit yes.",
        ),
        (
            1,
            "Shared-state mutation is ask territory — rotating credentials in staging, bumping published versions.",
        ),
        (
            2,
            "Deny anything touching secrets: printing env files, reading private keys, echoing tokens into logs.",
        ),
        (
            2,
            "Destructive paths are denied: recursive force-removal across checkouts, history rewrites on shared branches, volume wipes.",
        ),
        (
            2,
            "Credential exfiltration patterns deny, always — a private signing key never reaches command output.",
        ),
    ],
    cal: &[
        FamilyText {
            text: "git diff on the current branch, reviewing what the session changed.",
            gold: 0,
        },
        FamilyText {
            text: "cargo test --lib in this workspace, the expected inner loop.",
            gold: 0,
        },
        FamilyText {
            text: "grep the sources for the symbol the error message names.",
            gold: 0,
        },
        FamilyText {
            text: "cat the README section the task references.",
            gold: 0,
        },
        FamilyText {
            text: "List the target directory to see what the last build left.",
            gold: 0,
        },
        FamilyText {
            text: "Run the linter over the files the session edited.",
            gold: 0,
        },
        FamilyText {
            text: "Push a new branch to the shared remote for the first time.",
            gold: 1,
        },
        FamilyText {
            text: "Install a global cargo subcommand on this machine.",
            gold: 1,
        },
        FamilyText {
            text: "Fetch a URL from an external host the repo has never talked to.",
            gold: 1,
        },
        FamilyText {
            text: "Rewrite the deployment config that lives outside the repository.",
            gold: 1,
        },
        FamilyText {
            text: "Bump the version and publish the crate to the public registry.",
            gold: 1,
        },
        FamilyText {
            text: "Rotate the staging database password during business hours.",
            gold: 1,
        },
        FamilyText {
            text: "Print the environment file holding the live API keys.",
            gold: 2,
        },
        FamilyText {
            text: "cat the private SSH identity used for signing.",
            gold: 2,
        },
        FamilyText {
            text: "Recursive-force-remove the sibling checkout of another agent.",
            gold: 2,
        },
        FamilyText {
            text: "Force-push a rewritten history onto the shared develop branch.",
            gold: 2,
        },
        FamilyText {
            text: "Echo the bearer token into the build log for debugging.",
            gold: 2,
        },
        FamilyText {
            text: "Wipe the production database volume to free space.",
            gold: 2,
        },
    ],
    eval: &families_eval_wide::PERM_WIDE_EVAL,
};

// ── tool_fit (T4): which single tool fits the intent ───────────────────────

const TOOL_INSTR: &str = "Which single tool best fits this intent?";

static TOOL: FamilyDef = FamilyDef {
    name: "harness_tool_fit",
    qid: "tool_fit",
    kind: QKind::Choice,
    labels: &[
        "grep",
        "ast_grep",
        "git_log",
        "cargo_test",
        "cargo_fmt",
        "docs_search",
    ],
    instructions: TOOL_INSTR,
    score_levels: &[],
    note: "authored intent fixtures over a six-tool tier-1 snippet universe; gold by construction (Issue 004 T4)",
    corpus: &[
        (
            0,
            "grep: plain-text pattern search over files and directories. Use grep to find every literal TODO comment across the sources.",
        ),
        (
            0,
            "grep answers find-this-string-everywhere intents over the working tree.",
        ),
        (
            1,
            "ast_grep: structural search over the syntax tree. ast_grep locates every match expression that returns unit, by shape.",
        ),
        (
            1,
            "ast_grep answers find-this-syntax-shape intents that text search cannot express.",
        ),
        (
            2,
            "git_log: history over commits, authors, and blame. git_log finds the commit that introduced a line.",
        ),
        (
            2,
            "git_log answers when-did-this-change and who-wrote-this intents over history.",
        ),
        (
            3,
            "cargo_test: run the test suites and report results. cargo_test checks whether the parity gates still pass.",
        ),
        (
            3,
            "cargo_test answers is-it-still-green intents after an edit.",
        ),
        (
            4,
            "cargo_fmt: format the sources to the house style. cargo_fmt normalizes whitespace and wrapping before a commit.",
        ),
        (
            4,
            "cargo_fmt answers make-this-clean intents over formatting.",
        ),
        (
            5,
            "docs_search: search documentation, READMEs, and guides. docs_search finds where a contract is documented.",
        ),
        (
            5,
            "docs_search answers where-is-this-documented intents over prose.",
        ),
    ],
    cal: &[
        FamilyText {
            text: "Find all occurrences of the constant that the error message names.",
            gold: 0,
        },
        FamilyText {
            text: "Search the tree for every remaining use of the deprecated flag.",
            gold: 0,
        },
        FamilyText {
            text: "Locate the literal string that is leaking into the logs.",
            gold: 0,
        },
        FamilyText {
            text: "Find every function whose body is a single unit return, by syntax shape.",
            gold: 1,
        },
        FamilyText {
            text: "Locate all match arms that fall through to the same expression.",
            gold: 1,
        },
        FamilyText {
            text: "Find every call site that passes a raw pointer, structurally.",
            gold: 1,
        },
        FamilyText {
            text: "Which commit changed this function last, and by whom?",
            gold: 2,
        },
        FamilyText {
            text: "Find the author and date of the commit that broke the build.",
            gold: 2,
        },
        FamilyText {
            text: "Trace when this regression first entered the history.",
            gold: 2,
        },
        FamilyText {
            text: "Do the unit tests still pass after the refactor?",
            gold: 3,
        },
        FamilyText {
            text: "Run the suites and report which gates fail and by how much.",
            gold: 3,
        },
        FamilyText {
            text: "Verify the fix did not regress the count floors.",
            gold: 3,
        },
        FamilyText {
            text: "Normalize the formatting of the files the session edited.",
            gold: 4,
        },
        FamilyText {
            text: "Bring the new module to the house style before review.",
            gold: 4,
        },
        FamilyText {
            text: "Clean up the whitespace-only noise from the last patch.",
            gold: 4,
        },
        FamilyText {
            text: "Where is the calibration protocol documented?",
            gold: 5,
        },
        FamilyText {
            text: "Find the guide section that explains the arena tables.",
            gold: 5,
        },
        FamilyText {
            text: "Look up the runbook step for the release ceremony.",
            gold: 5,
        },
    ],
    eval: &families_eval_wide::TOOL_WIDE_EVAL,
};

// ── routing (T5): which execution lane runs the subtask ────────────────────

const ROUTE_INSTR: &str = "Which execution lane should this subtask run on? \
local_engine = microsecond deterministic decisions at volume, in-process; \
cheap_api = simple prose work, cost-sensitive; frontier_api = hard novel \
reasoning; background_batch = deferred bulk work.";

static ROUTE: FamilyDef = FamilyDef {
    name: "harness_routing",
    qid: "routing",
    kind: QKind::Choice,
    labels: &[
        "local_engine",
        "cheap_api",
        "frontier_api",
        "background_batch",
    ],
    instructions: ROUTE_INSTR,
    score_levels: &[],
    note: "authored subtask fixtures; gold = the lane by construction (Issue 004 T5); states are label-token-free",
    corpus: &[
        (
            0,
            "The local engine takes microsecond deterministic decisions at volume: fixed label sets, in-process, no network. Tick-rate routing and per-command verdicts live here.",
        ),
        (
            0,
            "The fix lane scores spans in-process on the local engine — microsecond tier, bit-identical repeats.",
        ),
        (
            0,
            "Thousands of classifications per hour with a fixed option universe belong on the local engine.",
        ),
        (
            1,
            "The cheap API takes simple prose work where cost matters: summarizing short changelog entries, tagging sentiment.",
        ),
        (
            1,
            "Structured extraction from support email at volume runs on the cheap API.",
        ),
        (
            1,
            "Backlogs of brief reviews get labeled on the cheap API — good enough, and nearly free.",
        ),
        (
            2,
            "The frontier API takes hard novel reasoning: the multi-region schema migration design, the cross-service heisenbug.",
        ),
        (
            2,
            "The security review of a new authentication flow needs the frontier model's depth.",
        ),
        (
            2,
            "When the bug reproduces only under load and the cause is unknown, the frontier API gets it.",
        ),
        (
            3,
            "Deferred bulk work runs as a background batch: regenerate the docs site overnight, re-embed the corpus after a swap.",
        ),
        (
            3,
            "Scheduled maintenance — compaction, archival, weekly digests — is background batch work.",
        ),
        (
            3,
            "The monthly usage report assembles itself as a background batch job.",
        ),
    ],
    cal: &[
        FamilyText {
            text: "Classify this ticket into one of eight queues; ten thousand arrive per hour.",
            gold: 0,
        },
        FamilyText {
            text: "Decide, in microseconds, whether this command may run.",
            gold: 0,
        },
        FamilyText {
            text: "Score these candidate spans for the fix lane, in-process, deterministically.",
            gold: 0,
        },
        FamilyText {
            text: "Route each incoming request's intent against the fixed option set, per tick.",
            gold: 0,
        },
        FamilyText {
            text: "Summarize the day's release notes; the prose is simple and the budget is tight.",
            gold: 1,
        },
        FamilyText {
            text: "Extract the invoice fields from a thousand emails this week.",
            gold: 1,
        },
        FamilyText {
            text: "Label the support threads by topic, at minimal cost per item.",
            gold: 1,
        },
        FamilyText {
            text: "Rewrite these changelog blurbs in plainer language.",
            gold: 1,
        },
        FamilyText {
            text: "Design the concurrency model for the new executor from scratch.",
            gold: 2,
        },
        FamilyText {
            text: "Root-cause the parity gate that fails only under load.",
            gold: 2,
        },
        FamilyText {
            text: "Write the threat model for the new credential flow.",
            gold: 2,
        },
        FamilyText {
            text: "Work out which algebraic identity the optimizer might be exploiting.",
            gold: 2,
        },
        FamilyText {
            text: "Rebuild the search index tonight, after the traffic window.",
            gold: 3,
        },
        FamilyText {
            text: "Generate the evaluation tables for every checkpoint overnight.",
            gold: 3,
        },
        FamilyText {
            text: "Archive the old runs and compact the stores on a schedule.",
            gold: 3,
        },
        FamilyText {
            text: "Assemble the monthly usage report from the ledgers.",
            gold: 3,
        },
    ],
    eval: &families_eval_wide::ROUTE_WIDE_EVAL,
};

// ── sensitivity (T5): file-sensitivity ordinal score ───────────────────────

const SENS_INSTR: &str = "How sensitive is this file? 0 = public; 1 = open-source \
dependency; 2 = application code; 3 = infrastructure config; 4 = secrets and \
credentials.";

static SENS: FamilyDef = FamilyDef {
    name: "harness_sensitivity",
    qid: "sensitivity",
    kind: QKind::Score,
    labels: &["0", "1", "2", "3", "4"],
    instructions: SENS_INSTR,
    score_levels: &[
        "0 public",
        "1 open dependency",
        "2 application code",
        "3 infra config",
        "4 secrets",
    ],
    note: "authored file fixtures; gold = the sensitivity level by construction (Issue 004 T5); states carry no level digits",
    corpus: &[
        (
            0,
            "README and LICENSE are public by definition — sensitivity level 0. Published docs and marketing pages sit here too.",
        ),
        (
            0,
            "The public install instructions ship in the open repository — level 0.",
        ),
        (
            0,
            "Conference slides and the FAQ page are public material, sensitivity 0.",
        ),
        (
            1,
            "Vendored upstream sources are an open dependency — sensitivity 1. A lock file pinning public registry versions sits at level 1.",
        ),
        (
            1,
            "A vendored MIT crate is open-source dependency code, level 1.",
        ),
        (
            1,
            "Upstream library headers are open dependency material, sensitivity 1.",
        ),
        (
            2,
            "The engine and serve modules are application code — sensitivity 2, proprietary source.",
        ),
        (2, "Hand-written service code under src rates level 2."),
        (
            2,
            "The internal calibration module is proprietary source, sensitivity 2.",
        ),
        (
            3,
            "The reverse-proxy config in deploy is infrastructure configuration — sensitivity 3.",
        ),
        (
            3,
            "The worker's deployment routes file and CI wiring are infra config, level 3.",
        ),
        (
            3,
            "Provisioning manifests for the cluster rate sensitivity 3.",
        ),
        (
            4,
            "The environment file holding live API keys is secrets — sensitivity 4.",
        ),
        (
            4,
            "A private signing key is a credential, level 4; it never leaves the machine.",
        ),
        (
            4,
            "Token files and webhook signing secrets rate sensitivity 4.",
        ),
    ],
    cal: &[
        FamilyText {
            text: "The announcement post and the FAQ answer everything a newcomer needs, published on the site.",
            gold: 0,
        },
        FamilyText {
            text: "The license text ships in the open repository for anyone to read.",
            gold: 0,
        },
        FamilyText {
            text: "Slides from the conference talk walk through the architecture for a general audience.",
            gold: 0,
        },
        FamilyText {
            text: "The public roadmap lists what already shipped.",
            gold: 0,
        },
        FamilyText {
            text: "The regex engine pinned in the lock file comes from the public registry.",
            gold: 1,
        },
        FamilyText {
            text: "A vendored Apache-licensed module, unmodified from its upstream.",
            gold: 1,
        },
        FamilyText {
            text: "Upstream headers copied verbatim from the library this crate binds.",
            gold: 1,
        },
        FamilyText {
            text: "The dependency's own changelog, fetched from its open repository.",
            gold: 1,
        },
        FamilyText {
            text: "The request router written for this service, the heart of the product.",
            gold: 2,
        },
        FamilyText {
            text: "The internal calibration module, tuned on proprietary fixtures.",
            gold: 2,
        },
        FamilyText {
            text: "Hand-written service logic that embodies the house methodology.",
            gold: 2,
        },
        FamilyText {
            text: "The arena's scoring implementation, our own design.",
            gold: 2,
        },
        FamilyText {
            text: "The reverse-proxy stanza that terminates TLS for the cluster.",
            gold: 3,
        },
        FamilyText {
            text: "Terraform describing the fleet's network topology.",
            gold: 3,
        },
        FamilyText {
            text: "The CI workflow's deployment wiring, with internal hostnames.",
            gold: 3,
        },
        FamilyText {
            text: "The manifest that maps worker routes to environments.",
            gold: 3,
        },
        FamilyText {
            text: "The file holding the live payment-provider API keys.",
            gold: 4,
        },
        FamilyText {
            text: "The signing identity used to authorize releases.",
            gold: 4,
        },
        FamilyText {
            text: "The webhook verification value that third parties check our callbacks against.",
            gold: 4,
        },
        FamilyText {
            text: "The session-store encryption key material.",
            gold: 4,
        },
    ],
    eval: &families_eval_wide::SENS_WIDE_EVAL,
};

// ── cache_reuse (Issue 045): modelless DEFAULT ───────────────────────────
// The T3 carve-out is REVERSED (Issue 045): the family is text-decidable
// ("does the described prefix still cover the described next turn"), the
// 12 eval fixtures keep their programmatic gold, and the family now ships
// the two slices the other five always had — the authored per-class corpus
// and the cal front. The lever that makes the answer live is the noul
// count-table polarity (issue 038's `nb_noul_domain`, cal-selected — noul
// never takes route terms by the issue-030 law, so the corpus reaches the
// decision through the NB tables, not the drafter).

pub const CACHE_REUSE_NAME: &str = "harness_cache_reuse";
const CACHE_REUSE_INSTR: &str = "Should the harness reuse the cached prefix, or \
rebuild the context? yes = reuse (the prefix still serves the coming turn); \
no = rebuild (the context no longer matches the task).";
const CACHE_REUSE_NOTE: &str = "authored prefix scenarios; gold = reuse-vs-rebuild by \
construction. Modelless since Issue 045 (the T3 carve-out reversed): the \
corpus + cal slices below teach the coverage-vs-divergence distinction; the \
12 eval fixtures and their gold are UNCHANGED from the T3 authoring. \
Plan 009 REVISED-2 (issue 059): this is the DOCUMENTED DIVERGENCE — the other \
five families went wide (families_eval_wide), this one keeps its frozen T3 \
record on purpose (its grounded-posture gate pins the 12-fixture history); \
the wide-eval gates exempt it by name.";

static CACHE_REUSE_EVAL: [FamilyText; 12] = [
    FamilyText {
        text: "The transcript holds the file read from turn 3; the user now asks about a symbol inside that same file. Reuse the cached prefix?",
        gold: 1,
    },
    FamilyText {
        text: "The assembled context carries the module under discussion; the follow-up question stays within it. Reuse?",
        gold: 1,
    },
    FamilyText {
        text: "The prefix contains the schema the user is iterating on; the next request refines a field of it. Reuse the cache?",
        gold: 1,
    },
    FamilyText {
        text: "The session context holds the failing test output; the user asks why the assertion failed. Reuse?",
        gold: 1,
    },
    FamilyText {
        text: "The conversation already carries the reviewed diff; the user requests one more pass over it. Reuse the prefix?",
        gold: 1,
    },
    FamilyText {
        text: "The loaded API docs cover the function in question; the user asks about one of its flags. Reuse?",
        gold: 1,
    },
    FamilyText {
        text: "The context holds a long-dead debug session; the new task moves to a different subsystem. Reuse the cache?",
        gold: 0,
    },
    FamilyText {
        text: "The prefix is filled with superseded plan text; the task pivoted an hour ago. Reuse?",
        gold: 0,
    },
    FamilyText {
        text: "The transcript contains an unrelated project's logs; the new request concerns this repo's engine. Reuse the cached prefix?",
        gold: 0,
    },
    FamilyText {
        text: "The context carries the old schema version; the migration just changed the layout wholesale. Reuse?",
        gold: 0,
    },
    FamilyText {
        text: "The window is full of resolved review comments; the next task writes new code in another module. Reuse the cache?",
        gold: 0,
    },
    FamilyText {
        text: "The assembled context mixes five finished tasks; the user starts a fresh green-field request. Reuse?",
        gold: 0,
    },
];

/// The LLM-era fixture set, exposed for the gate tests (the runner
/// consumes [`synth_cache_reuse`] instead). UNCHANGED from the T3
/// authoring — the non-goal law: gold and fixture text never move.
/// Plan 009 REVISED-2: the documented divergence — the other five
/// families' evals went wide; this one stays the frozen 12 on purpose.
#[must_use]
pub fn cache_reuse_eval() -> &'static [FamilyText] {
    &CACHE_REUSE_EVAL
}

/// The coverage class (gold 1 = yes/reuse): the coming turn's subject
/// lives INSIDE what the window already holds. Class-distinctive by
/// vocabulary per the fixture law; the docs do not carry the yes/no
/// question words (issue 030: those are question vocabulary, never label
/// tokens — and the noul drafter never reads option text anyway).
static CACHE_REUSE_CORPUS: [(usize, &str); 12] = [
    (
        1,
        "A prefix still serving the coming turn is the one to keep. The transcript \
         holds the file the user is asking about, and the next question reads a \
         symbol from that same file — nothing about the task has moved.",
    ),
    (
        1,
        "Keep the cached context while the work stays inside it. A schema the user \
         is iterating on covers the field-level refinement they now request; the \
         bytes are already in place.",
    ),
    (
        1,
        "The failing test output in the window is exactly what the follow-up question \
         is about, so the assembled context still serves the turn — dropping it \
         would throw away the evidence the answer needs.",
    ),
    (
        1,
        "An already-loaded diff supports another pass over the same code. The \
         reviewed lines are the lines being discussed; the context and the task \
         still point at each other.",
    ),
    (
        1,
        "Loaded API reference earns its window while the questions stay on the \
         documented surface. A user asking about one of the function's flags is \
         reading from the pages already present.",
    ),
    (
        1,
        "The test of whether a prefix serves is containment: the coming turn's \
         subject lives inside what the window already holds, so nothing new must \
         be fetched to answer it.",
    ),
    (
        0,
        "A window full of one subsystem's history cannot serve a task that has moved \
         to another. The stale transcript gets rebuilt from the sources the new \
         question actually reads.",
    ),
    (
        0,
        "Superseded material justifies a rebuild. The plan text in the context was \
         abandoned an hour ago; carrying it forward would anchor every answer to \
         a design the task no longer follows.",
    ),
    (
        0,
        "Context from an unrelated project poisons the turn it sits in. The new \
         request concerns this repository's engine, so the foreign logs are \
         dropped and the context rebuilt around the real subject.",
    ),
    (
        0,
        "When a migration changes the layout wholesale, the cached schema no longer \
         describes the world. The old version in the window is discarded so the \
         answer reflects the current shape.",
    ),
    (
        0,
        "Finished work does not serve a fresh task. The window of resolved review \
         comments gives way to a rebuild when the next request starts new code \
         in another module.",
    ),
    (
        0,
        "The test of whether a prefix serves is divergence: the coming turn's \
         subject sits outside what the window holds, so the context is torn down \
         and reassembled from the task's own sources.",
    ),
];

/// The calibration front (10 per class, ≥ 16 total — the runner law).
/// Statement-only scenarios in the family's third-person shape, pairwise
/// disjoint from the corpus and the eval texts (the self-inclusion law).
static CACHE_REUSE_CAL: [FamilyText; 20] = [
    FamilyText {
        text: "The window holds the parser module the user has been editing; the \
              next question asks why one of its branches mis-scores a boundary case.",
        gold: 1,
    },
    FamilyText {
        text: "The assembled context carries the deployment manifest under \
              discussion; the follow-up changes one of its replica counts.",
        gold: 1,
    },
    FamilyText {
        text: "The prefix contains the crash report the session opened with; the \
              user now asks which allocation site it points at.",
        gold: 1,
    },
    FamilyText {
        text: "The session context holds the benchmark table being discussed; the \
              next request asks for the worst row's configuration.",
        gold: 1,
    },
    FamilyText {
        text: "The transcript holds the design doc the user summarized; the \
              follow-up quotes one of its paragraphs back for clarification.",
        gold: 1,
    },
    FamilyText {
        text: "The loaded trace covers the failing request end to end; the user \
              asks what the second hop returned.",
        gold: 1,
    },
    FamilyText {
        text: "The context carries the type's definition and its doc comment; the \
              question concerns one of the documented invariants.",
        gold: 1,
    },
    FamilyText {
        text: "The window holds the migration script mid-review; the next comment \
              asks about the rollback leg of that same script.",
        gold: 1,
    },
    FamilyText {
        text: "The prefix holds the search results the user requested; the \
              follow-up narrows them to the two hits in the auth module.",
        gold: 1,
    },
    FamilyText {
        text: "The assembled context contains the customer's reproduced bug; the \
              user now asks for the smallest patch that dismisses it.",
        gold: 1,
    },
    FamilyText {
        text: "The window is still full of yesterday's load-generator session; \
              today's task is a documentation rewrite in an untouched directory.",
        gold: 0,
    },
    FamilyText {
        text: "The prefix carries the first prototype's interface notes; the ABI \
              was redesigned from scratch since, and the task now concerns the \
              new one.",
        gold: 0,
    },
    FamilyText {
        text: "The context holds another team's incident postmortem; the user's \
              next request is about this repository's release checklist.",
        gold: 0,
    },
    FamilyText {
        text: "The transcript contains the shopping-cart service's logs; the \
              question that follows targets the auth service, which shares no \
              code with it.",
        gold: 0,
    },
    FamilyText {
        text: "The prefix is filled with the winter roadmap; priorities changed \
              last week and the task now executes the spring plan.",
        gold: 0,
    },
    FamilyText {
        text: "The window holds the retired parser's grammar; the language was \
              replaced wholesale, and the user asks about the replacement's \
              lexer.",
        gold: 0,
    },
    FamilyText {
        text: "The context carries the closed bug's full timeline; the next task \
              files a brand-new report about an unrelated crash.",
        gold: 0,
    },
    FamilyText {
        text: "The assembled context is one long finished interview; the user now \
              starts a clean-slate design session with new participants.",
        gold: 0,
    },
    FamilyText {
        text: "The prefix contains the retired crawler's configuration; the task \
              configures its event-driven successor, which shares no knobs.",
        gold: 0,
    },
    FamilyText {
        text: "The window holds the onboarding checklist the user completed this \
              morning; the next request is an advanced internals question it \
              never covered.",
        gold: 0,
    },
];

static CACHE_REUSE: FamilyDef = FamilyDef {
    name: CACHE_REUSE_NAME,
    qid: "cache_reuse",
    kind: QKind::Noul,
    labels: &["false", "true"],
    instructions: CACHE_REUSE_INSTR,
    score_levels: &[],
    note: CACHE_REUSE_NOTE,
    corpus: &CACHE_REUSE_CORPUS,
    cal: &CACHE_REUSE_CAL,
    eval: &CACHE_REUSE_EVAL,
};

#[must_use]
pub fn synth_cache_reuse() -> SynthData {
    build_family(&CACHE_REUSE)
}

// ── the shared builder ─────────────────────────────────────────────────────

fn build_family(def: &'static FamilyDef) -> SynthData {
    let criteria: Value = match def.kind {
        QKind::Choice => {
            let mut m = Map::new();
            for l in def.labels {
                m.insert((*l).to_string(), Value::Null);
            }
            Value::Object(m)
        }
        QKind::Score => Value::Array(
            def.score_levels
                .iter()
                .map(|s| Value::String((*s).to_string()))
                .collect(),
        ),
        QKind::Noul => Value::Null,
    };
    let case = |slice: &str, i: usize, t: &FamilyText| SuiteCase {
        id: format!("{}:{slice}:{i}", def.name),
        state: Value::String(t.text.to_string()),
        questions: vec![SuiteQuestion {
            qid: def.qid.to_string(),
            kind: def.kind,
            instructions: def.instructions.to_string(),
            criteria: criteria.clone(),
        }],
        gold: vec![GoldAnswer {
            idx: t.gold,
            soft: vec![0.0; def.labels.len()],
            gold_score: if def.kind == QKind::Score {
                Some(t.gold as f64)
            } else {
                None
            },
        }],
    };
    let cal_cases: Vec<SuiteCase> = def
        .cal
        .iter()
        .enumerate()
        .map(|(i, t)| case("cal", i, t))
        .collect();
    let cases: Vec<SuiteCase> = def
        .eval
        .iter()
        .enumerate()
        .map(|(i, t)| case("eval", i, t))
        .collect();
    let docs: Vec<TrainDoc> = def
        .corpus
        .iter()
        .map(|(g, text)| TrainDoc {
            label: def.labels[*g].to_string(),
            text: (*text).to_string(),
        })
        .collect();
    SynthData {
        suite: Suite {
            name: def.name,
            cases,
            option_counts_note: def.note,
        },
        cal_cases,
        docs,
        labels: def.labels.iter().map(|s| (*s).to_string()).collect(),
    }
}

/// The six modelless family definitions (registry order = suite order).
pub const FAMILY_DEFS: &[&FamilyDef] = &[&VIS, &PERM, &TOOL, &ROUTE, &SENS, &CACHE_REUSE];

/// Lookup by suite name (all six families carry a `FamilyDef`).
#[must_use]
pub fn family_def(name: &str) -> Option<&'static FamilyDef> {
    FAMILY_DEFS.iter().copied().find(|d| d.name == name)
}

/// Build any family's [`SynthData`] by suite name (all six).
#[must_use]
pub fn synth_by_name(name: &str) -> Option<SynthData> {
    family_def(name).map(build_family)
}

#[must_use]
pub fn synth_visibility() -> SynthData {
    build_family(&VIS)
}

#[must_use]
pub fn synth_permissions() -> SynthData {
    build_family(&PERM)
}

#[must_use]
pub fn synth_tool_fit() -> SynthData {
    build_family(&TOOL)
}

#[must_use]
pub fn synth_routing() -> SynthData {
    build_family(&ROUTE)
}

#[must_use]
pub fn synth_sensitivity() -> SynthData {
    build_family(&SENS)
}
