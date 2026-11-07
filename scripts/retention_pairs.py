#!/usr/bin/env python3
"""Retention pairs (rethink Issue 057 / Bench 089): per-case mammoth
ok(paraphrase) vs ok(original) joined from the 082 bmo dumps + the paraphrase
dump. Paired drop = ok_orig - ok_para; reports mean, SE, LB95
(Z95 = 1.959963984540054, the PairedDiff law).

Usage: retention_pairs.py [--orig-dir DIR] [--para-dir DIR] SUITE...
Defaults: --orig-dir /tmp/t102_items --para-dir /tmp/para_read/items (the
Bench 089 run posture); every open is utf-8 (the locale_io law)."""
import json, sys, math

Z95 = 1.959963984540054

def load_mammoth(path):
    m = {}
    with open(path, encoding='utf-8') as f:
        for line in f:
            d = json.loads(line)
            if d.get('lane') != 'mammoth':
                continue
            m[(d['case_idx'], d['q_idx'])] = d
    return m

def paired(drops):
    n = len(drops)
    mean = sum(drops) / n
    if n > 1:
        var = sum((x - mean) ** 2 for x in drops) / (n - 1)
        se = math.sqrt(var / n)
    else:
        se = 0.0
    return mean, se, mean - Z95 * se

argv = sys.argv[1:]
orig_dir = '/tmp/t102_items'
para_dir = '/tmp/para_read/items'
suites = []
i = 0
while i < len(argv):
    if argv[i] == '--orig-dir':
        orig_dir = argv[i + 1]
        i += 2
    elif argv[i] == '--para-dir':
        para_dir = argv[i + 1]
        i += 2
    else:
        suites.append(argv[i])
        i += 1

for suite in suites:
    ref = load_mammoth('%s/%s.jsonl' % (orig_dir, suite))
    new = load_mammoth('%s/%s.jsonl' % (para_dir, suite))
    common = sorted(set(ref) & set(new))
    assert len(common) == len(ref) == len(new), 'coverage mismatch: ref=%d new=%d common=%d' % (len(ref), len(new), len(common))
    gold_mismatch = [k for k in common if ref[k]['gold'] != new[k]['gold']]
    assert not gold_mismatch, 'gold mismatch at %s' % gold_mismatch[:3]
    drops = [int(ref[k]['ok']) - int(new[k]['ok']) for k in common]
    acc_ref = sum(int(ref[k]['ok']) for k in common) / len(common)
    acc_new = sum(int(new[k]['ok']) for k in common) / len(common)
    flips_to_wrong = sum(1 for k in common if ref[k]['ok'] and not new[k]['ok'])
    flips_to_right = sum(1 for k in common if new[k]['ok'] and not ref[k]['ok'])
    mean, se, lb95 = paired(drops)
    print('%s: n=%d  orig=%.4f para=%.4f  drop=%.4f se=%.4f LB95=%.4f  (wrong=%d right=%d)' % (
        suite, len(common), acc_ref, acc_new, mean, se, lb95, flips_to_wrong, flips_to_right))
