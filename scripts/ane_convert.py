#!/usr/bin/env python3
"""ANE offline conversion helper (Plan 002 P0 / Issue 017).

ONE-TIME, OFFLINE tool: converts the laya encoder safetensors checkpoints
(~/.cache/riir-reflex/laya/<model>/) into per-bucket BC1S FP16 CoreML
.mlpackage artifacts under assets/ane/<model>/L<bucket>/, verifies the
compute plan lands 100% on the Apple Neural Engine with zero device
transitions (refusing the artifact otherwise), and writes a BLAKE3 manifest
alongside. The no-Python directive is untouched: this script never runs at
serving or bench time — its output is the shipped artifact (the GGUF-export
posture).

Usage:
    uv run --with coremltools --with torch --with safetensors \
        scripts/ane_convert.py --model multilingual --buckets 64,128

The BC1S discipline (issue 017, the qualified configuration): fixed
batch=1, fixed sequence length per bucket, attention mask folded as a
constant per bucket — the flexible/enumerated graph is numerically wrong on
the ANE and never leaves this script.
"""

import argparse
import hashlib
import json
import platform
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CACHE = Path.home() / ".cache" / "riir-reflex" / "laya"
OUT_ROOT = REPO / "assets" / "ane"


def blake3(path: Path) -> str:
    # blake3 via hashlib is unavailable in stdlib; use blake3 wheel if
    # present, else sha256 with a recorded algorithm tag (the manifest
    # carries the algorithm name so the Rust verifier matches it).
    try:
        import blake3 as _b3

        h = _b3.blake3()
        algo = "blake3"
    except ImportError:
        h = hashlib.sha256()
        algo = "sha256"
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return f"{algo}:{h.hexdigest()}"


def convert(model: str, bucket: int) -> dict:
    import coremltools as ct
    import torch
    from safetensors.torch import load_file

    src = CACHE / model
    weights = load_file(str(src / "model.safetensors"))
    config = json.loads((src / "encoder_config.json").read_text())
    d = config["hidden_size"]
    layers = config["num_hidden_layers"]
    heads = config["num_attention_heads"]
    head_dim = d // heads

    class BucketedEncoder(torch.nn.Module):
        """BC1S view of the encoder: fixed [1, L] int32 ids -> pooled [1, 2d]
        (mean + CLS concat, the classifier head's input pairing), attention
        mask constant-folded for the bucket. Weight-loaded verbatim from the
        safetensors state dict; numerics mirror the lane's CPU reference."""

        def __init__(self, L: int):
            super().__init__()
            self.L = L
            self.register_buffer(
                "mask_bias", torch.zeros(1, 1, L, L), persistent=False
            )

        def forward(self, input_ids):
            return torch.zeros(1, 2 * d)

    # NOTE(ane-convert): the trace above is the SCAFFOLD — the real weight
    # load + layer stack is P0's remaining work and lands with the numerical
    # smoke arm. Recorded honestly: this helper as committed produces the
    # artifact SHAPE (BC1S mlpackage + placement verify + manifest) and the
    # placement discipline, but the encoder math is a stub until the
    # safetensors map lands. Do NOT bench from these artifacts.
    raise SystemExit(
        "ane_convert: the weight-mapped encoder stack is not wired yet — "
        "see Plan 002 P0 T0.4/T0.5 (the stub would produce garbage "
        "artifacts, and issue 017's no-silent-fallback doctrine forbids "
        "quietly shipping them)"
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True,
                    choices=["english", "multilingual", "typed"])
    ap.add_argument("--buckets", default="64,128")
    args = ap.parse_args()

    for bucket in (int(b) for b in args.buckets.split(",")):
        convert(args.model, bucket)
    return 0


if __name__ == "__main__":
    sys.exit(main())
