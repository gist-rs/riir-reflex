"""The CLM prose-rendering golden generator (Issue 019 T1) — OFFLINE one-time
helper (the `ane_convert.py` carve-out: never on the serving or runtime path,
no Python in the lane itself).

Re-clone github.com/Contrastive-LM/CLM at the pin recorded in
`src/lanes/clm.rs` (currently `cca045ff`), point CLM_SRC at it, run, and
re-pin the golden expectations in `src/lanes/clm.rs::tests` against the
output. A diff against the current goldens is a LAW CHANGE upstream —
adjudicate before re-pinning, never fold it in silently.

    CLM_SRC=/path/to/CLM python3 scripts/clm_goldens.py > /tmp/goldens.json
"""
import json
import os
import sys

CLM_SRC = os.environ.get(
    "CLM_SRC", "/Users/katopz/git/riir-reflex/.raw/CLM/src"
)
sys.path.insert(0, CLM_SRC)
from clm.schema import to_text, state_text, candidates, build_pairs  # noqa: E402

cases = {}


def rec(name, fn):
    cases[name] = fn()


# ── to_text: scalars ────────────────────────────────────────────────
rec("null", lambda: to_text(None))
rec("str", lambda: to_text("hello"))
rec("empty_str", lambda: to_text(""))
rec("bool_true", lambda: to_text(True))
rec("bool_false", lambda: to_text(False))
rec("int", lambda: to_text(42))
rec("int_neg", lambda: to_text(-7))
rec("float", lambda: to_text(3.14))
rec("float_integral", lambda: to_text(1.0))
rec("float_zero", lambda: to_text(0.0))
rec("float_neg_zero", lambda: to_text(-0.0))
rec("float_big", lambda: to_text(1e20))
rec("float_1e16", lambda: to_text(1e16))
rec("float_1e15", lambda: to_text(1e15))
rec("float_small", lambda: to_text(0.0001))
rec("float_smaller", lambda: to_text(0.00001))
rec("float_tiny", lambda: to_text(1.5e-7))
rec("float_pi_long", lambda: to_text(0.1 + 0.2))
# ── to_text: containers ─────────────────────────────────────────────
rec("empty_dict", lambda: to_text({}))
rec("empty_list", lambda: to_text([]))
rec("flat_obj", lambda: to_text({"a": 1, "b": "x"}))
rec("nested_obj", lambda: to_text({"user": {"name": "Bo", "tags": ["a", "b"]}, "n": 3}))
rec("list_of_objects", lambda: to_text([{"k": 1}, {"k": 2}]))
rec("empty_containers", lambda: to_text({"e": {}, "f": []}))
rec("unicode", lambda: to_text({"name": "café", "city": "กรุงเทพ"}))
rec("deep", lambda: to_text({"a": {"b": {"c": ["x", "y"]}}}))
rec("list_mixed", lambda: to_text(["plain", {"k": 1}, 42, None]))
rec("bool_in_obj", lambda: to_text({"ok": True, "n": False}))
rec("nested_list_in_list", lambda: to_text([["x", "y"], ["z"]]))
# ── state_text ──────────────────────────────────────────────────────
rec("state_both", lambda: state_text({"t": "ctx"}, "Is it fine?"))
rec("state_str_both", lambda: state_text("plain context", "Is it fine?"))
rec("state_empty_state", lambda: state_text(None, "Only the question"))
rec("state_empty_ins", lambda: state_text("Only the context", ""))
rec("state_both_empty", lambda: state_text("", ""))
rec("state_ws_strip", lambda: state_text("  padded context  \n", "\n  padded question  "))
# ── candidates ──────────────────────────────────────────────────────
rec("cand_noul_default", lambda: candidates({"type": "noul", "instructions": "Is the sky blue?"}))
rec("cand_noul_no_ins", lambda: candidates({"type": "noul"}))
rec("cand_noul_explicit", lambda: candidates({"type": "noul", "instructions": "ins here", "criteria": {"true": "confirmed yes"}}))
rec("cand_choice", lambda: candidates({"type": "choice", "instructions": "pick", "criteria": {"move": "walk left", "jump": "", "stay": None}}))
rec("cand_score", lambda: candidates({"type": "score", "instructions": "rate", "criteria": ["bad", "meh", "great"]}))
rec("cand_choice_opts", lambda: candidates({"type": "choice", "criteria": {"0": "auth", "1": "session", "2": "transport"}}))
# ── build_pairs composite ───────────────────────────────────────────
rec("pairs", lambda: build_pairs({"ticket": "4711", "sys": "safari"}, {
    "q1": {"type": "choice", "instructions": "Which subsystem?", "criteria": {"0": "auth", "1": "session"}},
    "q2": {"type": "noul", "instructions": "Reproducible?"},
}))

print(json.dumps(cases, ensure_ascii=False, indent=1))
