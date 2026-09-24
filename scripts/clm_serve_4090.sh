#!/bin/bash
# Boot the CLM reference stack for the 4090 bench window (Issue 019 T3 /
# `.issues/027`). THEIR stack serves, our Rust measures — one docker
# container holding both halves:
#
#   * vLLM Qwen3-8B pooling server  (:8090) — their encoder, LAST-token
#     pooling + prefix cache (the exact flags of their
#     `serve_qwen3_8b.sh`, which the head's training precompute used);
#   * `clm-serve`                   (:8700) — their FastAPI System One
#     server over the trained head (`checkpoints/CLM_v0.1-8B.pt`).
#
# Host paths (both under the gitignored `.raw/`, never in the tree):
#   .raw/models/Qwen3-8B            the HF weights (fp16 safetensors)
#   .raw/CLM                        their repo @ cca045ff + the head
#
# ⛔ The 027 no-concurrency rule binds: run this ONLY on a quiet GPU (no
# perf-league cycle, no sibling compute job — vLLM at util 0.35 contends
# for VRAM and VRAM contention reads like a config-dependent failure, the
# Bench-649 class). Check `nvidia-smi` first.
#
# ⚠ GPU_UTIL on the 24 GB 4090: their script's 0.35 assumes a big card —
# the bf16 weights ALONE are 14.11 GiB (measured 2026-09-25), which
# busts a 0.35×24.5 GB = 8.6 GB budget with "No available memory for the
# cache blocks". The dedicated-window posture here is 0.72 (≈17.6 GB:
# weights + KV + eager overhead, ~6 GB card headroom). On an 80 GB card
# their 0.35 co-existence posture stands.
#
# Usage:  scripts/clm_serve_4090.sh start|stop|status|logs
# Env:    GPU_UTIL (0.72) · PORT_VLLM (8090) · PORT_CLM (8700)
set -eu

# Git Bash on Windows mangles colon-separated -v volume specs into
# `\Program Files\Git\...;C` — the docker calls below carry the
# conversion disables as COMMAND-PREFIX env (a no-op on Linux). They are
# deliberately NOT exported script-wide: `MSYS2_ARG_CONV_EXCL='*'` breaks
# `curl -o /dev/null` (rc 23, the write-error class) and the health waits
# would spin forever on a healthy server (measured 2026-09-25).
NOCV=(env MSYS_NO_PATHCONV=1 MSYS2_ARG_CONV_EXCL='*')

CT=clm-stack
IMAGE=vllm/vllm-openai:nightly
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODELS="$REPO_ROOT/.raw/models/Qwen3-8B"
CLM_SRC="$REPO_ROOT/.raw/CLM"
CKPT="$CLM_SRC/checkpoints/CLM_v0.1-8B.pt"
GPU_UTIL="${GPU_UTIL:-0.72}"
PORT_VLLM="${PORT_VLLM:-8090}"
PORT_CLM="${PORT_CLM:-8700}"

die() { echo "clm_serve_4090: $*" >&2; exit 1; }

need_paths() {
  [ -f "$MODELS/config.json" ] || die "no HF weights at $MODELS (download first: hf download Qwen/Qwen3-8B --local-dir '$MODELS')"
  [ -f "$CKPT" ] || die "no head at $CKPT (their download_head.sh / hf download Contrastive-LM/CLM-v0.1-8B CLM_v0.1-8B.pt)"
  [ -d "$CLM_SRC/src/clm" ] || die "no CLM repo at $CLM_SRC (clone github.com/Contrastive-LM/CLM @ cca045ff per .issues/019)"
}

wait_http() { # wait_http <url> <name> <timeout_s>
  local i=0
  until curl -sf -o /dev/null --max-time 2 "$1"; do
    i=$((i + 1))
    [ "$i" -gt "${3:-120}" ] && die "$2 never came up at $1 (docker logs $CT)"
    sleep 2
  done
}

create() {
  need_paths
  # One-time: the container (the CMD is the vLLM serve line — their
  # serve_qwen3_8b.sh flags verbatim, host ports mapped for the harness).
  "${NOCV[@]}" docker run -d --name "$CT" \
    --gpus all \
    -p "$PORT_VLLM:8090" -p "$PORT_CLM:8700" \
    -v "$MODELS":/models/Qwen3-8B:ro \
    -v "$CLM_SRC":/opt/CLM:ro \
    "$IMAGE" \
    --model /models/Qwen3-8B \
    --served-model-name qwen3-8b \
    --runner pooling \
    --enforce-eager \
    --enable-prefix-caching \
    --max-model-len 2048 \
    --gpu-memory-utilization "$GPU_UTIL" \
    --max-num-seqs 32 \
    --port 8090 \
    || die "docker run failed (image pulled? ports free?)"
  # One-time install: their package into the container's python (serve
  # extras only — torch/fastapi already in the vllm image; the head is
  # CPU-scale). The :ro mount means pip needs a copy — install from the
  # mounted source with --no-deps + the two serve deps explicitly.
  "${NOCV[@]}" docker exec "$CT" pip install --no-deps -q fastapi uvicorn 2>/dev/null || true
  "${NOCV[@]}" docker exec "$CT" sh -c "cp -r /opt/CLM /tmp/CLM && pip install --no-deps -q -e /tmp/CLM" \
    || die "pip install of their package failed"
}

start() {
  docker container inspect "$CT" >/dev/null 2>&1 || create
  docker start "$CT" >/dev/null 2>&1 || true
  echo "waiting for vLLM pooling server on :$PORT_VLLM (model load takes minutes)…"
  wait_http "http://127.0.0.1:$PORT_VLLM/health" "vLLM" 600
  # clm-serve as a second process in the SAME container — loopback to
  # vLLM, no docker network, --no-download (the head is mounted; a boot
  # that silently fetches weights is a provenance hole).
  "${NOCV[@]}" docker exec -d "$CT" clm-serve \
    --host 0.0.0.0 --port 8700 \
    --emb-url http://127.0.0.1:8090/v1/embeddings \
    --ckpt /opt/CLM/checkpoints/CLM_v0.1-8B.pt \
    --no-download
  wait_http "http://127.0.0.1:$PORT_CLM/health" "clm-serve" 60
  echo "CLM stack up:"
  echo "  encoder   http://127.0.0.1:$PORT_VLLM/v1/embeddings (qwen3-8b, runner=pooling)"
  echo "  systemone http://127.0.0.1:$PORT_CLM/v1/systemone"
  echo "harness:   CLM_SERVE_URL=http://127.0.0.1:$PORT_CLM cargo run --release --features clm-lane --bin harness -- --clm …"
}

stop() {
  docker exec "$CT" pkill -f clm-serve 2>/dev/null || true
  docker stop "$CT" >/dev/null 2>&1 || true
  echo "stopped (container kept — next start is warm)"
}

status() {
  docker ps -a --filter "name=$CT" --format '{{.Names}} {{.Status}}'
  for p in "$PORT_VLLM/health" "$PORT_CLM/health"; do
    if curl -sf -o /dev/null --max-time 2 "http://127.0.0.1:$p"; then
      echo "  :$p OK"
    else
      echo "  :$p DOWN"
    fi
  done
}

case "${1:-}" in
  start) start ;;
  stop) stop ;;
  status) status ;;
  logs) docker logs --tail "${2:-40}" "$CT" ;;
  *) die "usage: $0 start|stop|status|logs [n]" ;;
esac
