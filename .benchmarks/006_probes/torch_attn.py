import time, torch, statistics, os
F = torch.nn.functional
dev = torch.device("mps"); R = 24
for S in (188, 283, 370, 461, 512):
    q, k, v = (torch.randn(1, 16, S, 64, device=dev) for _ in range(3))
    idx = torch.arange(S, device=dev)
    win = (idx[None, :] - idx[:, None]).abs() <= 64
    mask = torch.zeros(S, S, device=dev).masked_fill(~win, float("-inf"))[None, None]
    out = {}
    for lab, m in (("full", None), ("masked", mask)):
        for _ in range(3): F.scaled_dot_product_attention(q, k, v, attn_mask=m)
        torch.mps.synchronize(); s = []
        for _ in range(5):
            t0 = time.perf_counter()
            for _ in range(R): F.scaled_dot_product_attention(q, k, v, attn_mask=m)
            torch.mps.synchronize(); s.append((time.perf_counter() - t0) / R * 1e3)
        out[lab] = statistics.median(s)
    fl = 4 * S * S * 1024
    print(f"torch S={S}: full={out['full']:.3f}ms ({fl/out['full']/1e9:.2f}TF) masked={out['masked']:.3f}ms "
          f"model(10 full+18 masked)={10*out['full']+18*out['masked']:.1f}ms load={os.getloadavg()[0]:.1f}")
