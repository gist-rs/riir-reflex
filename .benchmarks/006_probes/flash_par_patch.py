"""T10 rung 1: parallelize flash_attn's per-row softmax steps (1 simdgroup
= 1 row, 1 lane = 1 key) instead of 32 threads looping 32 keys serially."""
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()

old1 = '''        if (lid < 32u) {
            const uint q = q0 + lid;
            float m = st[lid];
            for (uint c = 0u; c < wk; ++c) {
                const uint k = t + c;
                const uint dk = (k > q) ? (k - q) : (q - k);
                if (dk <= window) { m = max(m, ts[lid * FTKS + c]); }
            }
            st[lid] = m;
        }'''
new1 = '''        // One simdgroup per row (sg = row), one lane per key: the tile's
        // [32][32] scores are exactly 32 simdgroups × 32 lanes, so the
        // row max is one simd_max instead of a 32-key serial loop on 32 of
        // the 1024 threads (Issue 020 T10). Max is order-independent —
        // bit-identical to the serial form.
        {
            const uint c = lid & 31u;
            const uint q = q0 + sg;
            const uint k = t + c;
            const uint dk = (k > q) ? (k - q) : (q - k);
            const float v = (c < wk && dk <= window) ? ts[sg * FTKS + c] : -3.402823466e+38f;
            const float mt = simd_max(v);
            if (c == 0u) { st[sg] = max(st[sg], mt); }
        }'''
assert s.count(old1) == 1, "old1"
s = s.replace(old1, new1)

old2 = '''        if (lid < 32u) {
            const float m = st[lid];
            const uint q = q0 + lid;
            float acc = 0.0f;
            for (uint c = 0u; c < wk; ++c) {
                const uint k = t + c;
                const uint dk = (k > q) ? (k - q) : (q - k);
                float p = 0.0f;
                if (dk <= window) { p = precise::exp(ts[lid * FTKS + c] - m); }
                ts[lid * FTKS + c] = p;
                acc += p;
            }
            l_reg += acc;
        }'''
new2 = '''        // Same row-per-simdgroup map: each lane exponentiates ONE score and
        // the row sum is a simd_sum. Every lane of simdgroup `sg` carries
        // row sg's running l. Padding keys (c ≥ wk) and out-of-window keys
        // write p = 0 — exact, as before (their V rows are staged zero and
        // l skips them). Only the l summation ORDER changes (tree vs
        // serial), inside the G5 drift budget.
        {
            const uint c = lid & 31u;
            const float m = st[sg];
            const uint q = q0 + sg;
            const uint k = t + c;
            const uint dk = (k > q) ? (k - q) : (q - k);
            float p = 0.0f;
            if (c < wk && dk <= window) { p = precise::exp(ts[sg * FTKS + c] - m); }
            ts[sg * FTKS + c] = p;
            l_reg += simd_sum(p);
        }'''
assert s.count(old2) == 1, "old2"
s = s.replace(old2, new2)

old3 = '''    // Publish the per-row l (each lane < 32 owns row `lid`'s register).
    if (lid < 32u) { st[32u + lid] = l_reg; }'''
new3 = '''    // Publish the per-row l (lane 0 of simdgroup `sg` owns row sg's).
    if ((lid & 31u) == 0u) { st[32u + sg] = l_reg; }'''
assert s.count(old3) == 1, "old3"
s = s.replace(old3, new3)
open(p, "w", encoding="utf-8").write(s)
print("patched")
