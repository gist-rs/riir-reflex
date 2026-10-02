//! The five widened family eval populations (Plan 009 REVISED-2 —
//! `.issues/059`): ~96 authored cases per family, class-balanced, replacing
//! each family's 12–16-case eval slice. `harness_cache_reuse` deliberately
//! keeps its frozen T3 12-fixture record (the documented divergence — see
//! `families.rs`'s `CACHE_REUSE_NOTE`).
//!
//! Authoring law (the gates in `tests/harness_families_gates.rs` enforce
//! every clause): no label tokens (per-family ban lists — TOOL bans the six
//! tool names, SENS bans every digit), no shared word 3-gram with the
//! family's corpus or cal texts (the memorization-path wall), per-case
//! unigram overlap ceilings, exact class balance, and a BLAKE3 digest pin
//! per family so any fixture edit reds the gate until consciously re-pinned.
//!
//! Authoring provenance: the `.scratch/famwide/*.tsv` sources + their
//! checker scripts (committed); assembled by
//! `.scratch/famwide/assemble_wide_eval.py`. The corpus and cal slices are
//! UNCHANGED — single-variable measurement.

// noqa-style note: this file is generated data; keep it data-only.

use super::families::FamilyText;

/// `harness_visibility` — the wide eval population (96 cases, class-balanced), authored under Plan 009 REVISED-2.
pub static VIS_WIDE_EVAL: [FamilyText; 96] = [
    FamilyText {
        text: "An upstream fork's changelog covering twenty features this codebase never merged into any branch.",
        gold: 0,
    },
    FamilyText {
        text: "Stale meeting minutes from a project that wound down two quarters ago, no decisions relevant today.",
        gold: 0,
    },
    FamilyText {
        text: "The retired nightly cleanup job whose cron entry was removed from the scheduler last month.",
        gold: 0,
    },
    FamilyText {
        text: "A canceled vendor webinar transcript about an admin console this stack never shipped.",
        gold: 0,
    },
    FamilyText {
        text: "Foreign import statements swept in from a neighboring repository's example app during the bulk search.",
        gold: 0,
    },
    FamilyText {
        text: "The outdated onboarding guide written before the team split services into separate deploy units.",
        gold: 0,
    },
    FamilyText {
        text: "Last winter's incident postmortem for a billing outage that a different service owns entirely now.",
        gold: 0,
    },
    FamilyText {
        text: "An expired migration checklist whose rollout finished cleanly eight sprints back and was archived.",
        gold: 0,
    },
    FamilyText {
        text: "Third-party telemetry constants copied from a sample app, none matching this deployment.",
        gold: 0,
    },
    FamilyText {
        text: "The decommissioned metrics endpoint documentation that the observability team replaced with a newer surface.",
        gold: 0,
    },
    FamilyText {
        text: "Superseded coding-standard drafts from before the lint policy settled, kept around solely for archaeology.",
        gold: 0,
    },
    FamilyText {
        text: "A neighbor team's feature flag registry, none of whose keys exist in this codebase.",
        gold: 0,
    },
    FamilyText {
        text: "Archived marketing copy for a launcher screen the product dropped before launch.",
        gold: 0,
    },
    FamilyText {
        text: "The legacy backup rotation script the storage group replaced with managed snapshots last year.",
        gold: 0,
    },
    FamilyText {
        text: "Interrupted prototype notes covering a plugin system that never advanced past the sketch phase.",
        gold: 0,
    },
    FamilyText {
        text: "Someone else's conference talk notes about a runtime this organization does not use.",
        gold: 0,
    },
    FamilyText {
        text: "The frozen 2019 style guide superseded by the automated formatter's configuration years ago.",
        gold: 0,
    },
    FamilyText {
        text: "A wrap-up thread for the retried deploy that eventually succeeded on its own.",
        gold: 0,
    },
    FamilyText {
        text: "The unused Swedish locale file committed by mistake next to the supported languages.",
        gold: 0,
    },
    FamilyText {
        text: "Random fixture data generated for a UI prototype unrelated to any shipped screen.",
        gold: 0,
    },
    FamilyText {
        text: "The deprecated webhook receiver's README, removed from service when the callbacks migrated elsewhere.",
        gold: 0,
    },
    FamilyText {
        text: "Quarter-old capacity planning numbers that two rebaselines and a hardware refresh made meaningless.",
        gold: 0,
    },
    FamilyText {
        text: "The boilerplate CONTRIBUTING template checked in from the scaffold, untouched since generation day.",
        gold: 0,
    },
    FamilyText {
        text: "An abandoned experiment harness for a scheduler the platform chose not to adopt.",
        gold: 0,
    },
    FamilyText {
        text: "Forty routine lint suppressions across the codebase; the digest is one sentence naming the pattern.",
        gold: 1,
    },
    FamilyText {
        text: "The changelog of a build plugin we consume; a single clause captures the relevant bump.",
        gold: 1,
    },
    FamilyText {
        text: "Twelve near-identical deprecation warnings from one vendored crate; trim them to their common cause.",
        gold: 1,
    },
    FamilyText {
        text: "The style-config diff from a tooling bump; its headline is the renamed rule.",
        gold: 1,
    },
    FamilyText {
        text: "Weekly flake reports from the UI suite, one line per week carries the signal.",
        gold: 1,
    },
    FamilyText {
        text: "The FAQ section on retry policies; its answer condenses into a single advisory sentence.",
        gold: 1,
    },
    FamilyText {
        text: "Routine dependency housekeeping: twenty transitive crates refreshed, the blurb is just the bumped lockfile hash.",
        gold: 1,
    },
    FamilyText {
        text: "An internal style tip repeated across six docs pages; its gist is one clause.",
        gold: 1,
    },
    FamilyText {
        text: "The inventory of example apps, most folders duplicating one template; state the count and move on.",
        gold: 1,
    },
    FamilyText {
        text: "Old deprecation scaffolding for two retired flags; the summary sentence fits on one row.",
        gold: 1,
    },
    FamilyText {
        text: "Benchmark configs for three discarded experiment arms; their takeaway is a single remark about noise.",
        gold: 1,
    },
    FamilyText {
        text: "The vendor's release bulletin for an optional exporter; capture the headline and ignore the appendices.",
        gold: 1,
    },
    FamilyText {
        text: "Recurring cron summaries from the maintenance pipeline; one sentence per entry preserves the schedule facts.",
        gold: 1,
    },
    FamilyText {
        text: "The glossary of metric names, most entries self-evident; one caption line covers the batch.",
        gold: 1,
    },
    FamilyText {
        text: "A tutorial walkthrough of the CLI, tangential to the fix; its outline is one line.",
        gold: 1,
    },
    FamilyText {
        text: "Twelve auto-generated changelog stubs for point releases; a single digest line holds their substance.",
        gold: 1,
    },
    FamilyText {
        text: "The minutes of a planning sync about queueing; one capsule sentence records the decision.",
        gold: 1,
    },
    FamilyText {
        text: "Catalog copy for a theme pack the editor lists; a one-sentence caption suffices.",
        gold: 1,
    },
    FamilyText {
        text: "The compatibility matrix of an optional codec; its takeaway is one row about versions.",
        gold: 1,
    },
    FamilyText {
        text: "Stacked announcements from the registry dashboard, each identical; one caption states the maintenance window.",
        gold: 1,
    },
    FamilyText {
        text: "The boilerplate section of generated client stubs; a headline stating the generator version covers it.",
        gold: 1,
    },
    FamilyText {
        text: "Nine consecutive green security scans with no findings; one line records the clean run.",
        gold: 1,
    },
    FamilyText {
        text: "The archive of solved FAQ tickets about auth cookies; each answer shrinks to its topic sentence.",
        gold: 1,
    },
    FamilyText {
        text: "Transitive license listings for bundled assets; their net result is one sentence about permissive terms.",
        gold: 1,
    },
    FamilyText {
        text: "The race window's interleaved thread dumps from every reproduction attempt; keep each schedule until the culprit emerges.",
        gold: 2,
    },
    FamilyText {
        text: "The deserialization path under suspicion, including every guard clause and the malformed payload it chokes on.",
        gold: 2,
    },
    FamilyText {
        text: "Memory-growth samples collected across every soak iteration, allocation sites annotated with their owners.",
        gold: 2,
    },
    FamilyText {
        text: "The flaky ordering failure's captured traces across ten reruns, timestamps preserved for the correlation work.",
        gold: 2,
    },
    FamilyText {
        text: "The rebalancing loop's suspected off-by-one, with the index math and the boundary case it mishandles.",
        gold: 2,
    },
    FamilyText {
        text: "Every packet capture from the stalled handshake, decoded field by field for the timeout hunt.",
        gold: 2,
    },
    FamilyText {
        text: "Cache stampede request logs during the thundering minute, arrival counts and backend latency intact.",
        gold: 2,
    },
    FamilyText {
        text: "The write-path regression's before-and-after profiles at each suspect site, retained until the culprit patch lands.",
        gold: 2,
    },
    FamilyText {
        text: "The deadlocked scheduler's lock acquisition sequence from every observed instance, holder and waiter labeled.",
        gold: 2,
    },
    FamilyText {
        text: "The schema drift investigation: every rejected row, its offending column, and the validator rule that flagged it.",
        gold: 2,
    },
    FamilyText {
        text: "Compiler diagnostics from the mis-optimization, each note tied to the intermediate representation it concerns.",
        gold: 2,
    },
    FamilyText {
        text: "The permission escalation trace under audit, every granted role and the code path that conferred it.",
        gold: 2,
    },
    FamilyText {
        text: "Spiky latency histograms from each deployment cohort, percentiles and sample counts retained for comparison.",
        gold: 2,
    },
    FamilyText {
        text: "Correlated backoff traces across services from the retry storm, every attempt timestamped for the cascade analysis.",
        gold: 2,
    },
    FamilyText {
        text: "The boot failure's kernel messages from each cold start, mount errors kept in order.",
        gold: 2,
    },
    FamilyText {
        text: "Regex backtracking blowup on the pathological input, with the matching steps that exploded.",
        gold: 2,
    },
    FamilyText {
        text: "Every assertion failure from the contract fuzz run, the offending pair and its seed recorded.",
        gold: 2,
    },
    FamilyText {
        text: "The index bloom investigation: query plans before and after each candidate, cost estimates kept.",
        gold: 2,
    },
    FamilyText {
        text: "The leak candidate's allocation stack from every growth phase, retained until the ownership question settles.",
        gold: 2,
    },
    FamilyText {
        text: "Malformed vendor callback responses captured during every reproduction, headers and payloads annotated.",
        gold: 2,
    },
    FamilyText {
        text: "Rollback artifacts from the failed cutover upgrade, error codes and stage names logged intact.",
        gold: 2,
    },
    FamilyText {
        text: "Cursor pagination's skipped-record reports from each boundary probe, offsets and row counts included.",
        gold: 2,
    },
    FamilyText {
        text: "The watermark regression's per-shard drift readings across every compaction, annotated beside the producer lag.",
        gold: 2,
    },
    FamilyText {
        text: "Span traces from the affected window during the throughput dip, each hop's queue depth recorded.",
        gold: 2,
    },
    FamilyText {
        text: "The regulator's consent clause under audit this sprint, untrimmed so counsel reads every word.",
        gold: 3,
    },
    FamilyText {
        text: "Crash signature hex supplied by the reporter, byte for byte, because the symbolizer keys on it.",
        gold: 3,
    },
    FamilyText {
        text: "The rate-limit stanza the production gateway enforces, kept in its deployed spelling, nothing elided.",
        gold: 3,
    },
    FamilyText {
        text: "The signer's recovery phrase policy under legal review, every clause unaltered from the executed version.",
        gold: 3,
    },
    FamilyText {
        text: "Compiler diagnostic output for the rejected trait impl, spans and error codes untouched.",
        gold: 3,
    },
    FamilyText {
        text: "The data-processing addendum the customer signed, quoted verbatim for the compliance packet.",
        gold: 3,
    },
    FamilyText {
        text: "Golden fixture bytes the serializer must reproduce, every byte preserved across the refactor.",
        gold: 3,
    },
    FamilyText {
        text: "The incident commander's declared timeline, quoted in its entirety for the postmortem record.",
        gold: 3,
    },
    FamilyText {
        text: "Import statement the patch relocates, shown exactly as authored, semicolons and all.",
        gold: 3,
    },
    FamilyText {
        text: "The consent banner text the legal team approved, unmodified for every locale build.",
        gold: 3,
    },
    FamilyText {
        text: "The lockfile conflict block verbatim, because the resolver's decision hinges on its exact shape.",
        gold: 3,
    },
    FamilyText {
        text: "Abatement notice from the standards board, reproduced word-perfect for the audit binder.",
        gold: 3,
    },
    FamilyText {
        text: "The signed artifact manifest the release gate checks, hashes and paths exactly as generated.",
        gold: 3,
    },
    FamilyText {
        text: "The stored procedure's original body under change review, kept uncut for the behavioral diff.",
        gold: 3,
    },
    FamilyText {
        text: "The handshake transcript the conformance suite replays, frames unmodified, because tolerance rules read raw bytes.",
        gold: 3,
    },
    FamilyText {
        text: "Embedded checksum constant from the migration, copied character for character, since the verifier compares raw text.",
        gold: 3,
    },
    FamilyText {
        text: "The accessibility statement the auditors cited, quoted complete with its exception clauses.",
        gold: 3,
    },
    FamilyText {
        text: "The parser's grammar rule under dispute, reproduced in its authored form for the review record.",
        gold: 3,
    },
    FamilyText {
        text: "The deprecation notice exactly as shipped, because downstream tooling greps its precise sentence.",
        gold: 3,
    },
    FamilyText {
        text: "The customer's escalation email in its entirety, unedited, for the response review.",
        gold: 3,
    },
    FamilyText {
        text: "The checksum block the firmware updater verifies, byte-exact, since any drift bricks the device.",
        gold: 3,
    },
    FamilyText {
        text: "On-call handoff notes completed during the outage, preserved unaltered for the review board.",
        gold: 3,
    },
    FamilyText {
        text: "The expired certificate's issuer string under forensic review, captured unmodified for the chain analysis.",
        gold: 3,
    },
    FamilyText {
        text: "Export schema's frozen field list, quoted complete, since downstream parsers match its literal shape.",
        gold: 3,
    },
];

/// `harness_permissions` — the wide eval population (96 cases, class-balanced), authored under Plan 009 REVISED-2.
pub static PERM_WIDE_EVAL: [FamilyText; 96] = [
    FamilyText {
        text: "Blame the parser module with git to trace which commit introduced the helper.",
        gold: 0,
    },
    FamilyText {
        text: "Render the workspace API docs with cargo doc --no-deps into target/.",
        gold: 0,
    },
    FamilyText {
        text: "Map the dependency graph rooted at the current crate with cargo tree.",
        gold: 0,
    },
    FamilyText {
        text: "Search the fixtures with ripgrep for placeholder strings still awaiting real data.",
        gold: 0,
    },
    FamilyText {
        text: "Count lines across the module sources with wc to size the refactor surface.",
        gold: 0,
    },
    FamilyText {
        text: "Summarize each recent commit's patch footprint with git log --stat.",
        gold: 0,
    },
    FamilyText {
        text: "Lint the deploy scripts with shellcheck, reporting syntax issues without editing.",
        gold: 0,
    },
    FamilyText {
        text: "Run cargo clippy over the touched crate with the default lint set.",
        gold: 0,
    },
    FamilyText {
        text: "Enumerate the test names with pytest --collect-only, executing nothing.",
        gold: 0,
    },
    FamilyText {
        text: "Pretty-print the captured API fixture's schema fields with jq.",
        gold: 0,
    },
    FamilyText {
        text: "Display the tagged release commit's diff summary with git show.",
        gold: 0,
    },
    FamilyText {
        text: "Read the lockfile diff the dependency upgrade produced.",
        gold: 0,
    },
    FamilyText {
        text: "Measure the incremental output directory with du to check artifact growth.",
        gold: 0,
    },
    FamilyText {
        text: "Query the local cache file with sqlite3, counting rows per table.",
        gold: 0,
    },
    FamilyText {
        text: "Check formatting drift on the edited files with rustfmt, rewriting nothing.",
        gold: 0,
    },
    FamilyText {
        text: "Preview the shelved work still pending with git stash list.",
        gold: 0,
    },
    FamilyText {
        text: "Curl the loopback health route to confirm the dev server is up.",
        gold: 0,
    },
    FamilyText {
        text: "Read the newest panic dump's crash backtrace with head.",
        gold: 0,
    },
    FamilyText {
        text: "Inventory the runnable demos with ls over the examples folder.",
        gold: 0,
    },
    FamilyText {
        text: "Emit the workspace graph as JSON with cargo metadata --no-deps.",
        gold: 0,
    },
    FamilyText {
        text: "Size the release delta with git diff --stat between the last two tags.",
        gold: 0,
    },
    FamilyText {
        text: "Surface type errors across the frontend sources with tsc --noEmit.",
        gold: 0,
    },
    FamilyText {
        text: "Count the TODO markers across the modules with grep, matching only.",
        gold: 0,
    },
    FamilyText {
        text: "List the locally cached container layers with docker images.",
        gold: 0,
    },
    FamilyText {
        text: "Show the configured origins for reference with git remote -v.",
        gold: 0,
    },
    FamilyText {
        text: "Hunt for orphaned images nothing references under the assets tree with find.",
        gold: 0,
    },
    FamilyText {
        text: "Profile one warmup iteration of the optimized binary with perf stat.",
        gold: 0,
    },
    FamilyText {
        text: "Compile the benchmark targets without executing them using cargo bench --no-run.",
        gold: 0,
    },
    FamilyText {
        text: "Outline the shipped archive contents with tar without extracting anything.",
        gold: 0,
    },
    FamilyText {
        text: "Inspect the checked-in test certificate's issuer metadata with openssl x509, keys excluded.",
        gold: 0,
    },
    FamilyText {
        text: "Summarize per-author contribution counts over the history with git shortlog.",
        gold: 0,
    },
    FamilyText {
        text: "Run the doc tests for the json module to verify examples compile.",
        gold: 0,
    },
    FamilyText {
        text: "Set up ripgrep globally with brew so every shell here picks it up.",
        gold: 1,
    },
    FamilyText {
        text: "Publish the JS bindings to npm under a fresh version number.",
        gold: 1,
    },
    FamilyText {
        text: "Push the rebuilt image to the org registry with docker.",
        gold: 1,
    },
    FamilyText {
        text: "Apply the patched deployment manifest to the cluster with kubectl.",
        gold: 1,
    },
    FamilyText {
        text: "Provision a new bucket in the cloud account with terraform apply.",
        gold: 1,
    },
    FamilyText {
        text: "Sync the release artifacts into the public bucket with aws s3.",
        gold: 1,
    },
    FamilyText {
        text: "Create the tagged build's GitHub release, uploading the archives with gh.",
        gold: 1,
    },
    FamilyText {
        text: "Deliver the release branch up to the team origin with git push.",
        gold: 1,
    },
    FamilyText {
        text: "Append a cron entry scheduling the nightly sync job.",
        gold: 1,
    },
    FamilyText {
        text: "Write a shell alias into the global bashrc for future sessions.",
        gold: 1,
    },
    FamilyText {
        text: "Roll the staging chart to the new revision with helm upgrade.",
        gold: 1,
    },
    FamilyText {
        text: "Ship the service to the live project with gcloud app deploy.",
        gold: 1,
    },
    FamilyText {
        text: "Release the edge worker to the platform with flyctl deploy.",
        gold: 1,
    },
    FamilyText {
        text: "Alias the staging deployment to production with vercel promote.",
        gold: 1,
    },
    FamilyText {
        text: "Point the docs subdomain's DNS record at the new host.",
        gold: 1,
    },
    FamilyText {
        text: "Add the CLI helper globally with pnpm so every project here sees it.",
        gold: 1,
    },
    FamilyText {
        text: "Place the linter bridge into the shared tool environment with pipx.",
        gold: 1,
    },
    FamilyText {
        text: "Authenticate against the internal registry with docker before the push window.",
        gold: 1,
    },
    FamilyText {
        text: "Update the team wiki page documenting the oncall rotation.",
        gold: 1,
    },
    FamilyText {
        text: "Run the pending migration that alters the shared staging schema.",
        gold: 1,
    },
    FamilyText {
        text: "Bring the existing load balancer under terraform management via import.",
        gold: 1,
    },
    FamilyText {
        text: "Set the git merge driver globally for every repository on the box.",
        gold: 1,
    },
    FamilyText {
        text: "Attach a webhook so every push pings the ops chat channel.",
        gold: 1,
    },
    FamilyText {
        text: "Upgrade the shared CI runner's packages with apt-get ahead of the sprint.",
        gold: 1,
    },
    FamilyText {
        text: "Register the new runner with the self-hosted CI fleet.",
        gold: 1,
    },
    FamilyText {
        text: "Mirror the container image digests up to the artifact store.",
        gold: 1,
    },
    FamilyText {
        text: "Retag the freshly published build as latest in the npm registry.",
        gold: 1,
    },
    FamilyText {
        text: "Swap the deploy credential on the release server ahead of the cutover.",
        gold: 1,
    },
    FamilyText {
        text: "Flush the CDN cache for the docs zone ahead of launch.",
        gold: 1,
    },
    FamilyText {
        text: "Expose the dev port to external traffic through a tunnel.",
        gold: 1,
    },
    FamilyText {
        text: "Grant the bot account write access to the org repositories.",
        gold: 1,
    },
    FamilyText {
        text: "Roll out the worker bundle to the edge environment with wrangler deploy.",
        gold: 1,
    },
    FamilyText {
        text: "Print the production env file with every service token inline.",
        gold: 2,
    },
    FamilyText {
        text: "Secure-copy the code-signing key to the bastion host for safekeeping.",
        gold: 2,
    },
    FamilyText {
        text: "Curl the webhook with the AWS secret in the query string.",
        gold: 2,
    },
    FamilyText {
        text: "Echo the session cookie jar into the CI annotation for diagnostics.",
        gold: 2,
    },
    FamilyText {
        text: "Dump the GPG agent's passphrase cache to the terminal.",
        gold: 2,
    },
    FamilyText {
        text: "Post the service-account JSON to the paste site for sharing.",
        gold: 2,
    },
    FamilyText {
        text: "Upload the wallet seed file to the gist for backup.",
        gold: 2,
    },
    FamilyText {
        text: "Write the raw disk image straight over the boot volume with dd.",
        gold: 2,
    },
    FamilyText {
        text: "Reformat the attached data disk from scratch with mkfs.",
        gold: 2,
    },
    FamilyText {
        text: "Drop the audit log tables in the live cluster schema.",
        gold: 2,
    },
    FamilyText {
        text: "Truncate the payments ledger right before the load test.",
        gold: 2,
    },
    FamilyText {
        text: "Recursively remove the untracked sibling worktree holding another agent's patches.",
        gold: 2,
    },
    FamilyText {
        text: "Sweep every untracked file across shared checkouts with git clean -xdf.",
        gold: 2,
    },
    FamilyText {
        text: "Shred the cold backup volume before the storage audit.",
        gold: 2,
    },
    FamilyText {
        text: "Force the rewritten main lineage over the protected upstream ref.",
        gold: 2,
    },
    FamilyText {
        text: "Delete the release tag on the origin that consumers pin against.",
        gold: 2,
    },
    FamilyText {
        text: "Zero out the RAID superblocks on the storage array.",
        gold: 2,
    },
    FamilyText {
        text: "Revoke every long-lived service token across the fleet at once.",
        gold: 2,
    },
    FamilyText {
        text: "Empty the WAL archive directory the point-in-time restore depends on.",
        gold: 2,
    },
    FamilyText {
        text: "Expunge the cached deploy credentials together with their source files.",
        gold: 2,
    },
    FamilyText {
        text: "Transfer the private key store to nobody and strip its permissions.",
        gold: 2,
    },
    FamilyText {
        text: "Embed the root password in the commit message for the record.",
        gold: 2,
    },
    FamilyText {
        text: "Log the full Authorization header for each incoming request.",
        gold: 2,
    },
    FamilyText {
        text: "Rewrite shared history in place with git filter-branch, no mirror backup.",
        gold: 2,
    },
    FamilyText {
        text: "Wipe the swap partition while the system is live.",
        gold: 2,
    },
    FamilyText {
        text: "Destroy the provisioning VMs mid-migration with a forced vagrant teardown.",
        gold: 2,
    },
    FamilyText {
        text: "Scrub the entire S3 version history for the compliance bucket.",
        gold: 2,
    },
    FamilyText {
        text: "Reset the hardware security module to factory defaults.",
        gold: 2,
    },
    FamilyText {
        text: "Purge the job queue, discarding every unacknowledged entry.",
        gold: 2,
    },
    FamilyText {
        text: "Uninstall the root certificate from the system trust store.",
        gold: 2,
    },
    FamilyText {
        text: "Unseal and export the vault's master key material to stdout.",
        gold: 2,
    },
    FamilyText {
        text: "Remove the parent directory assuming the glob matched nothing.",
        gold: 2,
    },
];

/// `harness_tool_fit` — the wide eval population (96 cases, class-balanced), authored under Plan 009 REVISED-2.
pub static TOOL_WIDE_EVAL: [FamilyText; 96] = [
    FamilyText {
        text: "Find every hardcoded endpoint URL scattered through the crate.",
        gold: 0,
    },
    FamilyText {
        text: "Sweep the repo for leftover credentials before the external review.",
        gold: 0,
    },
    FamilyText {
        text: "Find each truncated banner heading still shipping to staging.",
        gold: 0,
    },
    FamilyText {
        text: "Find each place the codename WANDERLUST appears verbatim.",
        gold: 0,
    },
    FamilyText {
        text: "Locate the exact panic message users keep reporting.",
        gold: 0,
    },
    FamilyText {
        text: "Find every stale feature-flag name mentioned in comments.",
        gold: 0,
    },
    FamilyText {
        text: "Scan the vendored sources for the misquoted disclaimer string.",
        gold: 0,
    },
    FamilyText {
        text: "Find each absolute file path left behind by the migration.",
        gold: 0,
    },
    FamilyText {
        text: "Pin down the offending snippet customers quote in bug reports.",
        gold: 0,
    },
    FamilyText {
        text: "Locate every placeholder token still shipped to production.",
        gold: 0,
    },
    FamilyText {
        text: "Find the retry budget literal buried in the client.",
        gold: 0,
    },
    FamilyText {
        text: "Sweep out every occurrence of the misspelled environment variable.",
        gold: 0,
    },
    FamilyText {
        text: "Find the legacy provider name hardcoded in the handshake.",
        gold: 0,
    },
    FamilyText {
        text: "Locate the currency symbol sprinkled through the invoice templates.",
        gold: 0,
    },
    FamilyText {
        text: "Find every vestigial debug marker before the code freeze.",
        gold: 0,
    },
    FamilyText {
        text: "Scan for the sentinel value the flaky check depends on.",
        gold: 0,
    },
    FamilyText {
        text: "Find every await nested inside a loop body, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Locate each match arm whose pattern binds nothing, by parse shape.",
        gold: 1,
    },
    FamilyText {
        text: "Find every if that wraps exactly one statement, by tree shape.",
        gold: 1,
    },
    FamilyText {
        text: "Pinpoint every enum variant that carries no payload, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Locate each impl block owning exactly one method, by shape.",
        gold: 1,
    },
    FamilyText {
        text: "Find each function whose tail expression is a bare unit, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Locate every nested if deeper than three levels, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Find each trait method lacking a default body, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Locate every tuple struct with more than four fields, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Find each call whose receiver is a chained field access, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Find each let that binds a tuple then destructures it immediately, by tree shape.",
        gold: 1,
    },
    FamilyText {
        text: "Locate every loop whose body holds a single continue.",
        gold: 1,
    },
    FamilyText {
        text: "Find each block that only forwards its lone argument, by tree shape.",
        gold: 1,
    },
    FamilyText {
        text: "Find every integer conversion hiding inside a comparison, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Locate every recursive call appearing in its own definition, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Find every pointer dereference sitting inside an index expression, structurally.",
        gold: 1,
    },
    FamilyText {
        text: "Who touched the pricing module since the big rewrite?",
        gold: 2,
    },
    FamilyText {
        text: "When did the retry policy land in the history?",
        gold: 2,
    },
    FamilyText {
        text: "Which revision replaced the polling loop with callbacks?",
        gold: 2,
    },
    FamilyText {
        text: "Blame whoever trimmed the timeout from thirty to ten.",
        gold: 2,
    },
    FamilyText {
        text: "Did anyone touch the ledger schema before Friday's deploy?",
        gold: 2,
    },
    FamilyText {
        text: "Trace the ancestry of the flaky handshake probe.",
        gold: 2,
    },
    FamilyText {
        text: "Who most recently rebased the importer onto the new backend?",
        gold: 2,
    },
    FamilyText {
        text: "When did the exporter begin emitting duplicate rows?",
        gold: 2,
    },
    FamilyText {
        text: "Which contributor renamed the shared config surface last month?",
        gold: 2,
    },
    FamilyText {
        text: "Did the hotfix that silenced the warning ever get merged?",
        gold: 2,
    },
    FamilyText {
        text: "Whose patch introduced the double permission check in the broker?",
        gold: 2,
    },
    FamilyText {
        text: "Reconstruct when the deprecated endpoint was quietly retired from duty.",
        gold: 2,
    },
    FamilyText {
        text: "Who owns the oldest surviving line in the lexer?",
        gold: 2,
    },
    FamilyText {
        text: "Which change severed the release notes from tagged milestones?",
        gold: 2,
    },
    FamilyText {
        text: "Date the revision that swapped the default profiler backend.",
        gold: 2,
    },
    FamilyText {
        text: "Find when two divergent implementations of caching first forked.",
        gold: 2,
    },
    FamilyText {
        text: "Are the integration suites green on tonight's release branch?",
        gold: 3,
    },
    FamilyText {
        text: "Run everything and tell me the failure tallies.",
        gold: 3,
    },
    FamilyText {
        text: "Does the nightly suite survive today's dependency bump?",
        gold: 3,
    },
    FamilyText {
        text: "After the schema migration, is anything red across the whole board?",
        gold: 3,
    },
    FamilyText {
        text: "Confirm the benchmark harness still clears its latency budget.",
        gold: 3,
    },
    FamilyText {
        text: "Which suites break if I bump the wire protocol?",
        gold: 3,
    },
    FamilyText {
        text: "Prove the frozen baselines survived the dedup sweep.",
        gold: 3,
    },
    FamilyText {
        text: "Is the whole matrix green before I tag the candidate?",
        gold: 3,
    },
    FamilyText {
        text: "Kick off a smoke pass and summarize what breaks.",
        gold: 3,
    },
    FamilyText {
        text: "Did my borrow-checker fix keep every parity probe happy?",
        gold: 3,
    },
    FamilyText {
        text: "Re-run the flaky shard suite until it settles.",
        gold: 3,
    },
    FamilyText {
        text: "How many checks fail on the sanitized fixture set?",
        gold: 3,
    },
    FamilyText {
        text: "Before the handoff, verify nothing regressed under the strict profile.",
        gold: 3,
    },
    FamilyText {
        text: "Run the property checks once more against the seeded corpus.",
        gold: 3,
    },
    FamilyText {
        text: "Does the merge candidate clear every gate on Windows?",
        gold: 3,
    },
    FamilyText {
        text: "Signal me when the soak finishes and tally the failures.",
        gold: 3,
    },
    FamilyText {
        text: "Tidy the indentation across the freshly extracted samples.",
        gold: 4,
    },
    FamilyText {
        text: "Normalize the line endings before the upcoming upstream sync.",
        gold: 4,
    },
    FamilyText {
        text: "Restyle the vendored bindings to match our conventions.",
        gold: 4,
    },
    FamilyText {
        text: "Collapse the stray double blank lines in the hot path.",
        gold: 4,
    },
    FamilyText {
        text: "Make the generated parser match the checked-in style sheet.",
        gold: 4,
    },
    FamilyText {
        text: "Strip trailing whitespace from every file the wizard touched.",
        gold: 4,
    },
    FamilyText {
        text: "Even out the import grouping across the crate root.",
        gold: 4,
    },
    FamilyText {
        text: "Rewrap the long signature lines to the agreed width.",
        gold: 4,
    },
    FamilyText {
        text: "Alphabetize the re-export block before the audit lands.",
        gold: 4,
    },
    FamilyText {
        text: "Apply our spacing rules to the translated manifests.",
        gold: 4,
    },
    FamilyText {
        text: "Standardize quote and comma style in the fixtures folder.",
        gold: 4,
    },
    FamilyText {
        text: "Straighten the misaligned table literals in the benchmarks.",
        gold: 4,
    },
    FamilyText {
        text: "Untangle the mixed indentation the merge introduced overnight.",
        gold: 4,
    },
    FamilyText {
        text: "Harmonize the brace placement across both engine halves.",
        gold: 4,
    },
    FamilyText {
        text: "Prepare the diff for review by evening out style.",
        gold: 4,
    },
    FamilyText {
        text: "Level the semicolon placement throughout the example binaries.",
        gold: 4,
    },
    FamilyText {
        text: "Which README section spells out the quota model?",
        gold: 5,
    },
    FamilyText {
        text: "Where are the onboarding steps for new maintainers written up?",
        gold: 5,
    },
    FamilyText {
        text: "Which handbook page covers the escalation ladder for reviewers?",
        gold: 5,
    },
    FamilyText {
        text: "Find the prose rule that governs snapshot retention windows.",
        gold: 5,
    },
    FamilyText {
        text: "What does the maintainer manual say about triage duties?",
        gold: 5,
    },
    FamilyText {
        text: "Point me to the runbook paragraph on cold restores.",
        gold: 5,
    },
    FamilyText {
        text: "Which guide chapter unpacks the tiering vocabulary for newcomers?",
        gold: 5,
    },
    FamilyText {
        text: "In which file does the contributor covenant live?",
        gold: 5,
    },
    FamilyText {
        text: "Locate the glossary entry that defines shard epochs.",
        gold: 5,
    },
    FamilyText {
        text: "Which notes describe the deprecation timeline for plugins?",
        gold: 5,
    },
    FamilyText {
        text: "Find the FAQ answer about rotating signing keys.",
        gold: 5,
    },
    FamilyText {
        text: "Where do the style guidelines discuss header banners?",
        gold: 5,
    },
    FamilyText {
        text: "Which quickstart walks a newcomer through the first boot?",
        gold: 5,
    },
    FamilyText {
        text: "Which appendix item defines the signed artifact envelope?",
        gold: 5,
    },
    FamilyText {
        text: "Is there a troubleshooting page for chronically stuck migrations?",
        gold: 5,
    },
    FamilyText {
        text: "Where can I read about the quarterly maintainer handoff ritual?",
        gold: 5,
    },
];

/// `harness_routing` — the wide eval population (96 cases, class-balanced), authored under Plan 009 REVISED-2.
pub static ROUTE_WIDE_EVAL: [FamilyText; 96] = [
    FamilyText {
        text: "Grade every incoming telemetry frame against the frozen severity ladder, one verdict per frame, entirely in memory.",
        gold: 0,
    },
    FamilyText {
        text: "Admit or drop each packet at line rate using the compiled filter table, never touching the wire.",
        gold: 0,
    },
    FamilyText {
        text: "Pick the cache eviction victim on every miss with the deterministic scorecard, reproducibly, millions of times daily.",
        gold: 0,
    },
    FamilyText {
        text: "Serve the autocomplete suggestion slot from the precomputed ranked list, identical output for identical keystrokes.",
        gold: 0,
    },
    FamilyText {
        text: "Answer each feature-flag probe inside the game loop before the next frame renders.",
        gold: 0,
    },
    FamilyText {
        text: "Stamp every sensor reading with its quality band via the lookup matrix, with zero round trips.",
        gold: 0,
    },
    FamilyText {
        text: "Adjudicate the spam verdict for every comment submission against the calibrated threshold card, sub-millisecond.",
        gold: 0,
    },
    FamilyText {
        text: "Resolve each order-book match decision in the matching core, deterministic and identical across replay runs.",
        gold: 0,
    },
    FamilyText {
        text: "Grant or deny the warehouse pick request from the static policy lattice, thousands of times per shift.",
        gold: 0,
    },
    FamilyText {
        text: "Compute the retry-or-fail verdict for each queued write inside the storage node, without leaving the process.",
        gold: 0,
    },
    FamilyText {
        text: "Evaluate the maintenance-window gate for every crane command on the embedded controller, with identical outcomes on replay.",
        gold: 0,
    },
    FamilyText {
        text: "Choose the ad slot winner per impression from the frozen scoring table, deterministically, every millisecond.",
        gold: 0,
    },
    FamilyText {
        text: "Reject or accept each inbound URL against the compiled denylist matcher before the request proceeds further.",
        gold: 0,
    },
    FamilyText {
        text: "Map every DNS query to its block or allow verdict with the in-memory ruleset, no exceptions.",
        gold: 0,
    },
    FamilyText {
        text: "Assign each job a priority tier from the fixed rubric the moment it enters the queue.",
        gold: 0,
    },
    FamilyText {
        text: "Score each gesture stroke against the recognizer's templates entirely on the wearable, with no calls home.",
        gold: 0,
    },
    FamilyText {
        text: "Decide the merge-or-reject verdict for each transaction against the consensus quorum rules, fully deterministic.",
        gold: 0,
    },
    FamilyText {
        text: "Flip the traffic-light phase per intersection cycle with the fixed timing chart, identical every run.",
        gold: 0,
    },
    FamilyText {
        text: "Rule on each returned parcel refund using the tiered matrix at the kiosk, instantly and repeatably.",
        gold: 0,
    },
    FamilyText {
        text: "Sort every request among four latency classes before the scheduler ever sees it, deterministically.",
        gold: 0,
    },
    FamilyText {
        text: "Vet each container image signature against the pinned trust anchors before the pod may start.",
        gold: 0,
    },
    FamilyText {
        text: "Settle every physics contact between the two bodies with the fixed restitution table inside the frame.",
        gold: 0,
    },
    FamilyText {
        text: "Return the same checksum verdict for the same input, every single call, straight from the hot cache.",
        gold: 0,
    },
    FamilyText {
        text: "Select one canned reply from the six-slot template wall for every nudge, wholly in memory.",
        gold: 0,
    },
    FamilyText {
        text: "Summarize each customer chat transcript in two sentences; there are forty thousand of them monthly.",
        gold: 1,
    },
    FamilyText {
        text: "Write one-line alt text for the two hundred thousand catalog photos, keeping wording plain.",
        gold: 1,
    },
    FamilyText {
        text: "Tag the language and tone of every inbound tweet, pennies per thousand items.",
        gold: 1,
    },
    FamilyText {
        text: "Draft three subject-line variants for tomorrow's promotional blast without spending real money per variant.",
        gold: 1,
    },
    FamilyText {
        text: "Convert the eighty thousand survey free-text answers into tidy categories, favoring adequate over polished.",
        gold: 1,
    },
    FamilyText {
        text: "Rewrite the four hundred help-center snippets in simpler wording; nobody reads them carefully anyway.",
        gold: 1,
    },
    FamilyText {
        text: "Extract contact names and dates from ten thousand voicemail transcripts this quarter; roughly right is fine.",
        gold: 1,
    },
    FamilyText {
        text: "Label each returned-product remark as praise, complaint, or question; margins are thin so keep unit price low.",
        gold: 1,
    },
    FamilyText {
        text: "Generate short meta descriptions for every recipe page; millions exist and nobody pays much per one.",
        gold: 1,
    },
    FamilyText {
        text: "Condense the meeting notes into five bullets per note; plain cleanup, priced by the million tokens.",
        gold: 1,
    },
    FamilyText {
        text: "Sort the ninety thousand store reviews into compliments, gripes, and feature wishes, inexpensively.",
        gold: 1,
    },
    FamilyText {
        text: "Caption the security-camera clips with one short line each; accuracy need not be flawless.",
        gold: 1,
    },
    FamilyText {
        text: "Produce plain-English answers for the thirty most-asked billing questions, updated weekly at negligible expense.",
        gold: 1,
    },
    FamilyText {
        text: "Hyphenate and casing-fix the ten thousand imported product titles so the catalog reads cleanly, at near-zero cost.",
        gold: 1,
    },
    FamilyText {
        text: "Author the tooltip copy for all nine hundred settings switches, brief and inexpensive by design.",
        gold: 1,
    },
    FamilyText {
        text: "Turn every caller's stated reason into a two-word disposition tag across the quarter's logs, at trivial cost.",
        gold: 1,
    },
    FamilyText {
        text: "Blurb each of the six hundred marketplace listings in one friendly sentence, keeping spend minimal.",
        gold: 1,
    },
    FamilyText {
        text: "Detect and mask the phone numbers inside ten thousand scraped bios, where approximate suffices.",
        gold: 1,
    },
    FamilyText {
        text: "Translate the onboarding emails into four languages with a passable first draft; humans polish later.",
        gold: 1,
    },
    FamilyText {
        text: "Pull each quoted price and warranty term out of scanned receipts, at rock-bottom unit cost.",
        gold: 1,
    },
    FamilyText {
        text: "Give each of the four million plays a one-phrase mood label before dawn, inexpensively.",
        gold: 1,
    },
    FamilyText {
        text: "Rate the readability of ten thousand textbook paragraphs, flagging the dozen worst for the editors' pass, at minimal spend.",
        gold: 1,
    },
    FamilyText {
        text: "Rewrite the bounce-back messages in kinder wording; hundreds daily, keep the per-message bill small.",
        gold: 1,
    },
    FamilyText {
        text: "Compose one-line scene descriptions for every paused lecture frame to serve the accessibility overlay while watching every cent.",
        gold: 1,
    },
    FamilyText {
        text: "Redesign the replicated log's failure-recovery path after the split-brain incident nobody can yet explain.",
        gold: 2,
    },
    FamilyText {
        text: "Hunt the deadlock that appears once a quarter and leaves no stack trace behind.",
        gold: 2,
    },
    FamilyText {
        text: "Devise a cache-coherence protocol for the heterogeneous accelerator mesh that keeps writes coherent without stalls.",
        gold: 2,
    },
    FamilyText {
        text: "Prove the scheduler's starvation-freedom claim on paper before anyone ships the new preemption mode.",
        gold: 2,
    },
    FamilyText {
        text: "Trace the numerical divergence that only shows up past the billionth integration step.",
        gold: 2,
    },
    FamilyText {
        text: "Draft the memory-model argument for why the lock-free queue is correct on weakly ordered silicon.",
        gold: 2,
    },
    FamilyText {
        text: "Diagnose why the query planner flips plans when statistics cross a million rows and latency triples.",
        gold: 2,
    },
    FamilyText {
        text: "Reason through the protocol downgrade attack that the penetration team cannot yet reproduce reliably.",
        gold: 2,
    },
    FamilyText {
        text: "Untangle which subsystem leaks file descriptors during the nightly stress marathon, from first principles.",
        gold: 2,
    },
    FamilyText {
        text: "Invent a compaction strategy that survives adversarial deletion patterns without ever stalling foreground reads.",
        gold: 2,
    },
    FamilyText {
        text: "Work out why the Bloom filter's false-positive rate drifts upward after each failover.",
        gold: 2,
    },
    FamilyText {
        text: "Sketch the consensus amendment that lets two datacenters disagree for ninety seconds yet converge.",
        gold: 2,
    },
    FamilyText {
        text: "Explain the second-long stall that appears only when the L3 cache line straddles two NUMA nodes.",
        gold: 2,
    },
    FamilyText {
        text: "Formalize the invariants the new journal must uphold so recovery proofs stay tractable.",
        gold: 2,
    },
    FamilyText {
        text: "Deduce the root cause of the checksum mismatch that survives three rewrites of the storage driver.",
        gold: 2,
    },
    FamilyText {
        text: "Decode why the garbage collector's pauses spike exactly during reference-promotion bursts.",
        gold: 2,
    },
    FamilyText {
        text: "Dissect the undocumented quirk that makes the vendor's TLS stack drop long handshakes.",
        gold: 2,
    },
    FamilyText {
        text: "Puzzle out the cache-thrashing pattern that emerges when three index builds overlap.",
        gold: 2,
    },
    FamilyText {
        text: "Argue whether the optimistic concurrency check can starve readers under the new retry storm.",
        gold: 2,
    },
    FamilyText {
        text: "Design the resharding dance that moves petabytes without dropping a single in-flight request.",
        gold: 2,
    },
    FamilyText {
        text: "Chase the ghost write that corrupts one page per ten thousand flushes, inexplicably.",
        gold: 2,
    },
    FamilyText {
        text: "Formulate the backpressure policy for the stream processor when upstream bursts exceed capacity tenfold.",
        gold: 2,
    },
    FamilyText {
        text: "Check the Alloy model's counterexample that explains the lost update under contention.",
        gold: 2,
    },
    FamilyText {
        text: "Audit the timing side channel that leaks key bits through the branch predictor.",
        gold: 2,
    },
    FamilyText {
        text: "Recompute every player's seasonal leaderboard standing after midnight, when nobody is watching.",
        gold: 3,
    },
    FamilyText {
        text: "Roll up the hourly counters into daily tallies each night at three.",
        gold: 3,
    },
    FamilyText {
        text: "Retire the cold snapshots to tape every Sunday, long after the dashboards go quiet.",
        gold: 3,
    },
    FamilyText {
        text: "Shrink the append-only ledger every Tuesday morning while traffic idles near zero.",
        gold: 3,
    },
    FamilyText {
        text: "Rebuild the recommendation matrices over the weekend once the fresh event dump lands.",
        gold: 3,
    },
    FamilyText {
        text: "Purge the expired session rows on the first Monday of each month, per policy.",
        gold: 3,
    },
    FamilyText {
        text: "Mail the quarterly investor pack once the numbers freeze on the last business day.",
        gold: 3,
    },
    FamilyText {
        text: "Refresh the warehouse marts from the upstream extracts daily between two and four.",
        gold: 3,
    },
    FamilyText {
        text: "Rotate the signing certificates across the fleet during Saturday's maintenance slot.",
        gold: 3,
    },
    FamilyText {
        text: "Compile the weekly changelog digest every Friday evening from the merged pull requests.",
        gold: 3,
    },
    FamilyText {
        text: "Regrade the terabyte of stored photos for new face crops whenever the idle box frees up.",
        gold: 3,
    },
    FamilyText {
        text: "Trim the object store down to the retention window during the quarterly housekeeping pass.",
        gold: 3,
    },
    FamilyText {
        text: "Reindex the forum posts against the new ranking signals every first Sunday, off-peak.",
        gold: 3,
    },
    FamilyText {
        text: "Rescan the whole photo library for duplicates while the office sleeps.",
        gold: 3,
    },
    FamilyText {
        text: "Ship the monthly metrics digest to stakeholders once the final bucket closes at midnight.",
        gold: 3,
    },
    FamilyText {
        text: "Re-encode the ten-year video archive into the newer codec across several weekend windows.",
        gold: 3,
    },
    FamilyText {
        text: "Vacuum and defragment the document tables overnight behind a short banner notice.",
        gold: 3,
    },
    FamilyText {
        text: "Reload the geo mapping table from the vendor drop whenever it arrives, off the serving path.",
        gold: 3,
    },
    FamilyText {
        text: "Churn through the backlog of unanswered webhook retries each evening until the queue drains.",
        gold: 3,
    },
    FamilyText {
        text: "Fold the edge-server logs into the columnar store every hour, on the odd minutes.",
        gold: 3,
    },
    FamilyText {
        text: "Rewarm the search suggester's caches from nothing every sunrise, before the first query lands.",
        gold: 3,
    },
    FamilyText {
        text: "Migrate last year's orders into the archival bucket each January first, unhurried.",
        gold: 3,
    },
    FamilyText {
        text: "Transcode the podcast back catalog to the space-saving format over the holiday lull.",
        gold: 3,
    },
    FamilyText {
        text: "Send the daily wrap-up note to the on-call channel at six, when the noise dies.",
        gold: 3,
    },
];

/// `harness_sensitivity` — the wide eval population (100 cases, class-balanced), authored under Plan 009 REVISED-2.
pub static SENS_WIDE_EVAL: [FamilyText; 100] = [
    FamilyText {
        text: "The press kit folder with logos and boilerplate for journalists covering the launch.",
        gold: 0,
    },
    FamilyText {
        text: "The blog post announcing last month's feature drop, straight from the marketing team.",
        gold: 0,
    },
    FamilyText {
        text: "The contributing guide that shows newcomers how to submit their first patch.",
        gold: 0,
    },
    FamilyText {
        text: "The code-of-conduct page every community member agrees to before posting.",
        gold: 0,
    },
    FamilyText {
        text: "The screenshot gallery showcasing the redesigned dashboard in several color themes.",
        gold: 0,
    },
    FamilyText {
        text: "The quickstart tutorial that gets a fresh clone compiling within minutes.",
        gold: 0,
    },
    FamilyText {
        text: "The podcast episode where the founders discuss the roadmap for the year ahead.",
        gold: 0,
    },
    FamilyText {
        text: "The webinar deck explaining the architecture to a nontechnical audience.",
        gold: 0,
    },
    FamilyText {
        text: "The newsletter archive holding every monthly digest since the project went public.",
        gold: 0,
    },
    FamilyText {
        text: "The download page linking the signed installers for each supported platform.",
        gold: 0,
    },
    FamilyText {
        text: "The glossary entry defining every term newcomers encounter in the handbook.",
        gold: 0,
    },
    FamilyText {
        text: "The office-hours notes where maintainers answered community questions last quarter.",
        gold: 0,
    },
    FamilyText {
        text: "The changelog page on the docs portal listing what shipped since the winter release.",
        gold: 0,
    },
    FamilyText {
        text: "The status page widget that reports uptime to visitors in plain language.",
        gold: 0,
    },
    FamilyText {
        text: "The brand guidelines specifying which accent colors accompany the wordmark.",
        gold: 0,
    },
    FamilyText {
        text: "The community showcase thread collecting fan-built plugins and themes from the forums.",
        gold: 0,
    },
    FamilyText {
        text: "The migration guide walking administrators through upgrading from the legacy release.",
        gold: 0,
    },
    FamilyText {
        text: "The interview article where the engineering lead demos the public beta build.",
        gold: 0,
    },
    FamilyText {
        text: "The faq entry clarifying which license applies to the sample snippets.",
        gold: 0,
    },
    FamilyText {
        text: "The poster artwork pinned beside the booth at last spring's expo.",
        gold: 0,
    },
    FamilyText {
        text: "The vendored fork of the YAML loader, carrying small local patches.",
        gold: 1,
    },
    FamilyText {
        text: "The third-party compression crate mirrored into the vendor directory at a pinned revision.",
        gold: 1,
    },
    FamilyText {
        text: "The checksum entry that pins the registry tarball of the templating library.",
        gold: 1,
    },
    FamilyText {
        text: "The git submodule snapshot tracking the upstream parser library, unmodified.",
        gold: 1,
    },
    FamilyText {
        text: "The dependency's migration notes copied into the tree for reference during upgrades.",
        gold: 1,
    },
    FamilyText {
        text: "The Apache license header at the top of every vendored math routine.",
        gold: 1,
    },
    FamilyText {
        text: "The upstream test suite bundled beside the bindings so regressions stay reproducible.",
        gold: 1,
    },
    FamilyText {
        text: "The wasm glue package imported wholesale from the original authors.",
        gold: 1,
    },
    FamilyText {
        text: "The crate metadata describing the pinned semver range for the tracing client.",
        gold: 1,
    },
    FamilyText {
        text: "The vendored image codec whose upstream repo we mirror for offline builds.",
        gold: 1,
    },
    FamilyText {
        text: "The gem spec synced from the community package index at install time.",
        gold: 1,
    },
    FamilyText {
        text: "The vendored protobuf definitions generated from the schemas of the upstream service.",
        gold: 1,
    },
    FamilyText {
        text: "The changelog excerpt from the date library, documenting its breaking releases.",
        gold: 1,
    },
    FamilyText {
        text: "The BSD-licensed renderer module dropped into vendor/ during the dependency audit.",
        gold: 1,
    },
    FamilyText {
        text: "The shell completions copied from the CLI toolkit we depend on.",
        gold: 1,
    },
    FamilyText {
        text: "The upstream issue tracker export archived next to the vendored sources.",
        gold: 1,
    },
    FamilyText {
        text: "The polyfill shim adapted slightly from the browser compatibility library.",
        gold: 1,
    },
    FamilyText {
        text: "The module cache entry referencing the released version of the websocket library.",
        gold: 1,
    },
    FamilyText {
        text: "The example programs shipped inside the vendored database driver for reference.",
        gold: 1,
    },
    FamilyText {
        text: "The compatibility layer vendored from the SDK, matching its published release exactly.",
        gold: 1,
    },
    FamilyText {
        text: "The pricing engine that computes subscription quotes from our rate cards.",
        gold: 2,
    },
    FamilyText {
        text: "The recommendation module blending viewing history with taste embeddings for suggestions.",
        gold: 2,
    },
    FamilyText {
        text: "The fraud heuristic that flags unusually rapid checkout attempts for review.",
        gold: 2,
    },
    FamilyText {
        text: "The matchmaking routine pairing players by latent skill and latency budget.",
        gold: 2,
    },
    FamilyText {
        text: "The sprite atlas packer that packs hand-drawn frames into GPU-friendly pages.",
        gold: 2,
    },
    FamilyText {
        text: "The physics solver integrating our custom friction model every tick.",
        gold: 2,
    },
    FamilyText {
        text: "The lexer and parser for the templating DSL we designed in-house.",
        gold: 2,
    },
    FamilyText {
        text: "The schema migrator that applies versioned DDL batches to the product database on startup.",
        gold: 2,
    },
    FamilyText {
        text: "The cache eviction policy tuned for our read-heavy workload patterns.",
        gold: 2,
    },
    FamilyText {
        text: "The binary protocol encoder that serializes world snapshots for the netcode.",
        gold: 2,
    },
    FamilyText {
        text: "The leaderboard aggregator computing seasonal ranks from raw match outcomes.",
        gold: 2,
    },
    FamilyText {
        text: "The batch scheduler that groups telemetry flushes to protect the tick budget.",
        gold: 2,
    },
    FamilyText {
        text: "The reconciler that settles divergent inventory counts after a node rejoins.",
        gold: 2,
    },
    FamilyText {
        text: "The entitlement checker deciding which premium features unlock for each account.",
        gold: 2,
    },
    FamilyText {
        text: "The order-matching core pairing limit orders against the live book.",
        gold: 2,
    },
    FamilyText {
        text: "The search ranking head that blends lexical scores with our learned prior.",
        gold: 2,
    },
    FamilyText {
        text: "The loyalty accrual module awarding points for streaks and referrals.",
        gold: 2,
    },
    FamilyText {
        text: "The widget toolkit rendering our bespoke calendar controls on the canvas.",
        gold: 2,
    },
    FamilyText {
        text: "The compaction pass that merges cold segments during idle windows.",
        gold: 2,
    },
    FamilyText {
        text: "The domain model expressing orders, holdings, and settlements in the ledger core.",
        gold: 2,
    },
    FamilyText {
        text: "The nginx pool routing traffic to the staging fleet's health endpoints.",
        gold: 3,
    },
    FamilyText {
        text: "The kubernetes rollout manifest describing canary weights across the node pools.",
        gold: 3,
    },
    FamilyText {
        text: "The terraform module provisioning the private subnets and NAT gateways for the database tier.",
        gold: 3,
    },
    FamilyText {
        text: "The CI pipeline definition that lints, tests, and publishes on every merge.",
        gold: 3,
    },
    FamilyText {
        text: "The ansible playbook that hardens the bastion before it joins the production network.",
        gold: 3,
    },
    FamilyText {
        text: "The compose file wiring the database, cache, and queue for local orchestration.",
        gold: 3,
    },
    FamilyText {
        text: "The ingress ruleset routing external paths to the right backend services.",
        gold: 3,
    },
    FamilyText {
        text: "The load-balancer health probe settings that drain instances before shutdown.",
        gold: 3,
    },
    FamilyText {
        text: "The DNS zone file listing the records for every internal service name.",
        gold: 3,
    },
    FamilyText {
        text: "The cron schedule that snapshots the metrics store nightly to the archive bucket.",
        gold: 3,
    },
    FamilyText {
        text: "The service-mesh routing table steering mirrored traffic into the shadow cluster.",
        gold: 3,
    },
    FamilyText {
        text: "The scrape configuration enumerating the metrics endpoints across every monitored host.",
        gold: 3,
    },
    FamilyText {
        text: "The wireguard topology file describing the tunnels between regional relays.",
        gold: 3,
    },
    FamilyText {
        text: "The subnet allocation sheet reserving address blocks for each availability zone.",
        gold: 3,
    },
    FamilyText {
        text: "The build pipeline that cross-compiles the release binaries for every target triple.",
        gold: 3,
    },
    FamilyText {
        text: "The proxy allowlist that gates which client origins may reach the API gateway.",
        gold: 3,
    },
    FamilyText {
        text: "The retention policy governing how long access logs persist in cold storage.",
        gold: 3,
    },
    FamilyText {
        text: "The autoscaling thresholds that grow the worker pool as queue depth climbs.",
        gold: 3,
    },
    FamilyText {
        text: "The migration runner config that applies database revisions before each deploy.",
        gold: 3,
    },
    FamilyText {
        text: "The observability dashboard definition charting tail latency per service tier.",
        gold: 3,
    },
    FamilyText {
        text: "The dotfile at the repo root holding the production database credentials.",
        gold: 4,
    },
    FamilyText {
        text: "The OAuth client secret issued for the billing dashboard's server-side calls.",
        gold: 4,
    },
    FamilyText {
        text: "The SSH identity that authenticates deploys to the bare git remote.",
        gold: 4,
    },
    FamilyText {
        text: "The webhook secret that authenticates every payload the vendor posts to us.",
        gold: 4,
    },
    FamilyText {
        text: "The HMAC secret used to sign the license activation responses.",
        gold: 4,
    },
    FamilyText {
        text: "The service-account JSON with the storage keys the nightly export assumes.",
        gold: 4,
    },
    FamilyText {
        text: "The vault unseal shares split across the keepers for disaster recovery.",
        gold: 4,
    },
    FamilyText {
        text: "The TLS private key backing the edge certificate for the storefront domain.",
        gold: 4,
    },
    FamilyText {
        text: "The bot token letting the release announcer post to the chat workspace.",
        gold: 4,
    },
    FamilyText {
        text: "The recovery codes sheet for the owner's hardware second factor.",
        gold: 4,
    },
    FamilyText {
        text: "The deploy passphrase guarding the encrypted bundle the installer fetches.",
        gold: 4,
    },
    FamilyText {
        text: "The cluster bootstrap token that nodes present when joining the managed fleet.",
        gold: 4,
    },
    FamilyText {
        text: "The publish token authorizing uploads of signed artifacts to the registry.",
        gold: 4,
    },
    FamilyText {
        text: "The pepper constant added to every password hash before storage.",
        gold: 4,
    },
    FamilyText {
        text: "The relay credentials the on-prem agent presents to phone home.",
        gold: 4,
    },
    FamilyText {
        text: "The cookie-signing secret that keeps session identifiers unforgeable across the storefront.",
        gold: 4,
    },
    FamilyText {
        text: "The MFA seed values provisioned for every operator account in the vault.",
        gold: 4,
    },
    FamilyText {
        text: "The release signing identity kept offline except during the monthly promotion.",
        gold: 4,
    },
    FamilyText {
        text: "The environment variables file exporting the third-party credentials for staging.",
        gold: 4,
    },
    FamilyText {
        text: "The encryption passphrase protecting the offline backup of the signer identities.",
        gold: 4,
    },
];
