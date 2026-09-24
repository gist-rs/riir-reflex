#!/usr/bin/env python3
"""ANE offline conversion + numerical smoke (Plan 002 P0 / Issue 017).

ONE-TIME, OFFLINE tool — never runs at serving or bench time (the no-Python
boundary; its output is the shipped artifact, the GGUF-export posture).

What it does:

  convert  Convert one checkpoint's encoder into per-bucket BC1S FP16 CoreML
           .mlpackage artifacts under assets/ane/<model>/L<bucket>.mlpackage,
           verify the compute plan lands 100% of device-bearing ops on the
           Apple Neural Engine with ZERO device transitions (refusing the
           artifact otherwise — issue 017's no-silent-fallback doctrine), and
           merge a BLAKE3 manifest entry into assets/ane/manifest.json.

  smoke    Numerical parity smoke (T0.5): 8 fixture prompts per model from the
           frozen G5 goldens (tests/fixtures/laya_parity_expected_v1.json),
           ANE encoder forward + an fp32 torch mirror of the Rust decision
           head (src/laya/riir/head.rs) -> probabilities vs the golden
           probabilities. Top-1 agreement + max prob err are OBSERVED data
           (expected 0.008-0.013 class for FP16 ANE math), never a gate;
           near-ties are listed row by row, never hidden.

The artifact is the ENCODER STACK: embeddings [1,L,d] fp16 + pad_bias
[1,1,1,L] fp16 -> hidden_state [1,L,d] fp16. The token->embedding gather is
HOST-side (a bit-exact fp16 row copy from the safetensors table — the Rust
runtime holds that table resident anyway), because the ANE compiler rejects
the vocab-sized gather op (measured: ios16.gather is the one op that will
not place on the ANE; everything after it places 100%). The decision head
stays in the Rust runtime unchanged (Plan 002 P1: "the EXISTING calibration
head unchanged") — the same split the smoke mirrors on the Python side.

BC1S discipline (issue 017, copied not approximated): fixed batch=1, one
artifact per length bucket, masked attention. The sliding-window mask is
CONSTANT-FOLDED per bucket (built only when window < L-1, the Rust lane's
own rule — at L64 the window covers the bucket and no mask exists); the
PADDING mask is an input tensor the host builds from n, so real positions
never attend to pad positions and the head can consume the first n rows.

Weights resolve exactly like src/laya/weights.rs: LAYA_WEIGHTS_DIR, then
LAYA_HOME, then ~/.cache/riir-reflex/laya; model.safetensors is verified
against the repo's SHA-256 pins (the HF LFS oids), encoder_config.json
against the blake3 pins; a missing file downloads from the hub (curl, to
.part then rename only after verify); a PRESENT file that fails its pin is
a hard error — never a silent re-download.

Usage:
  uv run --python 3.12 --with coremltools --with "torch==2.7.0" \
      --with safetensors --with numpy --with blake3 --with "tokenizers==0.22.0" \
      scripts/ane_convert.py convert --model multilingual --buckets 64,128
  uv run --python 3.12 --with coremltools --with "torch==2.7.0" \
      --with safetensors --with numpy --with blake3 --with "tokenizers==0.22.0" \
      scripts/ane_convert.py smoke --model multilingual

Python 3.12 is REQUIRED: coremltools 9.x ships its compiled CoreML bindings
(libcoremlpython — the compute-plan API this tool's T0.3 gate is built on)
only for <= 3.13; on newer interpreters the proxy silently fails to load
and placement verification would be blind. torch is pinned to 2.7.0 (the
newest version coremltools 9.0 tested against).

Recorded deviation (documented, not silent): the FP16 mask sentinel is
-1e4, not the Rust lane's f32::MIN — f32::MIN is not representable in
fp16. The softmax outcome is identical: masked weights underflow to exact
0, and an all-masked row (a pad position's query) reads max == sentinel,
exp(0) = 1 -> a uniform garbage row exactly like the f32 lane's.
"""

import argparse
import hashlib
import json
import math
import os
import platform
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
OUT_ROOT = REPO / "assets" / "ane"
EXPECTED = REPO / "tests" / "fixtures" / "laya_parity_expected_v1.json"
CORPUS = REPO / "tests" / "fixtures" / "laya_parity_v1.jsonl"
LOG = OUT_ROOT / "conversion_log.md"
MANIFEST = OUT_ROOT / "manifest.json"

MODELS = ["english", "multilingual", "typed"]

# ── the repo's weight pins (src/laya/weights.rs, mirrored verbatim) ──────

WEIGHT_SHA256 = {
    "english": "891102d372688fc2a094dac56a384bc537b87c63f21f9f3dac0be2b7cbc8d86c",
    "multilingual": "9d628fd971b700382ac6f65920a86f149777b2e748e0c955fb3b19695aa8f204",
    "typed": "4fa56de72383a9d3efa9cfa78955733c81b9fc8067a587ca4beb82c78107a24e",
}

SMALL_BLAKE3 = {
    ("english", "tokenizer.json"): "d8d4c8b30443535d6416e4f542d0a77e7f10e2809df063d11f7642a667037030",
    ("english", "tokenizer_config.json"): "8b275b6f13f464ceb2e6268c566ab3b40c8bfdcd6af7a5a299539194450083f0",
    ("english", "encoder_config.json"): "3b1e7e84b9d90a3835a55ed2a81395698e1d143f3553123c1e498649c7464098",
    ("english", "rl_agent_config.json"): "dbc03fdb75def9d9ea5218dd6df470d4bb5842584ea372bd9232ded658ed0f8f",
    ("multilingual", "tokenizer.json"): "01e0f0d31015fa48f99c5e7aea639fbf136181e632cfbf40156862aced7b20b1",
    ("multilingual", "tokenizer_config.json"): "8486197d7556f869092d214857a10fc9b75f8108250197ba08e7e9df8dba6637",
    ("multilingual", "encoder_config.json"): "a831925d30809ab5bb2ad5424eff016a422310a8989758c44631748276eee06a",
    ("multilingual", "rl_agent_config.json"): "dd2da6b7f43afd2babffa290bb1857784e49d825e6f76c5c0ab1d0fbdf441714",
    ("typed", "tokenizer.json"): "d8d4c8b30443535d6416e4f542d0a77e7f10e2809df063d11f7642a667037030",
    ("typed", "tokenizer_config.json"): "66b37468dfcfbc4b9022c2026d602bbd3dac39903fccd4361d81caa5e3ab4056",
    ("typed", "encoder_config.json"): "e724310ec3024cb84e2d7ad571f1ff20f2f3d4c1021b18cc43140a51a0a3a02e",
    ("typed", "rl_agent_config.json"): "4fbe491144bb1b3119a7ad308b6942523e85b9b35d554329b8d95d17e71e51d7",
}

HF_REPO = "convaiinnovations/laya"
HUB_PREFIX = {"english": "", "multilingual": "multilingual", "typed": "typed-decisions"}

# (local flat name, hub-relative path) — src/laya/weights.rs FILES verbatim.
FILES = [
    ("model.safetensors", "model.safetensors"),
    ("rl_agent_config.json", "rl_agent_config.json"),
    ("tokenizer.json", "tokenizer/tokenizer.json"),
    ("tokenizer_config.json", "tokenizer/tokenizer_config.json"),
    ("encoder_config.json", "encoder/config.json"),
]

MASK_SENTINEL = -10000.0  # the fp16-safe stand-in for f32::MIN (see docstring)


def weights_root() -> Path:
    if os.environ.get("LAYA_WEIGHTS_DIR"):
        return Path(os.environ["LAYA_WEIGHTS_DIR"])
    if os.environ.get("LAYA_HOME"):
        return Path(os.environ["LAYA_HOME"])
    return Path.home() / ".cache" / "riir-reflex" / "laya"


def hash_file(path: Path, kind: str) -> str:
    if kind == "blake3":
        import blake3

        h = blake3.blake3()
    elif kind == "sha256":
        h = hashlib.sha256()
    else:
        raise ValueError(kind)
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def blake3_dir(p: Path) -> dict:
    """Deterministic directory digest: blake3 over sorted (relpath, bytes)."""
    import blake3

    h = blake3.blake3()
    files = sorted(f for f in p.rglob("*") if f.is_file())
    total = 0
    for f in files:
        rel = str(f.relative_to(p)).encode()
        data = f.read_bytes()
        h.update(rel)
        h.update(b"\0")
        h.update(data)
        h.update(b"\0")
        total += len(data)
    return {"algo": "blake3-dir-v1", "digest": h.hexdigest(), "files": len(files), "bytes": total}


def ensure_checkpoint(model: str) -> dict:
    """Mirror src/laya/weights.rs ensure_checkpoint: locate -> verify ->
    (download). Returns {local name: resolved path}. A present-but-wrong
    file is a hard error, never a silent re-download."""
    root = weights_root()
    dirp = root / model
    dirp.mkdir(parents=True, exist_ok=True)
    resolved = {}
    for local_name, hub_rel in FILES:
        kind = "sha256" if local_name == "model.safetensors" else "blake3"
        if kind == "sha256":
            pin = WEIGHT_SHA256[model]
        else:
            pin = SMALL_BLAKE3[(model, local_name)]
        flat = dirp / local_name
        hub_shaped = dirp / hub_rel
        if flat.exists():
            got = hash_file(flat, kind)
            if got != pin:
                raise SystemExit(
                    f"ane_convert: {model}/{local_name} PRESENT but {kind} {got} != pinned {pin} "
                    "— a wrong local file is a fact to surface, not to paper over"
                )
            resolved[local_name] = flat
            continue
        if hub_shaped.exists():
            got = hash_file(hub_shaped, kind)
            if got != pin:
                raise SystemExit(
                    f"ane_convert: {model}/{hub_rel} PRESENT but {kind} {got} != pinned {pin}"
                )
            resolved[local_name] = hub_shaped
            continue
        prefix = HUB_PREFIX[model]
        base = f"https://huggingface.co/{HF_REPO}/resolve/main"
        url = f"{base}/{hub_rel}" if not prefix else f"{base}/{prefix}/{hub_rel}"
        tmp = flat.with_name(flat.name + ".part")
        print(f"ane_convert: downloading {url}")
        st = subprocess.run(
            ["curl", "--location", "--silent", "--show-error", "--fail", "--output", str(tmp), url]
        )
        if st.returncode != 0:
            tmp.unlink(missing_ok=True)
            raise SystemExit(f"ane_convert: download failed ({url})")
        got = hash_file(tmp, kind)
        if got != pin:
            tmp.unlink(missing_ok=True)
            raise SystemExit(f"ane_convert: downloaded {local_name} fails its pin ({got})")
        tmp.rename(flat)
        resolved[local_name] = flat
    return resolved


def box_state() -> dict:
    l1, l5, l15 = os.getloadavg()
    power = "unknown"
    try:
        out = subprocess.run(["pmset", "-g", "ps"], capture_output=True, text=True).stdout
        first = out.splitlines()
        if first:
            power = first[0].strip()
    except Exception:
        pass
    free_gb = None
    try:
        vm = subprocess.run(["vm_stat"], capture_output=True, text=True).stdout
        m = re.search(r"page size of (\d+) bytes", vm)

        def pages(label):
            mm = re.search(label + r":\s+(\d+)", vm)
            return int(mm.group(1)) if mm else 0

        if m:
            page = int(m.group(1))
            free = (pages("Pages free") + pages("Pages inactive") + pages("Pages speculative")) * page
            free_gb = round(free / 2**30, 1)
    except Exception:
        pass
    return {
        "machine": platform.machine(),
        "macos": platform.mac_ver()[0],
        "load1": round(l1, 2),
        "load5": round(l5, 2),
        "load15": round(l15, 2),
        "power": power,
        "free_ram_gb_approx": free_gb,
    }


# ── encoder config (mirrors src/laya/config.rs EncoderConfig::parse) ────


class EncoderCfg:
    def __init__(self, raw: dict, model: str):
        def num(key):
            v = raw.get(key)
            if not isinstance(v, (int, float)):
                raise SystemExit(f"ane_convert: {model} encoder_config missing numeric {key}")
            return float(v)

        self.hidden = int(num("hidden_size"))
        self.heads = int(num("num_attention_heads"))
        if self.hidden % self.heads != 0:
            raise SystemExit(f"ane_convert: hidden not divisible by heads")
        self.layers = int(num("num_hidden_layers"))
        self.intermediate = int(num("intermediate_size"))
        self.vocab = int(num("vocab_size"))
        self.eps = float(num("norm_eps"))
        self.window = int(num("local_attention")) // 2  # the Rust sliding_window()
        layer_types = raw.get("layer_types")
        if not isinstance(layer_types, list):
            raise SystemExit(f"ane_convert: {model} encoder_config missing layer_types")
        self.sliding = []
        for x in layer_types:
            if x == "full_attention":
                self.sliding.append(False)
            elif x == "sliding_attention":
                self.sliding.append(True)
            else:
                raise SystemExit(f"ane_convert: unknown layer_type {x!r}")
        if self.layers != len(self.sliding):
            raise SystemExit(f"ane_convert: num_hidden_layers != len(layer_types)")
        rope = raw.get("rope_parameters") or {}
        try:
            self.theta_full = float(rope["full_attention"]["rope_theta"])
            self.theta_slide = float(rope["sliding_attention"]["rope_theta"])
        except (KeyError, TypeError):
            raise SystemExit(f"ane_convert: {model} missing rope_parameters thetas")
        act = raw.get("hidden_activation")
        if act != "gelu":
            raise SystemExit(
                f"ane_convert: hidden_activation {act!r} unsupported — the lane compiles against gelu"
            )
        self.head_dim = self.hidden // self.heads

    def theta_for(self, sliding: bool) -> float:
        return self.theta_slide if sliding else self.theta_full


def parse_encoder_config(raw: dict, model: str) -> EncoderCfg:
    cfg = EncoderCfg(raw, model)
    return cfg


# ── rope tables + masks (mirror src/laya/riir/ops.rs bit-for-bit) ────────


def rope_tables_np(seq: int, hd: int, theta: float):
    """ops::rope_tables: inv is the f32 RECIPROCAL of the f32-cast f64 pow
    (`1.0 / (theta.powf(e) as f32)`), ang = pos_f32 * inv_f32, f32 cos/sin."""
    import numpy as np

    half = hd // 2
    j = np.arange(half, dtype=np.float64)
    p64 = np.float64(theta) ** ((2.0 * j) / float(hd))
    inv = np.float32(1.0) / p64.astype(np.float32)
    pos = np.arange(seq, dtype=np.float32)[:, None]
    ang = pos * inv[None, :].astype(np.float32)
    cos = np.cos(ang).astype(np.float32)
    sin = np.sin(ang).astype(np.float32)
    return cos, sin  # [seq, half]


def slide_bias_np(L: int, window: int):
    """The Rust lane's [seq, seq] additive window mask: 0.0 allowed inside
    |q-k| <= window, sentinel masked outside. Built only when
    window < L - 1 — the reference's own mask-skip rule."""
    import numpy as np

    q = np.arange(L, dtype=np.int64)[:, None]
    k = np.arange(L, dtype=np.int64)[None, :]
    allowed = np.abs(q - k) <= window
    m = np.where(allowed, np.float16(0.0), np.float16(MASK_SENTINEL))
    return m  # [L, L] fp16


# ── the traced encoder (mirrors src/laya/riir/encoder.rs forward) ────────


def build_encoder(cfg: EncoderCfg, W: dict, L: int):
    import torch
    import torch.nn.functional as F

    class BucketedEncoder(torch.nn.Module):
        """BC1S view of the ModernBERT encoder: fixed [1, L] int32 ids +
        [1,1,1,L] fp16 pad bias -> [1, L, d] fp16 last hidden state. Weight
        mapped verbatim from the safetensors map; numerics mirror the Rust
        lane (weight-only LayerNorms, no biases anywhere, rope-then-scale,
        GLU gelu erf)."""

        def __init__(self):
            super().__init__()
            d, H, hd, I = cfg.hidden, cfg.heads, cfg.head_dim, cfg.intermediate
            self.d, self.H, self.hd, self.I, self.L = d, H, hd, I, L
            self.eps = cfg.eps
            # fp16 scalar tensor: MIL layer_norm requires epsilon to match x's dtype
            self.eps_t = torch.tensor(cfg.eps, dtype=torch.float16)
            self.scale = float(1.0 / math.sqrt(hd))
            self.has_slide = cfg.window < L - 1
            self.register_buffer("emb_norm", W["encoder.embeddings.norm.weight"].half())
            for i in range(cfg.layers):
                if i > 0:
                    self.register_buffer(
                        f"attn_norm_{i}", W[f"encoder.layers.{i}.attn_norm.weight"].half()
                    )
                self.register_buffer(f"wqkv_{i}", W[f"encoder.layers.{i}.attn.Wqkv.weight"].half())
                self.register_buffer(f"wo_{i}", W[f"encoder.layers.{i}.attn.Wo.weight"].half())
                self.register_buffer(f"wi_{i}", W[f"encoder.layers.{i}.mlp.Wi.weight"].half())
                self.register_buffer(f"mlp_wo_{i}", W[f"encoder.layers.{i}.mlp.Wo.weight"].half())
                self.register_buffer(f"mlp_norm_{i}", W[f"encoder.layers.{i}.mlp_norm.weight"].half())
            self.register_buffer("final_norm", W["encoder.final_norm.weight"].half())
            for name, theta in (("full", cfg.theta_full), ("slide", cfg.theta_slide)):
                cos, sin = rope_tables_np(L, hd, theta)
                self.register_buffer(
                    f"cos_{name}", torch.from_numpy(cos).half()[None, None]
                )  # [1,1,L,half]
                self.register_buffer(f"sin_{name}", torch.from_numpy(sin).half()[None, None])
            if self.has_slide:
                sb = torch.from_numpy(slide_bias_np(L, cfg.window))
                self.register_buffer("slide_bias", sb[None, None])  # [1,1,L,L]

        def _ln(self, x, w):
            # Manual weight-only LayerNorm in fp16: F.layer_norm stores eps as
            # an fp32 scalar attribute, which fails MIL's dtype check against
            # fp16 x. Decomposed ops (mean/sub/mul/sqrt) are all ANE-native.
            mu = x.mean(dim=-1, keepdim=True)
            centered = x - mu
            var = (centered * centered).mean(dim=-1, keepdim=True)
            inv = 1.0 / torch.sqrt(var + self.eps_t)
            return centered * inv * w

        def forward(self, embeddings, pad_bias):
            h = embeddings  # [1, L, d] fp16 — host-side token gather
            h = self._ln(h, self.emb_norm)
            for i in range(len(cfg.sliding)):
                sliding = cfg.sliding[i]
                if i == 0:
                    x = h  # the layer-0 identity quirk
                else:
                    x = self._ln(h, getattr(self, f"attn_norm_{i}"))
                qkv = F.linear(x, getattr(self, f"wqkv_{i}"))  # [1, L, 3d]
                q = qkv[..., : self.d]
                k = qkv[..., self.d : 2 * self.d]
                v = qkv[..., 2 * self.d :]
                q = q.view(1, self.L, self.H, self.hd).permute(0, 2, 1, 3)
                k = k.view(1, self.L, self.H, self.hd).permute(0, 2, 1, 3)
                v = v.view(1, self.L, self.H, self.hd).permute(0, 2, 1, 3)
                cos = self.cos_slide if sliding else self.cos_full
                sin = self.sin_slide if sliding else self.sin_full
                half = self.hd // 2
                q1, q2 = q[..., :half], q[..., half:]
                q = torch.cat((q1 * cos - q2 * sin, q2 * cos + q1 * sin), dim=-1)
                k1, k2 = k[..., :half], k[..., half:]
                k = torch.cat((k1 * cos - k2 * sin, k2 * cos + k1 * sin), dim=-1)
                q = q * self.scale  # the lane's rope-then-scale order
                scores = torch.matmul(q, k.transpose(-1, -2))  # [1, H, L, L]
                scores = scores + pad_bias  # [1,1,1,L] broadcast
                if sliding and self.has_slide:
                    scores = scores + self.slide_bias
                probs = F.softmax(scores, dim=-1)
                ctx = torch.matmul(probs, v)  # [1, H, L, hd]
                merged = ctx.permute(0, 2, 1, 3).reshape(1, self.L, self.d)
                h = h + F.linear(merged, getattr(self, f"wo_{i}"))
                xn = self._ln(h, getattr(self, f"mlp_norm_{i}"))
                fused = F.linear(xn, getattr(self, f"wi_{i}"))  # [1, L, 2I]
                act = F.gelu(fused[..., : self.I], approximate="none") * fused[..., self.I :]
                h = h + F.linear(act, getattr(self, f"mlp_wo_{i}"))
            return self._ln(h, self.final_norm)

    return BucketedEncoder().eval()


# ── T0.3: the compute-plan placement gate ────────────────────────────────

DEVICE_SHORT = {
    "MLNeuralEngineComputeDevice": "ane",
    "MLGPUComputeDevice": "gpu",
    "MLCPUComputeDevice": "cpu",
}


def placement_table(compiled_path: str, compute_units) -> dict:
    import coremltools as ct
    from coremltools.models import compute_plan as cpmod

    plan = cpmod.MLComputePlan.load_from_path(compiled_path, compute_units=compute_units)
    fn = plan.model_structure.program.functions["main"]
    seq = []
    hist = {}
    for op in fn.block.operations:
        usage = plan.get_compute_device_usage_for_mlprogram_operation(op)
        if usage is None:
            continue  # const / identity — materialized weights, no device
        dev = DEVICE_SHORT.get(
            type(usage.preferred_compute_device).__name__,
            type(usage.preferred_compute_device).__name__,
        )
        name = op.operator_name
        seq.append((name, dev))
        hist.setdefault(name, {}).setdefault(dev, 0)
        hist[name][dev] += 1
    ane = sum(1 for _, d in seq if d == "ane")
    gpu = sum(1 for _, d in seq if d == "gpu")
    cpu = sum(1 for _, d in seq if d == "cpu")
    other = len(seq) - ane - gpu - cpu
    transitions = sum(1 for i in range(1, len(seq)) if seq[i][1] != seq[i - 1][1])
    return {
        "device_ops": len(seq),
        "ane_ops": ane,
        "gpu_ops": gpu,
        "cpu_ops": cpu,
        "other_ops": other,
        "transitions": transitions,
        "histogram": hist,
        "sequence": seq,
    }


def placement_summary(t: dict) -> str:
    return (
        f"device_ops={t['device_ops']} ane={t['ane_ops']} gpu={t['gpu_ops']} cpu={t['cpu_ops']} "
        f"transitions={t['transitions']}"
    )


def cmd_convert(args) -> int:
    import coremltools as ct
    import numpy as np
    import torch
    from safetensors.torch import load_file

    paths = ensure_checkpoint(args.model)
    cfg = parse_encoder_config(json.loads(Path(paths["encoder_config.json"]).read_text()), args.model)
    print(
        f"ane_convert: {args.model} geometry d={cfg.hidden} layers={cfg.layers} heads={cfg.heads} "
        f"hd={cfg.head_dim} I={cfg.intermediate} vocab={cfg.vocab} window={cfg.window} "
        f"sliding={sum(cfg.sliding)}/{cfg.layers}"
    )
    W = load_file(paths["model.safetensors"])
    tok_name = "encoder.embeddings.tok_embeddings.weight"
    if W[tok_name].shape[0] != cfg.vocab:
        raise SystemExit(
            f"ane_convert: tok_embeddings rows {W[tok_name].shape[0]} != config vocab {cfg.vocab}"
        )

    buckets = [int(b) for b in args.buckets.split(",")]
    for L in buckets:
        t0 = time.time()
        net = build_encoder(cfg, W, L)
        emb_ex = torch.zeros(1, L, cfg.hidden, dtype=torch.float16)
        pb_ex = torch.zeros(1, 1, 1, L, dtype=torch.float16)
        traced = torch.jit.trace(net, (emb_ex, pb_ex))
        out_dir = OUT_ROOT / args.model
        out_dir.mkdir(parents=True, exist_ok=True)
        art = out_dir / f"L{L}.mlpackage"
        if art.exists():
            shutil.rmtree(art)
        print(f"ane_convert: converting {args.model} L{L} (trace done {time.time() - t0:.1f}s)")
        mlmodel = ct.convert(
            traced,
            inputs=[
                ct.TensorType(name="embeddings", shape=(1, L, cfg.hidden), dtype=np.float16),
                ct.TensorType(name="pad_bias", shape=(1, 1, 1, L), dtype=np.float16),
            ],
            compute_precision=ct.precision.FLOAT16,
            convert_to="mlprogram",
            minimum_deployment_target=ct.target.macOS13,
        )
        mlmodel.save(str(art))
        out_name = mlmodel._spec.description.output[0].name
        print(f"ane_convert: saved {art} (output {out_name!r}, {time.time() - t0:.1f}s)")

        # T0.3 — placement verify on the COMPILED artifact. Gate at
        # CPU_AND_NE (the runtime posture: ANE-preferred, GPU excluded);
        # the ALL posture is recorded as observation beside it.
        loaded = ct.models.MLModel(str(art))
        compiled = loaded.get_compiled_model_path()
        gate = placement_table(compiled, ct.ComputeUnit.CPU_AND_NE)
        obs = placement_table(compiled, ct.ComputeUnit.ALL)
        print(f"ane_convert: placement CPU_AND_NE  {placement_summary(gate)}")
        print(f"ane_convert: placement ALL (obs)   {placement_summary(obs)}")
        ok = gate["ane_ops"] == gate["device_ops"] and gate["device_ops"] > 0 and gate["transitions"] == 0
        if not ok:
            bad = [(n, d) for n, d in gate["sequence"] if d != "ane"]
            print("ane_convert: OFF-ANE OPS (gate posture):", file=sys.stderr)
            for n, d in bad[:50]:
                print(f"    {d:4s} {n}", file=sys.stderr)
            if len(bad) > 50:
                print(f"    ... and {len(bad) - 50} more", file=sys.stderr)
            raise SystemExit(
                f"ane_convert: REFUSED {args.model} L{L} — placement {gate['ane_ops']}/"
                f"{gate['device_ops']} ANE, {gate['transitions']} transitions; "
                "issue 017 forbids shipping a partial-placement artifact"
            )

        entry = {
            "path": str(art.relative_to(REPO)),
            "bytes": art.stat().st_size,
            "digest": blake3_dir(art),
            "bucket_L": L,
            "inputs": {
                "embeddings": {"shape": [1, L, cfg.hidden], "dtype": "float16",
                               "note": "host-side token->embedding gather (bit-exact fp16 row "
                                       "copy); the vocab-sized gather op does not place on the ANE"},
                "pad_bias": {"shape": [1, 1, 1, L], "dtype": "float16",
                             "note": "0.0 = attend, -1e4 = masked; host-built from n"},
            },
            "outputs": {out_name: {"shape": [1, L, cfg.hidden], "dtype": "float16"}},
            "placement": {
                "gate": "cpu_and_ne",
                "ane_ops": gate["ane_ops"],
                "device_ops": gate["device_ops"],
                "transitions": gate["transitions"],
                "histogram": gate["histogram"],
                "all_units_observation": placement_summary(obs),
            },
            "geometry": {
                "hidden": cfg.hidden,
                "layers": cfg.layers,
                "heads": cfg.heads,
                "head_dim": cfg.head_dim,
                "intermediate": cfg.intermediate,
                "vocab": cfg.vocab,
                "norm_eps": cfg.eps,
                "window": cfg.window,
                "sliding_layers": sum(cfg.sliding),
                "rope_theta_full": cfg.theta_full,
                "rope_theta_slide": cfg.theta_slide,
                "slide_mask_folded": gate is not None and L > 1 and cfg.window < L - 1,
            },
            "weight_source": {"safetensors_sha256": WEIGHT_SHA256[args.model]},
            "mask_sentinel_fp16": MASK_SENTINEL,
            "coremltools": ct.__version__,
            "torch": torch.__version__,
            "created": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "box_state": box_state(),
        }
        merge_manifest(args.model, f"L{L}", entry)
        print(f"ane_convert: PASS {args.model} L{L} in {time.time() - t0:.1f}s — manifest updated")
    return 0


def merge_manifest(model: str, bucket_key: str, entry: dict):
    manifest = {}
    if MANIFEST.exists():
        manifest = json.loads(MANIFEST.read_text())
    meta = manifest.setdefault(
        "_meta",
        {
            "gate": "T0.3: 100% of device-bearing ops ANE-preferred under ComputeUnit.CPU_AND_NE, "
            "0 device transitions, on the compiling M3 — the tool refuses otherwise",
            "artifact_rule": "binaries local-only (gitignored); this manifest + the log are the "
            "committed record; release scope = download-on-demand, digest-pinned (issue 017)",
            "digest_algo": "blake3-dir-v1 (sorted relpath\\0 content\\0)",
        },
    )
    meta["tool"] = "scripts/ane_convert.py"
    meta.setdefault("history", [])
    line = f"{entry['created']} {model}/{bucket_key} ane={entry['placement']['ane_ops']}/{entry['placement']['device_ops']}"
    if line not in meta["history"]:
        meta["history"].append(line)
    manifest.setdefault("artifacts", {})[f"{model}/{bucket_key}"] = entry
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(json.dumps(manifest, indent=2, sort_keys=False) + "\n")


def log_append(text: str):
    LOG.parent.mkdir(parents=True, exist_ok=True)
    if not LOG.exists():
        LOG.write_text(
            "# ANE conversion log (Plan 002 P0)\n\n"
            "Appended by `scripts/ane_convert.py` per run. Artifacts themselves are "
            "local-only (gitignored); this log + `manifest.json` are the committed record.\n"
        )
    with open(LOG, "a") as f:
        f.write(text.rstrip() + "\n\n")


# ── T0.5: the numerical smoke — render/tokenize/head mirrors ─────────────


def load_tokenizer(paths):
    from tokenizers import Tokenizer

    tok = Tokenizer.from_file(str(paths["tokenizer.json"]))
    tcfg = json.loads(Path(paths["tokenizer_config.json"]).read_text())
    name_of = lambda key: tcfg[key]  # must be a plain string, like the Rust name_of

    def id_of(name):
        v = tok.token_to_id(name)
        if v is None:
            raise SystemExit(f"ane_convert: special token {name!r} not in vocab")
        return v

    return {
        "tok": tok,
        "cls": id_of(name_of("cls_token")),
        "sep": id_of(name_of("sep_token")),
        "mask": id_of(name_of("mask_token")),
        "pad": id_of(name_of("pad_token")),
        "mask_str": name_of("mask_token"),
    }


def to_internal(qdef: dict) -> dict:
    """Mirror tokenize::to_internal on the fixture's internal-ish row
    (t/ins/crit): choice list criteria become {opt: null} per element."""
    t = qdef["t"]
    qtype = {"choice": 0, "score": 1, "noul": 2}[t]
    crit = qdef.get("crit")
    if crit is None:
        crit = None
    if t == "choice" and isinstance(crit, list):
        crit = {k: None for k in crit}
    ins = qdef["ins"]
    if not isinstance(ins, str):
        ins = json.dumps(ins)  # ASCII-escaped, default separators — py_json(v, true)
    return {"t": t, "qtype": qtype, "ins": ins, "crit": crit}


def py_json_compact(v) -> str:
    return json.dumps(v, ensure_ascii=False, separators=(", ", ": "))


def render_criterion(v) -> str:
    if isinstance(v, str):
        return v
    return py_json_compact(v)


def is_none_or_empty(v) -> bool:
    return v is None or (isinstance(v, str) and v == "")


def render_options(q: dict):
    t, crit = q["t"], q["crit"]
    if t == "choice":
        out = []
        if isinstance(crit, dict):
            for k, v in crit.items():
                if is_none_or_empty(v):
                    out.append(k)
                else:
                    out.append(f"{k}: {render_criterion(v)}")
        return out
    if t == "score":
        crit_list = crit if isinstance(crit, list) else []
        return [f"level {i}: {render_criterion(c)}" for i, c in enumerate(crit_list)]
    # noul — [false, true] always
    def get(side):
        return crit.get(side) if isinstance(crit, dict) else None

    def render_side(name, v, default):
        body = render_criterion(v) if not is_none_or_empty(v) else default
        return f"{name}: {body}"

    return [
        render_side("false", get("false"), "no, the statement does not hold"),
        render_side("true", get("true"), "yes, the statement holds"),
    ]


def serialize_state(state) -> str:
    if isinstance(state, str):
        return state
    return py_json_compact(state)


def build_sequence(spec, state, q: dict, max_len: int, head_max_len: int):
    """Line-faithful port of tokenize::build_sequence. Returns (ids, markers)."""
    tok, S = spec["tok"], spec
    opts = render_options(q)
    ins = q["ins"].replace(S["mask_str"], " ")
    head_ids = tok.encode(f"{q['t']} question: {ins}", add_special_tokens=False).ids

    opt_ids = []
    for opt in opts:
        text = opt.replace(S["mask_str"], " ")
        ids = tok.encode(f" {text}", add_special_tokens=False).ids[:48]
        opt_ids.append([S["mask"]] + ids)

    total = sum(len(o) for o in opt_ids)
    opt_budget = max(0, head_max_len - total)
    if opt_budget < 16:
        per = max(4, (head_max_len - 16) // max(1, len(opt_ids)))
        opt_ids = [o[:per] for o in opt_ids]
        opt_budget = max(0, head_max_len - sum(len(o) for o in opt_ids))

    head_keep = max(8, opt_budget)
    ids = [S["cls"]] + head_ids[:head_keep] + [S["sep"]]

    markers = []
    for o in opt_ids:
        markers.append(len(ids))
        ids.extend(o)
    ids.append(S["sep"])

    room = max(0, max_len - (len(ids) + 1))
    state_str = serialize_state(state).replace(S["mask_str"], " ")
    st = tok.encode(state_str, add_special_tokens=False).ids[:room]

    ids.extend(st)
    ids.append(S["sep"])
    ids = ids[:max_len]
    markers = [m for m in markers if m < max_len]
    return ids, markers


def softmax32(z):
    import numpy as np

    z = np.asarray(z, dtype=np.float32)
    e = np.exp(z - z.max()).astype(np.float32)
    return (e / e.sum()).astype(np.float32)


def temp_bucket(qtype: int, k: int) -> str:
    name = ["choice", "score", "noul"][qtype]
    size = "2" if k <= 2 else "3-5" if k <= 5 else "6-10" if k <= 10 else "11+"
    return f"{name}:{size}"


class Temps:
    """temps::Temperatures — clamp at load, bucket table first."""

    def __init__(self, agent_cfg: dict):
        self.base = [clamp_temperature(t) for t in agent_cfg["temperature"]]
        self.by = {k: clamp_temperature(v) for k, v in agent_cfg.get("temperature_by_options", {}).items()}

    def for_question(self, qtype: int, k: int) -> float:
        return self.by.get(temp_bucket(qtype, k), self.base[qtype])


def clamp_temperature(t: float) -> float:
    if not math.isfinite(t):
        return 1.0
    return min(max(t, 0.5), 5.0)


def head_forward(W: dict, h: "np.ndarray", qtype: int, markers, eps: float, d: int):
    """fp32 torch mirror of src/laya/riir/head.rs forward. `h` is [n, d] f32
    (the ANE encoder's first n rows). Returns (logits [k], act_probs [2])."""
    import numpy as np
    import torch
    import torch.nn.functional as F

    n = h.shape[0]
    heads, hd = d // 64, 64
    scale = float(1.0 / math.sqrt(hd))
    f32 = lambda name: torch.from_numpy(np.asarray(W[name])).float()

    x = torch.from_numpy(h.copy()).float() + f32("type_emb.weight")[qtype]
    for li in range(2):
        p = f"head.layers.{li}."
        nx = F.layer_norm(x, (d,), f32(p + "norm1.weight"), f32(p + "norm1.bias"), eps)
        qkv = F.linear(nx, f32(p + "self_attn.in_proj_weight")) + f32(p + "self_attn.in_proj_bias")
        q = qkv[:, :d].view(n, heads, hd).transpose(0, 1)
        k = qkv[:, d : 2 * d].view(n, heads, hd).transpose(0, 1)
        v = qkv[:, 2 * d :].view(n, heads, hd).transpose(0, 1)
        q = q * scale  # torch need_weights path: q scaled BEFORE the matmul
        scores = torch.matmul(q, k.transpose(-1, -2))
        probs = F.softmax(scores, dim=-1)
        ctx = torch.matmul(probs, v)
        merged = ctx.transpose(0, 1).reshape(n, d)
        attn_out = F.linear(merged, f32(p + "self_attn.out_proj.weight")) + f32(
            p + "self_attn.out_proj.bias"
        )
        x = x + attn_out
        nx2 = F.layer_norm(x, (d,), f32(p + "norm2.weight"), f32(p + "norm2.bias"), eps)
        ff = F.relu(F.linear(nx2, f32(p + "linear1.weight")) + f32(p + "linear1.bias"))
        x = x + F.linear(ff, f32(p + "linear2.weight")) + f32(p + "linear2.bias")

    rows = x[markers]
    s = F.layer_norm(rows, (d,), f32("scorer.0.weight"), f32("scorer.0.bias"), eps)
    s1 = F.gelu(F.linear(s, f32("scorer.1.weight")) + f32("scorer.1.bias"), approximate="none")
    logits = (F.linear(s1, f32("scorer.3.weight")) + f32("scorer.3.bias")).squeeze(-1)

    p = softmax32(logits.detach().numpy())
    k_opts = len(markers)
    k_eff = float(max(k_opts, 2))
    ent = 0.0
    for pi in p:
        cl = max(float(pi), 1e-9)
        ent -= cl * math.log(cl)
    ent /= math.log(k_eff)
    srt = sorted((float(v) for v in p), reverse=True)
    top1, top2 = srt[0], srt[1] if len(srt) > 1 else 0.0
    feats = np.array(
        [
            np.float32(top1),
            np.float32(top1 - top2),
            np.float32(ent),
            np.float32(k_eff) / np.float32(255.0),
        ],
        dtype=np.float32,
    )
    cls = x[0].detach().numpy()
    act_in = np.concatenate([cls, feats]).astype(np.float32)
    a = F.gelu(
        torch.from_numpy(act_in) @ f32("act_head.0.weight").T + f32("act_head.0.bias"),
        approximate="none",
    )
    act_logits = a @ f32("act_head.2.weight").T + f32("act_head.2.bias")
    act_probs = softmax32(act_logits.detach().numpy())
    return logits.detach().numpy(), act_probs


def fixture_name(model: str) -> str:
    return {"english": "english", "multilingual": "multilingual", "typed": "typed-decisions"}[model]


def cmd_smoke(args) -> int:
    import coremltools as ct
    import numpy as np
    import torch
    from safetensors.torch import load_file

    model = args.model
    paths = ensure_checkpoint(model)
    exp_all = json.loads(EXPECTED.read_text())
    corpus_blake3 = exp_all["_meta"]["fixture_corpus_blake3"]
    got = hash_file(CORPUS, "blake3")
    if got != corpus_blake3:
        raise SystemExit(f"ane_convert: fixture corpus {got} != expected-file pin {corpus_blake3}")
    exp = exp_all["checkpoints"][fixture_name(model)]

    corpus = {}
    for line in CORPUS.read_text().splitlines():
        if line.strip():
            row = json.loads(line)
            corpus[row["id"]] = row

    spec = load_tokenizer(paths)
    agent_cfg = json.loads(Path(paths["rl_agent_config.json"]).read_text())
    max_len, head_max_len = agent_cfg["max_len"], agent_cfg["head_max_len"]
    temps = Temps(agent_cfg)
    cfg = parse_encoder_config(json.loads(Path(paths["encoder_config.json"]).read_text()), model)

    # Deterministic selection: the first `half` golden rows with n <= 64
    # (file order) + the first `half` with 64 < n <= 128 — both buckets
    # exercised, padding path included.
    want = args.fixtures
    half = want // 2
    lo = [k for k, v in exp.items() if v["seq_len"] <= 64][:half]
    hi = [k for k, v in exp.items() if 64 < v["seq_len"] <= 128][: want - len(lo)]
    sel = lo + hi
    if len(sel) < want:
        spare = [k for k, v in exp.items() if v["seq_len"] <= 128 and k not in sel]
        sel += spare[: want - len(sel)]
    if len(sel) != want:
        raise SystemExit(f"ane_convert: only {len(sel)} golden rows fit the buckets (want {want})")

    W = load_file(paths["model.safetensors"])
    emb_table = W["encoder.embeddings.tok_embeddings.weight"].numpy()  # fp16 [vocab, d]
    mlmodels = {}

    def ane_encode(L: int, ids):
        if L not in mlmodels:
            art = OUT_ROOT / model / f"L{L}.mlpackage"
            if not art.exists():
                raise SystemExit(f"ane_convert: artifact missing: {art} — run `convert` first")
            mlmodels[L] = ct.models.MLModel(str(art), compute_units=ct.ComputeUnit.CPU_AND_NE)
        x = list(ids) + [spec["pad"]] * (L - len(ids))
        idx = np.asarray(x, dtype=np.int64)
        emb = emb_table[idx].astype(np.float16)[None]  # [1, L, d] fp16 host gather
        pb = np.zeros((1, 1, 1, L), dtype=np.float16)
        pb[0, 0, 0, len(ids) :] = np.float16(MASK_SENTINEL)
        out = mlmodels[L].predict({"embeddings": emb, "pad_bias": pb})
        arr = out["hidden_state"] if "hidden_state" in out else next(iter(out.values()))
        return np.asarray(arr, dtype=np.float32)[0]  # [L, d]

    agree = 0
    max_err = 0.0
    max_act_err = 0.0
    flips = []
    rows_out = []
    for key in sel:
        row_id, qi_s = key.split("#q")
        row = corpus[row_id]
        qdef = row["questions"][int(qi_s)]
        q = to_internal(qdef)
        ids, markers = build_sequence(spec, row["state"], q, max_len, head_max_len)
        n = len(ids)
        e = exp[key]

        # Structural parity FIRST — the render/tokenize port must reproduce
        # the goldens byte-identically before any number is compared.
        if markers != e["markers"]:
            raise SystemExit(f"ane_convert: {model}/{key} markers {markers} != golden {e['markers']}")
        if n != e["seq_len"]:
            raise SystemExit(f"ane_convert: {model}/{key} seq_len {n} != golden {e['seq_len']}")
        bucket_str = temp_bucket(q["qtype"], len(markers))
        if bucket_str != e["bucket"]:
            raise SystemExit(f"ane_convert: {model}/{key} bucket {bucket_str} != golden {e['bucket']}")
        t = temps.for_question(q["qtype"], len(markers))
        if abs(t - e["temperature_used"]) > 1e-6:
            raise SystemExit(
                f"ane_convert: {model}/{key} temperature {t} != golden {e['temperature_used']}"
            )

        L = 64 if n <= 64 else 128
        hidden = ane_encode(L, ids)
        logits, act_probs = head_forward(W, hidden[:n], q["qtype"], markers, cfg.eps, cfg.hidden)
        z = (logits / np.float32(t)).astype(np.float32)
        probs = softmax32(z)

        exp_probs = np.asarray(e["probs"], dtype=np.float64)
        my = probs.astype(np.float64)
        err = float(np.max(np.abs(my - exp_probs)))
        my_arg = int(np.argmax(my))
        exp_arg = int(np.argmax(exp_probs))
        srt = np.sort(exp_probs)[::-1]
        margin = float(srt[0] - srt[1]) if len(srt) > 1 else 1.0
        ok = my_arg == exp_arg
        if ok:
            agree += 1
        elif margin < 2 * 0.02:  # issue 017's near-tie band (tolerance 0.02)
            flips.append((key, margin, my_arg, exp_arg, err))
        else:
            flips.append((key, margin, my_arg, exp_arg, err))
        max_err = max(max_err, err)
        a_err = float(
            np.max(np.abs(act_probs.astype(np.float64) - np.asarray(e["act_probabilities"])))
        )
        max_act_err = max(max_act_err, a_err)
        rows_out.append((key, n, L, len(markers), bucket_str, exp_arg, my_arg, margin, err))
        flag = "ok " if ok else "FLIP"
        print(
            f"  {flag} {key:34s} n={n:3d} L{L:<4d} k={len(markers):2d} {bucket_str:12s} "
            f"arg {exp_arg}->{my_arg} margin={margin:.4f} max_prob_err={err:.4f}"
        )

    print(
        f"ane_convert: smoke {model}: top-1 agreement {agree}/{len(sel)}, "
        f"max prob err {max_err:.4f}, max act err {max_act_err:.4f}"
    )
    for key, margin, my_arg, exp_arg, err in flips:
        kind = "near-tie flip" if margin < 2 * 0.02 else "HARD MISMATCH"
        print(
            f"  {kind}: {key} golden_margin={margin:.4f} arg {exp_arg}->{my_arg} (err {err:.4f})"
        )

    log_append(
        f"## Smoke — {model} — {time.strftime('%Y-%m-%d %H:%M')} {time.strftime('%Z')}\n"
        f"- fixtures: {', '.join(sel)}\n"
        f"- reference: frozen G5 goldens (`tests/fixtures/laya_parity_expected_v1.json`, "
        f"corpus blake3 {corpus_blake3[:16]}…); head = fp32 torch mirror of "
        "`src/laya/riir/head.rs`; encoder = the ANE artifact\n"
        f"- structural parity (markers/seq_len/bucket/temperature): exact on all {len(sel)}\n"
        f"- top-1 agreement: **{agree}/{len(sel)}** · max prob err **{max_err:.4f}** · "
        f"max act err {max_act_err:.4f}\n"
        + "\n".join(
            "  - " + ("ok" if ma == ea else ("near-tie flip" if m < 0.04 else "HARD MISMATCH"))
            + f" `{k}` n={n} L{L} k={kk} {b} arg {ea}->{ma} margin={m:.4f} err={err:.4f}"
            for k, n, L, kk, b, ea, ma, m, err in rows_out
        )
        + f"\n- near-tie band: golden margin < 0.04 (issue 017 tolerance 0.02 × 2); "
        f"flips listed above, never hidden\n"
        f"- box: {json.dumps(box_state())}"
    )
    print(f"ane_convert: smoke record appended to {LOG}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    sub = ap.add_subparsers(dest="cmd", required=True)

    c = sub.add_parser("convert", help="convert one model to per-bucket BC1S artifacts")
    c.add_argument("--model", required=True, choices=MODELS)
    c.add_argument("--buckets", default="64,128")
    c.set_defaults(fn=cmd_convert)

    s = sub.add_parser("smoke", help="numerical parity smoke vs the frozen goldens")
    s.add_argument("--model", required=True, choices=MODELS)
    s.add_argument("--fixtures", type=int, default=8)
    s.set_defaults(fn=cmd_smoke)

    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
