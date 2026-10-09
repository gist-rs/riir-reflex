#!/usr/bin/env python3
"""Throwaway timing probe (measurement only — NOT lane code, NOT committed).

Times the ORIGINAL laya reference (the pinned .raw/laya code, torch) over
the G5 fixture corpus on this M3, CPU and MPS, so the Rust port's numbers
have a same-box baseline. Mirrors the harness's posture: one system_one
call per row (the batch IS the row's question set), warmup first, then
timed passes.
"""
import json
import sys
import time

sys.path.insert(0, ".raw/laya")
import laya  # noqa: E402
# Keep this instrument's verdict printable on a non-UTF-8 console
# (katgpt-rs Issue 804 / the 928 drift census): it prints non-ASCII glyphs,
# and print() raises UnicodeEncodeError on e.g. cp874 — the process then dies
# with NO verdict. backslashreplace degrades the glyph visibly and keeps
# ASCII exact, so a verdict line stays greppable. Best-effort: a detached or
# captured stream is left alone rather than made fatal at import.
for _stream in (sys.stdout, sys.stderr):
    try:
        _stream.reconfigure(errors="backslashreplace")
    except (AttributeError, ValueError):
        pass

FIXTURE = "tests/fixtures/laya_parity_v1.jsonl"
SUBFOLDERS = {"english": "", "typed-decisions": "typed-decisions", "multilingual": "multilingual"}

rows = []
with open(FIXTURE, encoding="utf-8") as f:
    for line in f:
        d = json.loads(line)
        if d.get("id") == "_meta":
            continue
        rows.append(d)

# The fixture stores the INTERNAL question form (t/ins/crit) — the
# reference's to_internal output shape. system_one takes RAW definitions
# (type/instructions/criteria), so convert back 1:1.
def to_raw(q):
    r = {"type": q["t"], "instructions": q["ins"]}
    if q.get("crit") is not None:
        r["criteria"] = q["crit"]
    return r


def run(device: str, ckpt: str, reps: int = 3):
    sub = SUBFOLDERS[ckpt] or None
    agent = laya.load(device=device, subfolder=sub)
    # Warmup: every row once (weights transfers, allocator, kernels).
    for row in rows:
        if ckpt not in row["checkpoints"]:
            continue
        qs = {f"q{i}": to_raw(q) for i, q in enumerate(row["questions"])}
        agent.system_one(row["state"], qs)
    times = []
    n_q = 0
    for _ in range(reps):
        for row in rows:
            if ckpt not in row["checkpoints"]:
                continue
            qs = {f"q{i}": to_raw(q) for i, q in enumerate(row["questions"])}
            t0 = time.perf_counter()
            agent.system_one(row["state"], qs)
            times.append((time.perf_counter() - t0) * 1000.0)
            n_q += len(qs)
    times.sort()
    p50 = times[len(times) // 2]
    p90 = times[int(len(times) * 0.9)]
    mean_row_ms = sum(times) / len(times)
    qps = 1000.0 * len(times) * 1.0 / (sum(times)) * (n_q / max(1, len(times)))
    print(
        f"[orig torch:{device}] {ckpt}: rows={len(times)} reps={reps} "
        f"row_p50={p50:.1f}ms row_p90={p90:.1f}ms "
        f"ms/question={mean_row_ms / (n_q / len(times)):.1f} (n_q/row={n_q / len(times):.2f})",
        flush=True,
    )
    del agent


if __name__ == "__main__":
    device = sys.argv[1]
    ckpts = sys.argv[2].split(",") if len(sys.argv) > 2 else ["english"]
    for ck in ckpts:
        run(device, ck)
