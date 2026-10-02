//! Issue 061 — the `semantic_defects` family's wide eval population
//! (102 fixtures, 17 per class, exact balance; the wide authoring law
//! applies — label-token-free, zero shared word 3-grams with corpus∪cal,
//! fresh vocabulary). Assembled with the families module; the digest pin
//! lives in `tests/harness_families_gates.rs`.

use crate::harness::families::FamilyText;

pub static DEFECTS_WIDE_EVAL: [FamilyText; 102] = [
    // ── clean (0) × 17 ──
    FamilyText {
        text: "Copies each reading with `for i in 0 .. samples.len()`; the pass \
               ends where the buffer does, nothing short, nothing past.",
        gold: 0,
    },
    FamilyText {
        text: "Tiles with `&payload[i .. min(i + chunk, len)]`; the final short \
               tile shrinks to the remainder instead of reaching outside.",
        gold: 0,
    },
    FamilyText {
        text: "Pages half-open `page * per_page .. ((page + 1) * per_page).min(items.len())`; \
               boundaries never repeat and never overrun.",
        gold: 0,
    },
    FamilyText {
        text: "Advance wraps `(head + 1) % capacity`; after the final reserved \
               slot the cursor returns to zero.",
        gold: 0,
    },
    FamilyText {
        text: "`while cursor < pending.len()` dispatches each queued item the \
               count describes, in arrival order.",
        gold: 0,
    },
    FamilyText {
        text: "`end = start + keep` preserves every requested row through the \
               final index of the selection.",
        gold: 0,
    },
    FamilyText {
        text: "Argument parsing consumes the program name into its own binding \
               first, then iterates `argv[1 ..]` as the flag list.",
        gold: 0,
    },
    FamilyText {
        text: "Sums `for i in 0 .. n`, exactly the `n` readings the caller \
               passed in.",
        gold: 0,
    },
    FamilyText {
        text: "Folds `bytes[i .. i + 4]` per full group; the short tail group \
               folds its true remainder length.",
        gold: 0,
    },
    FamilyText {
        text: "Midpoint `(lo + hi) / 2` against an exclusive `hi`; the rightmost \
               probe stays inside the slice.",
        gold: 0,
    },
    FamilyText {
        text: "The trailing pair is `&history[history.len() - 2 ..]` — precisely \
               the two newest revisions, no more.",
        gold: 0,
    },
    FamilyText {
        text: "Zero-fill runs `for slot in 0 .. buffer.len()`; no slot keeps \
               stale bytes after the pass.",
        gold: 0,
    },
    FamilyText {
        text: "Gathers `src[x * 4 + k]` for `k in 0 .. 4`; every read of the \
               tile lands inside `src`.",
        gold: 0,
    },
    FamilyText {
        text: "`attempt in 0 .. max_retries` yields exactly the configured \
               number of sleeps between attempts.",
        gold: 0,
    },
    FamilyText {
        text: "The dedup scan initializes `j` to 1 before its first \
               `buf[j - 1]` read.",
        gold: 0,
    },
    FamilyText {
        text: "The FFI boundary passes `data.len() - 1`, the last index the \
               native side documents.",
        gold: 0,
    },
    FamilyText {
        text: "`assert!(idx < len)` precedes the fetch, so a probe at the \
               length itself is refused before indexing.",
        gold: 0,
    },
    // ── off_by_one (1) × 17 ──
    FamilyText {
        text: "Scanning runs `for i in 0 ..= samples.len()` to copy each \
               reading; the final iteration indexes the sentinel slot after \
               the buffer.",
        gold: 1,
    },
    FamilyText {
        text: "Tiling `&payload[i .. i + chunk]` while stepping `i += chunk`; \
               when `payload.len()` is not a multiple of `chunk`, the final \
               tile's end index lands outside the buffer.",
        gold: 1,
    },
    FamilyText {
        text: "Page math takes `page * per_page ..= page * per_page + per_page` \
               as the window, so each page repeats its neighbor's final row \
               and the last page touches index `items.len()`.",
        gold: 1,
    },
    FamilyText {
        text: "Advance wraps with `(head + 1) % (capacity + 1)`, letting the \
               write cursor land at `capacity` — an offset the allocation never \
               reserved.",
        gold: 1,
    },
    FamilyText {
        text: "`while cursor <= pending.len()` dispatches `pending[cursor]` and \
               increments; the last dispatch indexes the empty position.",
        gold: 1,
    },
    FamilyText {
        text: "Trimming computes `end = start + keep - 1` and slices \
               `rows[start .. end]`, dropping the final kept row whenever \
               `keep` rows were requested.",
        gold: 1,
    },
    FamilyText {
        text: "`for k in 1 .. argc` feeds the parser every argument except the \
               first flag; `argv[0]`'s program name gets parsed in its place.",
        gold: 1,
    },
    FamilyText {
        text: "Accumulation runs `sum += readings[i]` for `i in 0 ..= n` where \
               `n` was handed in as the reading count.",
        gold: 1,
    },
    FamilyText {
        text: "The checksum folds `bytes[i ..= i + 3]` per group of four; the \
               inclusive end re-reads each next group's first byte, and the \
               final group reaches outside the buffer.",
        gold: 1,
    },
    FamilyText {
        text: "Split point uses `(lo + hi + 1) / 2` with a `hi` that is already \
               exclusive, so the rightmost recursion probes `items[items.len()]`.",
        gold: 1,
    },
    FamilyText {
        text: "Snapshotting takes `&history[history.len() - 1 ..]` under the \
               comment about the last two revisions, yielding only the newest \
               entry.",
        gold: 1,
    },
    FamilyText {
        text: "The zero-fill pass runs `for slot in 1 .. buffer.len()`, leaving \
               the initial slot holding whatever it held before.",
        gold: 1,
    },
    FamilyText {
        text: "The 4-wide transpose reads `src[x * 4 + k]` for `k in 0 ..= 3` \
               at the final column, touching `src[len]`.",
        gold: 1,
    },
    FamilyText {
        text: "Backoff sleeps `2 ^ attempt` seconds for `attempt in 0 ..= \
               max_retries`, producing a further sleep than the config's \
               documented count.",
        gold: 1,
    },
    FamilyText {
        text: "The dedup scan compares `buf[j - 1]` starting at `j = 0`; the \
               index wraps to `usize::MAX` and the first read is out of \
               bounds.",
        gold: 1,
    },
    FamilyText {
        text: "The FFI call passes `data.len()` where the native side expects \
               a last index; each call reads a single slot beyond the frame, \
               and 4096-byte embeds trip it at the seam.",
        gold: 1,
    },
    FamilyText {
        text: "`assert!(idx <= len)` guards the fetch, so `idx == len` proceeds \
               and `ledger[idx]` panics at month close.",
        gold: 1,
    },
    // ── inverted_condition (2) × 17 ──
    FamilyText {
        text: "The comment reads 'skip entries already cached'; the body runs \
               `if cache.contains(key) { recompute(key); }` — the uncached \
               paths stall.",
        gold: 2,
    },
    FamilyText {
        text: "A guard documented as admit-when-verified ships as \
               `if !token.verified { grant(session); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Retry while the connection is down' is implemented as \
               `while link.is_up() { backoff(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Reject when the quota is exhausted' appears as \
               `if quota.remaining() > 0 { reject(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "The docstring says merge when equal; the code runs \
               `if a.hash != b.hash { merge(a, b); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Show the banner only during maintenance' ships \
               `if !maintenance.active { render_banner(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Panic when the checksum disagrees' is encoded as \
               `if checksum == expected { panic!(\"corrupt frame\"); }`.",
        gold: 2,
    },
    FamilyText {
        text: "The access rule 'staff take the fast path' ships with its sense \
               flipped: `if !user.is_staff { fast_path(user); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Debounce: ignore repeats inside the window' lands as \
               `if elapsed < window { fire(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Escalate when retries are exhausted' is written \
               `if attempt < max { escalate(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "The reaper's rule 'past the idle cap, drop' is coded \
               `if s.idle_secs() <= idle_cap { reap(s); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Skip the write when nothing changed' got turned into \
               `if !dirty { flush(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Mute when the level exceeds the cap' ships \
               `if level < cap { mute(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "The 'fast path when the table is warm' shortcut is encoded \
               `if !table.warm { fast_path(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "Docs say block writes during migration; the guard is \
               `if migration.done { block_writes(); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Alert on divergence' compiled to \
               `if local == remote { alert(operator); }`.",
        gold: 2,
    },
    FamilyText {
        text: "'Populate the cache when cold' was written \
               `if cache.is_fresh() { populate(cache); }`.",
        gold: 2,
    },
    // ── unwrapped_none (3) × 17 ──
    FamilyText {
        text: "`config.get(\"retry_limit\").unwrap()` runs during boot; \
               deployments lacking the optional key never come up.",
        gold: 3,
    },
    FamilyText {
        text: "`headers.get(\"etag\").expect(\"present\")` sits on a response \
               path whose server omits the header for cached replies.",
        gold: 3,
    },
    FamilyText {
        text: "`rows.first().unwrap()` is applied to a query result the API \
               documents as possibly empty.",
        gold: 3,
    },
    FamilyText {
        text: "`env::var(\"REGION\").unwrap()` appears inside a crate that \
               claims region-optional operation in its feature docs.",
        gold: 3,
    },
    FamilyText {
        text: "`map[&session_id]` indexes directly for a session the sweeper \
               may have expired between request and dispatch.",
        gold: 3,
    },
    FamilyText {
        text: "`parts.next().unwrap()` drives the split of user input, and the \
               empty-string path reaches it.",
        gold: 3,
    },
    FamilyText {
        text: "`resolve_route(path).unwrap()` runs in the router's fallthrough, \
               where unmatched paths are documented to reach the 404 handler \
               instead.",
        gold: 3,
    },
    FamilyText {
        text: "`latest_checkpoint().unwrap()` runs on a fresh install, where no \
               checkpoint exists yet — exactly the state a first boot is \
               documented to present.",
        gold: 3,
    },
    FamilyText {
        text: "`snapshot.regions.get(id).unwrap()` fetches a region deleted \
               between the listing call and the fetch.",
        gold: 3,
    },
    FamilyText {
        text: "`classify(byte).unwrap()` processes bytes above 127 that the \
               parser was promised it would never see.",
        gold: 3,
    },
    FamilyText {
        text: "`queue.pop().unwrap()` runs in the worker loop after a length \
               check that races with another consumer.",
        gold: 3,
    },
    FamilyText {
        text: "`Decimal::from_str(user_text).unwrap()` sits where free-form input \
               enters the ledger.",
        gold: 3,
    },
    FamilyText {
        text: "`metadata.get(\"schema_version\").unwrap()` meets datasets from \
               the previous format, which predate the key.",
        gold: 3,
    },
    FamilyText {
        text: "`txn.rows.iter().max().unwrap()` runs on the empty-ledger branch \
               the data model explicitly allows.",
        gold: 3,
    },
    FamilyText {
        text: "`text.split('\\n').next().unwrap()` handles a file that is \
               legitimately zero bytes.",
        gold: 3,
    },
    FamilyText {
        text: "`peer_table.entry(addr).or_default().last_seen.unwrap()` — the \
               default entry carries no timestamp, and boot-time pings panic \
               there.",
        gold: 3,
    },
    FamilyText {
        text: "`settings.theme.unwrap()` reads a field the loader documents as \
               user-optional.",
        gold: 3,
    },
    // ── swapped_lookup (4) × 17 ──
    FamilyText {
        text: "`prices[listing.seller_id]` is fetched for the quote that \
               belongs to `listing.sku`; both maps take the same value shape, \
               so nothing fails to compile.",
        gold: 4,
    },
    FamilyText {
        text: "`flags.get(\"verbose\")` is assigned into the `quiet` setting — \
               the twin key on that table, bound to the wrong knob.",
        gold: 4,
    },
    FamilyText {
        text: "`rows[col][row]` is transposed in the heatmap builder; test \
               fixtures are symmetric, and real data is not.",
        gold: 4,
    },
    FamilyText {
        text: "`users[display_name]` resolves where the session table keys on \
               `session_id`; two users sharing a display name collide.",
        gold: 4,
    },
    FamilyText {
        text: "`palette.accent` is fetched into the `palette.background` \
               binding — adjacent fields, copy-paste origin.",
        gold: 4,
    },
    FamilyText {
        text: "`args.output_path` is passed where `args.input_path` is read, so \
               the tool reads the very file it is writing.",
        gold: 4,
    },
    FamilyText {
        text: "`metrics.get(\"p95\")` is recorded into the latency_p99 column, \
               so the percentile markers swap places in every burndown chart \
               the capacity team reads.",
        gold: 4,
    },
    FamilyText {
        text: "`folios[reservation.room]` is charged where the folio belongs to \
               `reservation.guest_id`.",
        gold: 4,
    },
    FamilyText {
        text: "`banks[routing_nbr]` is resolved where the account table is keyed \
               through `acct_uuid`; identical routing numbers collapse distinct \
               branches.",
        gold: 4,
    },
    FamilyText {
        text: "`locales[\"en_US\"]` is pulled for an `en_GB` request because the \
               two constants sit beside each other in the source.",
        gold: 4,
    },
    FamilyText {
        text: "`schema.fields[\"created\"]` is mapped onto the `updated` column \
               during the migration copy.",
        gold: 4,
    },
    FamilyText {
        text: "`ports[name]` is consulted where the service registry keys on \
               `ports[alias]`; the canary answers on production's port.",
        gold: 4,
    },
    FamilyText {
        text: "`units[\"kg\"]` conversion is applied where the reading arrived \
               in `lb`, the pair sitting on a single row of that table.",
        gold: 4,
    },
    FamilyText {
        text: "`versions[stable]` is fetched for the rollout although \
               `versions[canary]` holds the candidate build; both slots \
               carry build ids.",
        gold: 4,
    },
    FamilyText {
        text: "`students[locker_nbr]` is pulled although the roster keys on \
               `student_id`; locker numbers repeat across grades.",
        gold: 4,
    },
    FamilyText {
        text: "`cache.primary.get(url)` is stored into the stale shelf where \
               the code meant `cache.fallback.get(url)`.",
        gold: 4,
    },
    FamilyText {
        text: "`grid[r][c]` is indexed as `grid[c][r]` inside the flood-fill's \
               boundary test while the rest of the crate goes row-major.",
        gold: 4,
    },
    // ── swallowed_error (5) × 17 ──
    FamilyText {
        text: "`let _ = wal.append(entry);` runs ahead of the ack; an append \
               that silently failed still tells the client the batch \
               committed.",
        gold: 5,
    },
    FamilyText {
        text: "`.ok();` follows `fs::remove_file(tmp)`; leftover temp files \
               accumulate silently whenever the handle is still open.",
        gold: 5,
    },
    FamilyText {
        text: "`if send(frame).is_err() { continue; }` runs with no log and no \
               counter; peers vanish from the mesh without a trace.",
        gold: 5,
    },
    FamilyText {
        text: "`match parse(raw) { Ok(v) => v, _ => Value::Null }` sits at the \
               ingest boundary; malformed rows become empty records instead of \
               flagging the batch for quarantine.",
        gold: 5,
    },
    FamilyText {
        text: "`catch_unwind(|| drive()).unwrap_or_default()` wraps the shard \
               loop; a panic mid-migration leaves the table half-moved and \
               reported healthy.",
        gold: 5,
    },
    FamilyText {
        text: "The writer ignores `flush()`'s outcome; the caller's durable- \
               write contract rests on a buffer that may never have reached \
               disk.",
        gold: 5,
    },
    FamilyText {
        text: "`spawn(worker).map(|_| ()).ok();` discards the join outcome; a \
               panicked indexer thread is indistinguishable from an idle \
               worker.",
        gold: 5,
    },
    FamilyText {
        text: "`let _ = self.journal.record(evt);` sits inside the replicator; \
               a full disk turns the node silently non-replicating.",
        gold: 5,
    },
    FamilyText {
        text: "`if let Ok(cfg) = read_config() { apply(cfg); }` — a torn or \
               unreadable file keeps the previous day's policy in force \
               unannounced.",
        gold: 5,
    },
    FamilyText {
        text: "`results.retain(|r| r.is_ok());` runs before the summary; the \
               failed fraction disappears from the report the SLO reads.",
        gold: 5,
    },
    FamilyText {
        text: "`Addr::from_str(listen).unwrap_or(([0, 0, 0, 0], 8080).into())` \
               swallows malformed bind specs and serves on the wildcard \
               instead.",
        gold: 5,
    },
    FamilyText {
        text: "Commit returns `Ok(())` after `inner.commit()`'s failure is \
               mapped to a warn-and-continue; the transaction boundary \
               lies.",
        gold: 5,
    },
    FamilyText {
        text: "`reader.lines().filter_map(Result::ok)` consumes the audit CSV; \
               corrupt lines exit the audit trail entirely.",
        gold: 5,
    },
    FamilyText {
        text: "`_ = lock.try_lock();` is the best-effort acquisition feeding a \
               critical section; the unprotected path is invisible in \
               production.",
        gold: 5,
    },
    FamilyText {
        text: "The health probe turns `ping()`'s failure into `false` and \
               clears the alert; a partitioned dependency reads as \
               recovered.",
        gold: 5,
    },
    FamilyText {
        text: "`let _ = file.set_len(0);` precedes rewriting the index; a \
               failed truncate leaves the old tail interpreted as fresh \
               entries.",
        gold: 5,
    },
    FamilyText {
        text: "`metrics.export().ok();` runs in the timer wheel; a failed push \
               leaves the interval's numbers unrecorded with the dashboards \
               still green.",
        gold: 5,
    },
];
