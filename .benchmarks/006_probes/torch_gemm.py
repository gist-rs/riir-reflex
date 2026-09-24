import time, torch, statistics, os
dev = torch.device("mps")
R = 24
for m in (92, 188, 283, 512):
    tot = 0.0
    row = []
    for (k, n, lab) in ((1024, 3072, "qkv"), (1024, 1024, "o"), (1024, 5248, "wi"), (2624, 1024, "wo")):
        a = torch.randn(m, k, device=dev); w = torch.randn(n, k, device=dev)
        for _ in range(3): torch.nn.functional.linear(a, w)
        torch.mps.synchronize()
        s = []
        for _ in range(5):
            t0 = time.perf_counter()
            for _ in range(R): y = torch.nn.functional.linear(a, w)
            torch.mps.synchronize()
            s.append((time.perf_counter() - t0) / R * 1e6)
        us = statistics.median(s); tot += us
        row.append(f"{lab}={us:.0f}us({2*m*k*n/us/1e6:.2f}TF)")
    print(f"torch m={m}: " + " ".join(row) + f" layer_sum={tot:.0f}us x28={tot*28/1000:.1f}ms load={os.getloadavg()[0]:.1f}")
