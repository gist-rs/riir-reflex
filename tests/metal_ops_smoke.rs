//! Metal-vs-Cpu op equivalence + dispatch-cost probes (`.issues/005` T4).
//! Runs only under `laya-riir-metal` on macOS — the [[test]] row keeps the
//! green-zero rule honest. The forward G5 gate (`laya_riir_parity`) is the
//! standing correctness authority; this file pins OP-level equivalence so
//! a bad kernel is diagnosable without re-deriving it from a red G5.

#![cfg(all(target_os = "macos", feature = "laya-riir-metal"))]

use riir_reflex::laya::riir::backend::{Backend, Cpu};
use riir_reflex::laya::riir::metal::Metal;

fn lcg() -> impl FnMut() -> f32 {
    let mut s = 0x1234_5678u32;
    move || {
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((s >> 8) as f32) / 8_388_608.0 - 1.0
    }
}

fn vec_of(n: usize) -> Vec<f32> {
    let mut r = lcg();
    (0..n).map(|_| r()).collect()
}

fn report(name: &str, a: &[f32], b: &[f32], tol: f32) {
    let max = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max);
    let scale = a
        .iter()
        .fold(0.0f32, |acc, v| acc.max(*v))
        .abs()
        .max(1e-9);
    println!("{name}: max abs {max:.4e} · scale {scale:.3e} · tol {tol:.1e}");
    assert!(max <= tol * scale.max(1e-3), "{name}: DIVERGED max {max:.4e}");
}

/// Sync a written slice's device result out (src = the written slice,
/// out = a fresh host buffer — the forward body's exact shape).
fn sync_out(m: &Metal, buf: &[f32]) -> Vec<f32> {
    let mut out = vec![0f32; buf.len()];
    m.download_into(buf, &mut out);
    out
}

#[test]
fn metal_ops_match_cpu_op_by_op() {
    let m = Metal::new().expect("metal backend");
    let c = Cpu;

    // GEMM — all three stride shapes at forward-like + ragged sizes.
    for (mm, k, n) in [(25usize, 768usize, 2304usize), (7, 64, 33), (1, 257, 129)] {
        let a = vec_of(mm * k);
        let b = vec_of(k * n);
        let mut dc = vec![0f32; mm * n];
        let mut dm = vec![0f32; mm * n];
        c.matmul(&a, 0, mm, k, &b, 0, n, &mut dc, 0);
        m.matmul(&a, 0, mm, k, &b, 0, n, &mut dm, 0);
        report(
            &format!("matmul {mm}x{k}x{n}"),
            &dc,
            &sync_out(&m, &dm),
            1e-3,
        );

        let w = vec_of(n * k);
        let mut wc = vec![0f32; mm * n];
        let mut wm = vec![0f32; mm * n];
        c.matmul_w(&a, mm, k, &w, n, &mut wc);
        m.matmul_w(&a, mm, k, &w, n, &mut wm);
        report(
            &format!("matmul_w {mm}x{k}x{n}"),
            &wc,
            &sync_out(&m, &wm),
            1e-3,
        );
    }
    {
        let (mm, hd) = (17usize, 64usize);
        let q = vec_of(mm * hd);
        let k = vec_of(mm * hd);
        let mut sc = vec![0f32; mm * mm];
        let mut sm = vec![0f32; mm * mm];
        c.matmul_kt(&q, 0, mm, hd, &k, 0, &mut sc, 0);
        m.matmul_kt(&q, 0, mm, hd, &k, 0, &mut sm, 0);
        report("matmul_kt", &sc, &sync_out(&m, &sm), 1e-3);
    }

    // Elementwise.
    let n = 1000;
    let x = vec_of(n);
    let y = vec_of(n);
    let mut xc = x.clone();
    let mut xm = x.clone();
    let xc_len = xc.len();
    c.add(&mut xc, 0, &y, 0, xc_len);
    let xm_len = xm.len();
    m.add(&mut xm, 0, &y, 0, xm_len);
    report("add", &xc, &sync_out(&m, &xm), 1e-6);

    let (d, rows) = (64usize, 15usize);
    let xb = vec_of(rows * d);
    let bias = vec_of(d);
    let mut xc = xb.clone();
    let mut xm = xb.clone();
    c.add_bias_row(&mut xc, d, &bias);
    m.add_bias_row(&mut xm, d, &bias);
    report("add_bias_row", &xc, &sync_out(&m, &xm), 1e-6);

    let mut xc = xb.clone();
    let mut xm = xb.clone();
    c.scale(&mut xc, 0.125);
    m.scale(&mut xm, 0.125);
    report("scale", &xc, &sync_out(&m, &xm), 1e-6);

    let mut xc = xb.clone();
    let mut xm = xb.clone();
    c.relu(&mut xc);
    m.relu(&mut xm);
    report("relu", &xc, &sync_out(&m, &xm), 1e-6);

    let mut xc = xb.clone();
    let mut xm = xb.clone();
    c.gelu_erf(&mut xc);
    m.gelu_erf(&mut xm);
    report("gelu_erf (libm vs A&S)", &xc, &sync_out(&m, &xm), 1e-4);

    let (r2, i_sz) = (5usize, 32usize);
    let fused = vec_of(r2 * 2 * i_sz);
    let mut oc = vec![0f32; r2 * i_sz];
    let mut om = vec![0f32; r2 * i_sz];
    c.glu_gelu_gate(&fused, r2, i_sz, &mut oc);
    m.glu_gelu_gate(&fused, r2, i_sz, &mut om);
    report("glu_gelu_gate", &oc, &sync_out(&m, &om), 1e-4);

    // LN + softmax (reduction order differs by design — loose tol).
    let w = vec_of(d);
    let mut oc = vec![0f32; rows * d];
    let mut om = vec![0f32; rows * d];
    let mut sq = Vec::new();
    c.layer_norm_nobias_into(&xb, &w, 1e-5, d, &mut sq, &mut oc);
    let mut sq2 = Vec::new();
    m.layer_norm_nobias_into(&xb, &w, 1e-5, d, &mut sq2, &mut om);
    report("layer_norm", &oc, &sync_out(&m, &om), 1e-3);

    let mut xc = xb.clone();
    let mut xm = xb.clone();
    c.softmax_rows(&mut xc, d);
    m.softmax_rows(&mut xm, d);
    report("softmax_rows", &xc, &sync_out(&m, &xm), 1e-3);

    // rope / split / merge / gather (exact-ish data movement).
    let (seq, heads, hd) = (9usize, 4usize, 64usize);
    let q = vec_of(heads * seq * hd);
    let (cos, sin) = riir_reflex::laya::riir::ops::rope_tables(seq, hd, 160_000.0);
    let mut qc = q.clone();
    let mut qm = q.clone();
    c.apply_rope(&mut qc, seq, heads, hd, &cos, &sin);
    m.apply_rope(&mut qm, seq, heads, hd, &cos, &sin);
    report("apply_rope", &qc, &sync_out(&m, &qm), 1e-4);

    let row_stride = 3 * 384;
    let src = vec_of(seq * row_stride);
    let mut oc = vec![0f32; heads * seq * hd];
    let mut om = vec![0f32; heads * seq * hd];
    c.split_heads(&src, row_stride, 384, seq, heads, hd, &mut oc);
    m.split_heads(&src, row_stride, 384, seq, heads, hd, &mut om);
    report("split_heads", &oc, &sync_out(&m, &om), 1e-6);

    let mut oc2 = vec![0f32; seq * heads * hd];
    let mut om2 = vec![0f32; seq * heads * hd];
    // merge consumes the SYNCED split output (om's own host bytes are a
    // stale handle under the lazy-sync contract — sync_out returns a copy).
    let om_synced = sync_out(&m, &om);
    c.merge_heads(&oc, seq, heads, hd, &mut oc2);
    m.merge_heads(&om_synced, seq, heads, hd, &mut om2);
    report("merge_heads", &oc2, &sync_out(&m, &om2), 1e-6);

    let markers = [3usize, 1, 7, 0];
    let mut oc3 = vec![0f32; markers.len() * d];
    let mut om3 = vec![0f32; markers.len() * d];
    c.gather_rows(&xb, d, &markers, &mut oc3);
    m.gather_rows(&xb, d, &markers, &mut om3);
    report("gather_rows", &oc3, &sync_out(&m, &om3), 1e-6);

    // download_into across syncs: the epoch bump must not lose the buffer.
    let a0 = vec_of(16); // m*k = 2*8
    let w0 = vec_of(16); // n*k = 2*8
    let mut dm = vec![0f32; 4]; // m*n
    m.matmul_w(&a0, 2, 8, &w0, 2, &mut dm);
    let dm_out = sync_out(&m, &dm); // sync 1
    let mut dm2 = vec![0f32; 4];
    m.matmul_w(&a0, 2, 8, &w0, 2, &mut dm2);
    let dm2_out = sync_out(&m, &dm2); // sync 2 — recycled dst slice
    assert_eq!(dm_out, dm2_out, "recycled dst slice diverged across syncs");
    println!("cross-sync download: ok");
}

#[test]
fn metal_batched_encode_cost() {
    let m = Metal::new().expect("metal backend");
    let mut x = vec![1.0f32; 100];
    for _ in 0..10 {
        m.scale(&mut x, 1.0);
    }
    // The batched shape: N encodes + ONE sync — the per-op cost the
    // forward actually pays.
    let t0 = std::time::Instant::now();
    const N: usize = 300;
    for _ in 0..N {
        m.scale(&mut x, 1.0);
    }
    let mut out = vec![0f32; 100];
    m.download_into(&x, &mut out);
    let per = t0.elapsed().as_secs_f64() * 1e3 / N as f64;
    println!("batched encode+1 sync: {per:.4} ms/op over {N} ops");
    assert!(out.iter().all(|v| v.is_finite()));
}

#[test]
fn metal_merge_heads_minimal() {
    let m = Metal::new().expect("metal backend");
    let c = Cpu;
    let (seq, heads, hd) = (9usize, 4usize, 64usize);
    // fresh src, no chained split
    let src = vec_of(heads * seq * hd);
    let mut oc2 = vec![0f32; seq * heads * hd];
    let mut om2 = vec![0f32; seq * heads * hd];
    c.merge_heads(&src, seq, heads, hd, &mut oc2);
    m.merge_heads(&src, seq, heads, hd, &mut om2);
    let got = sync_out(&m, &om2);
    report("merge_minimal", &oc2, &got, 1e-6);
    println!("cpu[0..4] = {:?}\nmetal[0..4] = {:?}", &oc2[..4], &got[..4]);
}

#[test]
fn metal_attention_chain_matches_cpu() {
    let m = Metal::new().expect("metal backend");
    let c = Cpu;
    let (seq, heads, hd) = (9usize, 4usize, 64usize);
    let d = heads * hd;
    let qkv = vec_of(seq * 3 * d);
    let mut q = vec![0f32; heads * seq * hd];
    let mut k = vec![0f32; heads * seq * hd];
    let mut v = vec![0f32; heads * seq * hd];
    let (cos, sin) = riir_reflex::laya::riir::ops::rope_tables(seq, hd, 160_000.0);
    let scale = (hd as f32).sqrt().recip();

    // metal chain (encodes only, one sync at the end)
    let mut qm = vec![0f32; heads * seq * hd];
    let mut km = vec![0f32; heads * seq * hd];
    let mut vm = vec![0f32; heads * seq * hd];
    let mut scoresm = vec![0f32; heads * seq * seq];
    let mut ctxm = vec![0f32; heads * seq * hd];
    let mut mergedm = vec![0f32; seq * d];
    m.split_heads(&qkv, 3 * d, 0, seq, heads, hd, &mut qm);
    m.split_heads(&qkv, 3 * d, d, seq, heads, hd, &mut km);
    m.split_heads(&qkv, 3 * d, 2 * d, seq, heads, hd, &mut vm);
    m.apply_rope(&mut qm, seq, heads, hd, &cos, &sin);
    m.apply_rope(&mut km, seq, heads, hd, &cos, &sin);
    m.scale(&mut qm, scale);
    for head in 0..heads {
        let qb = head * seq * hd;
        let sb = head * seq * seq;
        m.matmul_kt(&qm, qb, seq, hd, &km, qb, &mut scoresm, sb);
    }
    m.softmax_rows(&mut scoresm, seq);
    for head in 0..heads {
        let sb = head * seq * seq;
        let vb = head * seq * hd;
        m.matmul(&scoresm, sb, seq, seq, &vm, vb, hd, &mut ctxm, vb);
    }
    let scores_got = sync_out(&m, &scoresm);
    let infs = scores_got.iter().filter(|v| !v.is_finite()).count();
    let m2 = scores_got.iter().filter(|v| **v == f32::MIN).count();
    println!("metal scores: infs {infs} · f32::MIN count {m2} · max finite {:.4e}", scores_got.iter().filter(|v| v.is_finite()).fold(0.0f32, |a, b| a.max(*b)));
    m.merge_heads(&ctxm, seq, heads, hd, &mut mergedm);
    let merged_got = sync_out(&m, &mergedm);

    // cpu chain
    c.split_heads(&qkv, 3 * d, 0, seq, heads, hd, &mut q);
    c.split_heads(&qkv, 3 * d, d, seq, heads, hd, &mut k);
    c.split_heads(&qkv, 3 * d, 2 * d, seq, heads, hd, &mut v);
    c.apply_rope(&mut q, seq, heads, hd, &cos, &sin);
    c.apply_rope(&mut k, seq, heads, hd, &cos, &sin);
    c.scale(&mut q, scale);
    let mut scores = vec![0f32; heads * seq * seq];
    for head in 0..heads {
        let qb = head * seq * hd;
        let sb = head * seq * seq;
        c.matmul_kt(&q, qb, seq, hd, &k, qb, &mut scores, sb);
    }
    c.softmax_rows(&mut scores, seq);
    let mut ctx = vec![0f32; heads * seq * hd];
    for head in 0..heads {
        let sb = head * seq * seq;
        let vb = head * seq * hd;
        c.matmul(&scores, sb, seq, seq, &v, vb, hd, &mut ctx, vb);
    }
    let mut merged = vec![0f32; seq * d];
    c.merge_heads(&ctx, seq, heads, hd, &mut merged);

    report("attention chain (merged)", &merged, &merged_got, 1e-3);
}

/// Long-sequence sliding-window attention block: the mask add (offset
/// form) + full-size gemms — the english/multilingual shape that stayed
/// broken after the pass fix while typed-decisions went exact.
#[test]
fn metal_sliding_window_chain_matches_cpu() {
    let m = Metal::new().expect("metal backend");
    let c = Cpu;
    let (seq, heads, hd) = (512usize, 16usize, 64usize);
    let d = heads * hd;
    let qkv = vec_of(seq * 3 * d);
    let (cos, sin) = riir_reflex::laya::riir::ops::rope_tables(seq, hd, 160_000.0);
    let scale = (hd as f32).sqrt().recip();

    // the encoder's mask: [seq, seq], f32::MIN outside the window
    let window = 128usize;
    let mut mask = vec![f32::MIN; seq * seq];
    for qi in 0..seq {
        let lo = qi.saturating_sub(window);
        let hi = (qi + window).min(seq - 1);
        for kv in lo..=hi {
            mask[qi * seq + kv] = 0.0;
        }
    }

    let mut qm = vec![0f32; heads * seq * hd];
    let mut km = vec![0f32; heads * seq * hd];
    let mut vm = vec![0f32; heads * seq * hd];
    let mut scoresm = vec![0f32; heads * seq * seq];
    let mut ctxm = vec![0f32; heads * seq * hd];
    let mut mergedm = vec![0f32; seq * d];
    m.split_heads(&qkv, 3 * d, 0, seq, heads, hd, &mut qm);
    m.split_heads(&qkv, 3 * d, d, seq, heads, hd, &mut km);
    m.split_heads(&qkv, 3 * d, 2 * d, seq, heads, hd, &mut vm);
    m.apply_rope(&mut qm, seq, heads, hd, &cos, &sin);
    m.apply_rope(&mut km, seq, heads, hd, &cos, &sin);
    m.scale(&mut qm, scale);
    for head in 0..heads {
        let qb = head * seq * hd;
        let sb = head * seq * seq;
        m.matmul_kt(&qm, qb, seq, hd, &km, qb, &mut scoresm, sb);
    }
    // the sliding mask, offset form
    let mask_len = seq * seq;
    for head in 0..heads {
        let sb = head * seq * seq;
        m.add(&mut scoresm, sb, &mask, 0, mask_len);
    }
    m.softmax_rows(&mut scoresm, seq);
    for head in 0..heads {
        let sb = head * seq * seq;
        let vb = head * seq * hd;
        m.matmul(&scoresm, sb, seq, seq, &vm, vb, hd, &mut ctxm, vb);
    }
    m.merge_heads(&ctxm, seq, heads, hd, &mut mergedm);
    let merged_got = sync_out(&m, &mergedm);

    // cpu chain
    let mut q = vec![0f32; heads * seq * hd];
    let mut k = vec![0f32; heads * seq * hd];
    let mut v = vec![0f32; heads * seq * hd];
    c.split_heads(&qkv, 3 * d, 0, seq, heads, hd, &mut q);
    c.split_heads(&qkv, 3 * d, d, seq, heads, hd, &mut k);
    c.split_heads(&qkv, 3 * d, 2 * d, seq, heads, hd, &mut v);
    c.apply_rope(&mut q, seq, heads, hd, &cos, &sin);
    c.apply_rope(&mut k, seq, heads, hd, &cos, &sin);
    c.scale(&mut q, scale);
    let mut scores = vec![0f32; heads * seq * seq];
    for head in 0..heads {
        let qb = head * seq * hd;
        let sb = head * seq * seq;
        c.matmul_kt(&q, qb, seq, hd, &k, qb, &mut scores, sb);
    }
    for head in 0..heads {
        let sb = head * seq * seq;
        c.add(&mut scores, sb, &mask, 0, mask_len);
    }
    c.softmax_rows(&mut scores, seq);
    let mut ctx = vec![0f32; heads * seq * hd];
    for head in 0..heads {
        let sb = head * seq * seq;
        let vb = head * seq * hd;
        c.matmul(&scores, sb, seq, seq, &v, vb, hd, &mut ctx, vb);
    }
    let mut merged = vec![0f32; seq * d];
    c.merge_heads(&ctx, seq, heads, hd, &mut merged);

    report("sliding chain (merged)", &merged, &merged_got, 1e-3);
}
