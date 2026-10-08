# Benchmark dataset manifest — Plan 603 T1.5 fetch layer

Raw row snapshots for the riir-reflex benchmark harness (G1/G2 decision
benches). Fetched from the HF datasets-server `/rows` API (page size 100,
offset paging) by [`scripts/fetch_datasets.sh`](../../scripts/fetch_datasets.sh)
on 2026-09-22 into `.raw/datasets/<suite>/<split>-<NNN>.json` — one file per
100-row page; each page file is the raw API response (it carries `features`,
`rows`, `num_rows_total`, so every page is self-describing). Companion probes
(`/splits`, `/size`) land beside the pages as `splits.json` / `size.json`.

- **Digest tool:** blake3 via `b3sum` (present on PATH; the sha256
  `shasum -a 256` fallback in the fetch protocol was not needed). Full
  digests below, computed over the exact bytes on disk.
- **Rows column:** number of rows in that page file. `—` marks a probe file
  (no `rows` array).
- **Caps:** per the fetch list — `test` splits generally capped (400/600/500/
  300/1000), `train` splits capped at 4000 (typed_decisions train 1200);
  `cap=all` suites page until the API returns fewer than 100 rows.

## Licences (the ledger of record — born riir-rethink Issue 023 T1; 023 closed 2026-10-07, record: riir-rethink HISTORY.md)

Verified 2026-10-04 against the HF dataset API + the source LICENSE files
/cards (human reads, not hub-tag echoes — `license:other` and absent tags
are resolved by the file/card, never guessed). The fetch script's
`license.json` probe (the hub metadata tripwire) re-surfaces the hub-side
signal at every refetch; THIS table stays authoritative.

**Paid-lane law:** a suite marked ⛔ cannot back a paid lane — no served or
leased artifact trained on it, benchmark/record-only use only (research
use is fine). ⚠ rows must be cleared (or the suite demoted likewise)
before ANY paid mint names them; the mint-side refusal gate lands with
the mint surface (riir-rethink Issue 024 v1 — the Plan 007 P0 hosted lane;
023's tracker role folded there at close).

| Suite | Source | Licence (verified) | Attribution | Paid lane |
|---|---|---|---|---|
| typed_decisions | `LocalLLaMA/typed-decisions` | Apache-2.0 (first-party synthetic — latent factors + our teacher labels, no scraped content; card re-verified 2026-10-07) | the dataset card | ✓ |
| massive_intent_en | `mteb/amazon_massive_intent` (mirror; upstream `alexa/massive`) | **CC BY 4.0** (the DATA's licence — Amazon NOTICE.md verified 2026-10-07: "The MASSIVE dataset is licensed under CC BY 4.0"; SLURP seed text also CC BY 4.0; the mteb mirror's Apache-2.0 tag is the CODE repo's licence, not the data's) | the MASSIVE citation (FitzGerald et al., 2022) | ✓ |
| banking77 | `mteb/banking77` (mirror of PolyAI) | **CC BY 4.0** (PolyAI's own — the upstream licence, PolyAI card §Licensing "Creative Commons Attribution 4.0 International" verified 2026-10-07; the mteb mirror's MIT label cannot relicense the data) | Casanueva et al., 2020 | ✓ |
| prompt_injections | `deepset/prompt-injections` | Apache-2.0 | the dataset card | ✓ |
| thai_wisesight | `pythainlp/wisesight_sentiment` | CC0 | the pythainlp card | ✓ |
| thai_sib200 | `Davlan/sib200` | CC BY-SA 4.0 (share-alike — ⚠ may reach redistributed derivatives; v2 leases) | Etxaniz et al., 2023 | ⚠ share-alike review before v2 |
| ag_news | `fancyzhx/ag_news` | **none exists to clear** (verified 2026-10-05, T3: Antonio Gulli's 2004/05 "AG's corpus of news articles" — ~496k articles from 2000+ publishers; no licence on the card, the mirror, or any original hosting — the corpus predates the mirrors and was distributed without terms) | the original AG News corpus citation (Zhang et al., 2015 / Gulli, 2004) | ⛔ benchmark-only (demoted 2026-10-05, T3 — no grant exists to clear) |
| sst5 | `SetFit/sst5` | **none exists to clear** (verified 2026-10-05, T3 against the PRIMARY source: the Stanford SST zip's own README — `nlp.stanford.edu/~socherr/stanfordSentimentTreebank.zip` — carries NO licence terms, only a citation request; content = "10,605 processed snippets from the original pool of Rotten Tomatoes HTML files" — third-party copyrighted review text, no permission grant; the HF cards carry nothing) | Socher et al., 2013 | ⛔ benchmark-only (demoted 2026-10-05, T3 — no grant exists to clear) |
| xnli_en | `facebook/xnli` | **CC BY-NC 4.0** (facebookresearch/XNLI LICENSE) | Conneau et al., 2018 | ⛔ benchmark-only |
| wanli_en | `alisawuffles/WANLI` (AI2) | **CC BY 4.0** (AI2's own page states "License: CC BY" — [allenai.org/data/wanli](https://allenai.org/data/wanli); HF tag `cc-by-4.0` — both read 2026-10-08). Open derivative question carried for the owner (verbatim from rethink Research 002): the corpus is GPT-3-generated with MultiNLI examples used as in-context SEEDS (never copied); MNLI's own terms are not stated at its primary page, so whether seed-influence reaches the generated text is an open derivative question carried on this row for the owner | Liu et al., 2022 (WANLI: Worker-and-AI Collaboration for Natural Language Inference Dataset Creation) | ✓ |
| emotion | `dair-ai/emotion` | **research/educational only** (card §Licensing — not the hub's `license:other` tag alone) | Saravia et al., 2018 | ⛔ benchmark-only |

Attribution page for the ✓ rows before the hosted lane opens: **LANDED
2026-10-07** (rethink `7dd4f8a` `site/attributions.html` + the reflex-side
ledger corrections `2476d66`). Re-sourcing candidates for the ⛔ lanes
(permissive NLI + sentiment + news-classification sets) are tracked by
riir-rethink Issue 024 (the ESC re-source row; Issue 023 closed 2026-10-07,
record: riir-rethink HISTORY.md). The
2026-10-05 T3 verification settled both ⚠ rows to ⛔ by PRIMARY source
(the SST zip README fetched and read; the AG corpus provenance traced) —
"clear before charging" was measured-impossible for both, so they were
demoted alongside the T2 pair (riir-rethink `arsenal.toml` is now the
five permissive rows).

## Suites

### 1. typed_decisions — `LocalLLaMA/typed-decisions` (config `all`)

Config `all` verified via the `/splits` probe (the other configs are the four
single-workflow views: `agent_trace_observability`, `customer_service`,
`invoice_processing`, `security_incidents`). `test` fetched to exhaustion:
400/400 rows. `train` fetched to 1200 (dataset total 1200). Issue 052 lifted the
birth cap 800, which had stopped inside the security_incidents train
block (offsets 900-1199) plus invoice rows 200-299; pages 000-007
byte-verified unchanged by the extension. (The cal slice is the STRATIFIED
round-robin front over the whole train split — Issue 039 T2 — so it
re-derives on the wider pool; the test split is untouched. Accuracy is
published only at a re-measured posture: Bench 078.)

| `splits.json` | — | `0565a08d42c3114d801a36392c6e7e32a36f65ccd89abde0b5caeee539bafa78` | 882 |
| `test-000.json` | 100 | `0672487ad56a1bcd094cad13a2e3d36c1cb619afdb6f63439b0442f824377a4f` | 403387 |
| `test-001.json` | 100 | `b80c98bbe142e5f854af5af8670317b048d79f00511cafc5a2af75d5018b92fd` | 511941 |
| `test-002.json` | 100 | `4750168865c4dc55dd02964c0f954f8d9968c8b2cce620341d461fb627415214` | 434391 |
| `test-003.json` | 100 | `5011d0b46e7c1f88821b5d791613fafad358991b0f709e1358cb18839d718fd3` | 438040 |
| `train-000.json` | 100 | `bbf806ebb595bfa422b7693df45933413f22914002fb3fe3d8e86e956141e346` | 402209 |
| `train-001.json` | 100 | `048286d7076f43efca173ca2b5acfba54205d33b86f5d9be4aa6ae09239be585` | 401960 |
| `train-002.json` | 100 | `e293af922145dac66357405231e6b3ab3008789167ccb2c63f66a1b62aa32b14` | 402191 |
| `train-003.json` | 100 | `1e91d1754e903db78e1815e0e793d7b8944950a44a79662c24838163ecf83f46` | 505526 |
| `train-004.json` | 100 | `3851cd6327f07d3f2b375a3c30b0e0bb507f76a550604ffbcf9cbc950a570a64` | 507806 |
| `train-005.json` | 100 | `a3d6594001822fd171ec764d1fe01b1de715fa20e55887f9b94989fbb344d6de` | 520708 |
| `train-006.json` | 100 | `038d24a4157c2cf721cc29607c5add0a08d515b4c2af147a3dd97c9a469a13b3` | 435017 |
| `train-007.json` | 100 | `59cbf5cdda4cfd702126c335508e63f7222a2d2ae23a0452c4ac68b23c8fcd80` | 435916 |
| `train-008.json` | 100 | `d67bf2341f19e64b7b2ef08debccab86e0df0fb55b4676e80757df63cd9c3463` | 434805 |
| `train-009.json` | 100 | `10ffa982aa43194277b68dc481be170d9601dd32af2d754a8a4d92c3166249d4` | 439632 |
| `train-010.json` | 100 | `6e58806819a9c7e1bbba6367c52d498756964906dd00885422ec8bbf538652ac` | 439213 |
| `train-011.json` | 100 | `f5186ace26d830299087895851c86fce2ad4789a57ec960d9a5e6d03fd542118` | 438253 |

### 2. ag_news — `fancyzhx/ag_news` (config `default`)

`test` capped at 400 (dataset total 7600); `train` capped at 4000 (dataset
total 120000).

| `test-000.json` | 100 | `a4405952695337b9f671fde0dae5fa936d7debdf699abcc9fb87223bf4b2bd66` | 33430 |
| `test-001.json` | 100 | `25b0bae5519795fa420dfa05d00e5e3813e97bcc275c4229276772e4e9388e0e` | 31628 |
| `test-002.json` | 100 | `b72d06c7a948b3de8633908251097e1328f284c7d8cb6027f82f49829d42eef4` | 31232 |
| `test-003.json` | 100 | `c1c757bffe809dabbe5d3b66e7446fbdc52b06173e6aee3bef2013c5bea61612` | 30458 |
| `train-000.json` | 100 | `291f553454afaf7c9ab42a9e7b205bdddce9fb5bbb222222615702b6d536b085` | 29970 |
| `train-001.json` | 100 | `23bd2b3845b27ba3c329f5896a6fbd20ade5bb824f8949252bd3b42035df2bce` | 39349 |
| `train-002.json` | 100 | `732dbd4a86f7159cbb916e0e04b2f55f48f335a6edb256e483667c922091ac79` | 34144 |
| `train-003.json` | 100 | `edfb33bb8c73d5a65c459191025d421a5ad290e85b5aa504c597c4055fc5684d` | 25971 |
| `train-004.json` | 100 | `85f147c0b74b684f7b8735bcec3a23a0cbd899ac24bd89fab326daa78bde81b8` | 33914 |
| `train-005.json` | 100 | `2a4d49df4fe85d530b12fb93e96222af74f53c0524f1035d9da8647e37d9aef6` | 31734 |
| `train-006.json` | 100 | `ba5a596d25ae867b4a4712c152ee84bf315a601272a4f6e6af2ec2325afd79e0` | 31495 |
| `train-007.json` | 100 | `15811b5a3a12bd99f92360ba57c02052dad377d665bb11e1c4e8e3c829680cca` | 30654 |
| `train-008.json` | 100 | `8556428c460267d66b090570d1e6b8de6af001407ab3a88eee988b4e027ff588` | 31041 |
| `train-009.json` | 100 | `add5b8168a818beaf9eac7536f7734eeb2e15ad4c7b759b023afb15d2fdf982d` | 30437 |
| `train-010.json` | 100 | `779a07c673552337a1cdf88491def9a06750f4f2583bc8b6214cfe8b6470b092` | 33104 |
| `train-011.json` | 100 | `98638a435701c6351ea2c1ab8fcc26bd035752f275afbc0ba5cab16c5b848149` | 33446 |
| `train-012.json` | 100 | `200b36ba10eb9594d2ded7d905fbb1c120aa4e8e1e6072986c637a288298ef79` | 30082 |
| `train-013.json` | 100 | `ed912aac529ba78a637e8832bb3d321ce1ad3d39d30a86b572004296d0014342` | 30021 |
| `train-014.json` | 100 | `880ba240de6079e4a72865f313018dca774f9999e288ebb71bb77b7af080ac56` | 32565 |
| `train-015.json` | 100 | `4bd41b4dab6596b3bf19051cd8b4cc2c7708b8d0c21973f669b4b4c9aa355a3c` | 32125 |
| `train-016.json` | 100 | `ab84c2eb6703deb280b18ee9d8ed299d216b23c48c99ceabe9379efb68d3100c` | 30874 |
| `train-017.json` | 100 | `c41d88cd43fe1a0b22ea47262a4d158627744602f2bd71c19cd91e3cc9b67715` | 30880 |
| `train-018.json` | 100 | `88d24c1705962c43db2e5055ecfdb0bf67f280cd1c784e48db2596419835c0da` | 32044 |
| `train-019.json` | 100 | `601be8cd36572d557458bfb50173563c00bda003584e3e5cd10d15aa1e443ed2` | 32921 |
| `train-020.json` | 100 | `fa824d5e89e22ae06cfccb61e3de934429a3ad6f7969abbc7e42b70ab02a377a` | 31192 |
| `train-021.json` | 100 | `18f235f252362fe976ca890f5b478d2d0a5228af85a81ad4d66e09765ca58ed2` | 31052 |
| `train-022.json` | 100 | `208bc87926e3546b6e3bd874fd3442b5853bee36c52d599f474bc34d54deea71` | 30356 |
| `train-023.json` | 100 | `03acc78cd8549ad65c7d6c3cb2b702a32419e6324897e9e03cdaef949569eb65` | 31184 |
| `train-024.json` | 100 | `a9a25f9bd58283cadfccd0ca5910ea8a7d6500350b7c36ddf338835bc10f1278` | 30347 |
| `train-025.json` | 100 | `e11eccb00239aa92a0f999d221589dc9830f7cb2af6d819cf4fc4b12b9470c28` | 29956 |
| `train-026.json` | 100 | `a5fd0ab044c63b33558e9a31ad9edac74bc7db94e2b6f253ede5222ec7019372` | 31867 |
| `train-027.json` | 100 | `b95b05f8614d1282d25c0b699aefb03280877a302454e1033b1f5edfa475ed86` | 31266 |
| `train-028.json` | 100 | `9db5d7e0462a12c0a9101a74475567db0cfa6f78e1ec5499191f7bdc9900fadd` | 31837 |
| `train-029.json` | 100 | `d5f58f7b9e8d7e05ef2bde0009abf83185f495f82f5de3e237aeedbaf1113dd3` | 31477 |
| `train-030.json` | 100 | `b52debe042db3007f7c0c21e617df1d8a5f41d2471deac2e4bb961a63e3a8508` | 30853 |
| `train-031.json` | 100 | `72db6eec2800ec5aeb68f9d73baf8f9ee707205010cc1630bf5f8edbfaefb23a` | 32268 |
| `train-032.json` | 100 | `5bc83fe1157ea51c1aab964b603c19680deb7b542229ca8f29039d2fae12ea8f` | 32027 |
| `train-033.json` | 100 | `90851ca027863917e5cb0fa08ea1ddac878c007ab43cb3fc8c11914b8a08fa37` | 31983 |
| `train-034.json` | 100 | `32289b6a35d8f787a4d7d8fd7879bd7a25a443335e76c756ecf2d1f79ff10997` | 30497 |
| `train-035.json` | 100 | `1fc807d2dd87ecfb732f284a43384e8d38fbd923c0c73ac1417236c3382f55fa` | 31640 |
| `train-036.json` | 100 | `3e35931a31ffa40c43a60a35012f9ca3901b938622746d1b3c746bb7e4fbc050` | 32479 |
| `train-037.json` | 100 | `4d9a4875f01fe736760581bba2aecc762af7c89d19d88302b573865eb7efb7dc` | 31576 |
| `train-038.json` | 100 | `bc87ca74e7d74d45ea40c45bbefc346bf54768c3b291583d2bbec8a47a87ea3a` | 32074 |
| `train-039.json` | 100 | `af28affbe8d0255bf0c7a289f3ff0f7cb8e02a511a957c5cf49f68fe2232a2f4` | 31843 |

### 3. emotion — `dair-ai/emotion` (config `split`)

Config `split` verified (the alternative `unsplit` config was not fetched).
`test` capped at 400 (dataset total 2000); `train` capped at 4000 (dataset
total 16000).

| `test-000.json` | 100 | `6f6cc068a193fd2a9cfe022698c0c0336e9ff3954c6ff97493131f13b8f54255` | 15892 |
| `test-001.json` | 100 | `b59b7ce2df5094c25652728a435259a2be366632ef3bf0fb81e4f0c162b3cb12` | 16510 |
| `test-002.json` | 100 | `4fcfedbb92fc268f0ef9473abefb4f33104016c13d116a0e5e438f21c2fe42f7` | 15728 |
| `test-003.json` | 100 | `7d7db8cd2255aa406ebea842fa75cf5dcdd44c5f1f68d1ba9c7ca29db4f53614` | 15504 |
| `train-000.json` | 100 | `51ae2a4ba9bf428ec9b3f31cc89b9e173e51e7b07d48027647736b4c19bd52d1` | 16622 |
| `train-001.json` | 100 | `c7c4d2f67e158c7c2d7e1b9722298c665c4d140273deb5630945836581579412` | 17649 |
| `train-002.json` | 100 | `62c48cacdfba0b36d2b1ba88297dd68fd1343a1e462c1bd858af6c3d55e3618a` | 17792 |
| `train-003.json` | 100 | `66e18eed058013d78545ce190f7931b29654fa76489d4676e0ddb20a09ee10ea` | 15479 |
| `train-004.json` | 100 | `e2f628725a89e01c3b8359d0c258efb0bfe7a28cbb4ab15910715f72094414e1` | 17353 |
| `train-005.json` | 100 | `da56c108dfae33be3c95d5dd6c4be9cff74b6a60e65c08a85f1f9779cde1b348` | 16524 |
| `train-006.json` | 100 | `ff335329637c986a5329da5c92c4203c6ae6f70cbc29fe9f85d2cf8b4100b34c` | 16316 |
| `train-007.json` | 100 | `0b932893509882160193d0b2c2858e9c8a0ce5837ce662c23c734b725ad1c607` | 17580 |
| `train-008.json` | 100 | `4add7a3ab486e06cdede2875fd1907fa1f2566c2d435d7d70df316ac543c57bb` | 16692 |
| `train-009.json` | 100 | `b5ca91d6e1e04f53872e0177124452b045138782eb0576f8481c0330c6372fc0` | 15520 |
| `train-010.json` | 100 | `b389c12f1c11f23850dbfa657cf6f820c820c82785f4596e226c7ad544ff0cfa` | 17325 |
| `train-011.json` | 100 | `eb692ce9dd91f59cfbf4d644287f9bd219d277a68e6e216ad4fb707fa473286c` | 16975 |
| `train-012.json` | 100 | `234075dd3b673b311c98c96dc828f6aa5a52f80e004ce59fe7bb389a8f2c51e5` | 16818 |
| `train-013.json` | 100 | `01a22b418327839d962d99349ab645f4ef7c588865666421b4ebe4c95aa11913` | 16860 |
| `train-014.json` | 100 | `a08f45f41db3fec36d30bfcb257068d6067967e75277a4ce1d3231af44a1b8c0` | 16835 |
| `train-015.json` | 100 | `410f00b724a3d1e9847e12ccecf714222bacffa96dd3c5900c04e2871bb10e2b` | 16509 |
| `train-016.json` | 100 | `afc9f657f12b6c0a514d82f965b7b4335c37cc65489b78a7b3fe8a2574991bc2` | 16511 |
| `train-017.json` | 100 | `ea80ac1ea27394720a75425254b15fc561c6ac0bdd435065fbdf489a311ad06d` | 17311 |
| `train-018.json` | 100 | `d29746b36911e3e5531aee291fdcf5a5e8b8f713d96870c8cef44e63217cb805` | 16215 |
| `train-019.json` | 100 | `c6552958fc3cad8cdfd1efeb173aab41aa22af6637d8aff21df2600748e3f6bb` | 16122 |
| `train-020.json` | 100 | `ac50ee0b41ce4bd06ad642d614309e6bfb884404aac4f88dbb925f033d6a6b95` | 16183 |
| `train-021.json` | 100 | `7443d3e760f31337409c2a14378b9d176e78eadfcbcb87e69f9902f89b37d287` | 15642 |
| `train-022.json` | 100 | `af102f345aa072b7759586e77948a1ed7b9b75c1537a11bbdb1b51ac0cad4105` | 17209 |
| `train-023.json` | 100 | `db9b915b26e355ba649c27c4d349cb2bddb4b9ab2afb33b218fd15c1bcb414af` | 15221 |
| `train-024.json` | 100 | `4d2af20f93e76cfdf2548565d605a94b59e3384b0c5be1184f45f43a675355b7` | 17058 |
| `train-025.json` | 100 | `5383d5977d08edad0747c8ec57a676dab3edc598d5b70da0fcadaa55280b8fb7` | 16280 |
| `train-026.json` | 100 | `fa9f0b673c334d6c041a8eb52151f14b2a2930b1e7f8b7190e646e7930312c1c` | 17401 |
| `train-027.json` | 100 | `59d95781182b148e1b61aa33eb08ec809a668f9efa8080ff17c6b99b433f3130` | 16663 |
| `train-028.json` | 100 | `1dc1d6f885dcf17421bef20f05aeb2ccb01d38995c9fd60ff63524579d34d2eb` | 17004 |
| `train-029.json` | 100 | `33284e49894456e40b1d2c31e9c3c8195fe690a8edcfd45416f2d28d8f626ac4` | 16406 |
| `train-030.json` | 100 | `f30181eb7c4cbae5106f6d8b148890e3ebfbb19e0e966324ea350c565def505c` | 17155 |
| `train-031.json` | 100 | `139512ad93c17cc0aefc3bf61dec30930e3f6e07c829acc68b08eae72351bf3b` | 16415 |
| `train-032.json` | 100 | `8c305410e568638899a6e1e540603fb0689f5b908195e4d2246c14b69e37bef0` | 16442 |
| `train-033.json` | 100 | `9cad8a65839d92534828610a90a8244368220dc368a50f59910e9397c1b4392b` | 17309 |
| `train-034.json` | 100 | `f877661f4f0e5f217235e0e4a5eddf4db10a227851298e3a0337a1f803a9c8ea` | 15826 |
| `train-035.json` | 100 | `314f81f7645be83f2415597240c9554264c147c5ee2c4ffd67457e781f190425` | 17106 |
| `train-036.json` | 100 | `2d13323863bcbc45c4aede7980f70619dc8564b757f7e59757bdf0c3e2129abb` | 16685 |
| `train-037.json` | 100 | `91e013ffd37e954974e3b7b67df7c97854d71dff9405c12d70fc23c56d699fd8` | 17108 |
| `train-038.json` | 100 | `2fdd65380487dba0e63b472b031154d40cc6825721be3426f5c9a49e26d4ab22` | 16416 |
| `train-039.json` | 100 | `616c74e245ca74dc8be0bb6fa7ed467d4e1bf799b73f4578394a65357f00eee6` | 17496 |

### 4. sst5 — `SetFit/sst5` (config `default`)

`test` capped at 600 (dataset total 2210); `train` capped at 4000 (dataset
total 8544).

| `test-000.json` | 100 | `383a15537adcb9fc4ab20af79586dafcd0a77440491bd5444892bde8142d97ca` | 19932 |
| `test-001.json` | 100 | `a693c15aa15809c23655579d3e73510cf31d3f4e2bc2af3185abda81fe1d66dc` | 19603 |
| `test-002.json` | 100 | `e42e772d6e41169eb7e4a689e02803d27f6efcdbfea04ac627cf2e4b2852295d` | 18796 |
| `test-003.json` | 100 | `cdf9a6f9e7d34d66065edadb0753876dfd339627b80cb1d59efccd3260e43d3b` | 19555 |
| `test-004.json` | 100 | `589c7b800aba9ecdbd4cf42b0824115485821de3fa16b9badeda834b08247529` | 19832 |
| `test-005.json` | 100 | `82854163344d25430c08ddb079a9651e5a8cd9bef8b2a0c4363d3ae425f4a73b` | 19630 |
| `train-000.json` | 100 | `043499ec50ae731dd1db3a67ed8ca722f1368183ac6aa289a75f193575fe00c0` | 19145 |
| `train-001.json` | 100 | `1d4dbc027fb3cf8e7b6f1f8a0dcb767fd6a0db1f88bf2c1a065a9b530b8cdb68` | 18119 |
| `train-002.json` | 100 | `35472af60b2410d9b5b0ee6b608f1ce8161bfb7035a7d88dcded1f53d276392c` | 19959 |
| `train-003.json` | 100 | `e31f2789a43a0db196fbf4b39931ae68764d973e52d9995c7e7f083d576a57d5` | 19606 |
| `train-004.json` | 100 | `3ddfae9eef1b167f9a25b8dde6638fc6b0575c8eb00ba0055fdc0237f70a36ea` | 19921 |
| `train-005.json` | 100 | `3ce69ec866179bd9c872362c85cfd07f32a600fcd614f3fac35cbdbac65e9f33` | 18907 |
| `train-006.json` | 100 | `4ad72201cec5f7ed85e9c8c0b15a579b56ff06fc2c4955842b500503458f76c0` | 19507 |
| `train-007.json` | 100 | `fca44b67d7c212b9a7cae5bd601d933f21830296b7ab04701140605c01a1cbf6` | 19394 |
| `train-008.json` | 100 | `14efc0a10c4b40b06a497ef50b21f2ca734e414c639558116a26466ffd272373` | 20359 |
| `train-009.json` | 100 | `b0063be87d8fe84a2adebfbe9aac3ecd81f237fd4cb42956bb84d6dea145e095` | 19531 |
| `train-010.json` | 100 | `8afdb0567ec0d42fd9a062fce9f296b26a77608e6783c8155566341d3836ead3` | 19435 |
| `train-011.json` | 100 | `06bd1a03df738990fe81806947456cdcb1bcddc81640b9345eab9b2707b8f57d` | 21061 |
| `train-012.json` | 100 | `0d3f6a7b57d852ae1eecceba7a705be6408f5fcc743c6686480340a0b4ec7711` | 20722 |
| `train-013.json` | 100 | `f1cc77864f60ade42bd2eafbfdb195f7e33a60d1b43ac2114275a845787ce8d3` | 19071 |
| `train-014.json` | 100 | `06c5c73170dc83298af6ef52ce09f073ddf08e503a835fbda6dc8cd64e79b3be` | 20377 |
| `train-015.json` | 100 | `cb54b3cae338ba1b3ac1d69fd4513de6c7c06f89e76f8c8eaacfaf87391862fc` | 20589 |
| `train-016.json` | 100 | `2510b78e454b1d5cd6fc015b7322e3182dd7ec2b838db39029e90212ee98317d` | 20386 |
| `train-017.json` | 100 | `42aed65007e702355b58d65e519fa1919ec37966c7eed0da566506b1e047a867` | 20577 |
| `train-018.json` | 100 | `f97821b44bf89b65e43eb002893e0c22fefe62b4ca8108bfc20fbb7b8ddbac23` | 19335 |
| `train-019.json` | 100 | `88465890c5007d259a6b174de949c2e53db422d0ce942963da136f2cc87e9a28` | 19736 |
| `train-020.json` | 100 | `34fd64d74c292eb991ea3ae14c46aa6837371394205fac69fa382430a3444b35` | 19553 |
| `train-021.json` | 100 | `e0404e3d82322f4372d7ba7a34de440b1c0e7e4bccd392957fe347f9f05af03d` | 19740 |
| `train-022.json` | 100 | `fe6712d1cbefc20ff87c3f2468768896f522609870acb88a97172e852b7bb022` | 19351 |
| `train-023.json` | 100 | `345b24fba45b3e541142ed8b227d4088b859f093e182353176ee2a110f03ca67` | 20314 |
| `train-024.json` | 100 | `88fb666b0c20c0c61efe1da4264828e5aa44b943c59f6e9b1e2fb5338cc969f7` | 18955 |
| `train-025.json` | 100 | `15e65bf4bc0ab4ea2eea9c8e48ab9b5fee97a5bbfc1ed05506c83a875794a032` | 19900 |
| `train-026.json` | 100 | `18860a564cca1ee2f26b8f09a5f80841ed01f32b6b47bcd614901baf6a3c53a1` | 19851 |
| `train-027.json` | 100 | `32d6518da40f1aac527f291c3c05208a498ef2156caaefa78fad0c37e4c32617` | 19327 |
| `train-028.json` | 100 | `e7d18ae6bb351b153ae6493e9320427f70225d1ee36b7a345fdc5b08465710f1` | 20059 |
| `train-029.json` | 100 | `6e5de932498fe8d2b8ec9b41e44a0ee976f7ecdd2c2e25ca07933406bd86b7b7` | 20593 |
| `train-030.json` | 100 | `d1e74c9e29e587f8d417be7e64e42ab78ca9d66e99000ddd86447531091da572` | 19763 |
| `train-031.json` | 100 | `838f544a365613f5e58d8c5fca3b967d85c60e228be4d71360856f0a975e0e9a` | 20105 |
| `train-032.json` | 100 | `cc68292ae2e35184b9cff6eed70969b3a3d6657b810d0884d312ea9f985fcd5b` | 19901 |
| `train-033.json` | 100 | `5be853eb3c2b21d9a39d5304bde928f1a47f65cdebb5bef7a40c73478432662a` | 19610 |
| `train-034.json` | 100 | `8651e8d314dee86e479973bde314a727416dbae9ac8cb6f533f6134129911ef3` | 19131 |
| `train-035.json` | 100 | `3dad8f3393b60780a1f050e5edf1b6312194df60a5b67702c9e0bc0628443187` | 19518 |
| `train-036.json` | 100 | `f6279c962838b838733f4b57fad688d3d964a4febd0b9b6e60142f185b622a98` | 20086 |
| `train-037.json` | 100 | `deb6fc5a752c230bd1870eba71626fa58604ce66ed7fcbb217291248e9b740d9` | 19826 |
| `train-038.json` | 100 | `0116e8d80bed7a6cef421236e407be9d59d00300073267f71dc5541170a5fa30` | 19012 |
| `train-039.json` | 100 | `a5052c6addbb2c7ce36527e6dfb00c11004fabb0aad7d77df8d80d8a8eac627b` | 20132 |

### 5. banking77 — `mteb/banking77` (config `default`) — RESOLVED via the mirror

The Colab variant's `PolyAI/banking77` is a script-based dataset the
datasets-server cannot serve (see Gaps history). **RESOLVED 2026-09-22:** the
fetch now targets the parquet-backed mirror `mteb/banking77` — which is the
reference's OWN second variant (`bench_apps.py` "jev.banking77_full": labels =
sorted unique `label_text`, gold = key position). `test`: 500 rows (of 3076
per the mirror's own `/rows` total). `train`: 4000 rows (of 9993). The
builder (`build_banking77_mteb`) derives option keys from the data exactly
like `bench_apps` — no features-names extraction. The PolyAI reference table
below is retained for provenance only.

### 6. prompt_injections — `deepset/prompt-injections` (config `default`)

`test` fetched to exhaustion: 116/116. `train`: the cap was 1000, but the
actual split size is 546 (per the `/size` probe and the dataset card yaml —
both agree with the fetched count), so the whole split was fetched.

| `size.json` | — | `08283c92fbcf4a577cb79208e0ba2e75c2f9562f0d6c90a95c17d9794c774d31` | 838 |
| `test-000.json` | 100 | `ace8b72b4d3dc63ff0b5f2a6e1dfbc5177c87f580d45611e8bf6b1d1d035c0a5` | 18237 |
| `test-001.json` | 16 | `7187eddd05e8cc1e4c5671168ee18363af50587a5ed0c8bbbb97d74c2ec11ad2` | 4304 |
| `train-000.json` | 100 | `4143061ec6753c9fb27f5a6aa26efaba426392b203e17e1f7ca3db8b029edb86` | 14482 |
| `train-001.json` | 100 | `b2246c060dc1bf7c31acb06c24c731cf3424689e859560f24595fb5d26fb7185` | 16169 |
| `train-002.json` | 100 | `35bb4236807aed8c1fa3a108602fedefe823408c9cb332919def916013d62dc1` | 15442 |
| `train-003.json` | 100 | `51023c27fb348e46e132983a4de175461fc8bdbd11d5815b07d7068ef20696f4` | 19015 |
| `train-004.json` | 100 | `25613dda63132e5a1e378828b666e85a94a123842e9ad2f03580aa0eb59a6bab` | 22546 |
| `train-005.json` | 46 | `a7cda7ea1c7e60be791628e7ff17f81c581fa4998c21f260f3e971f77d6b8a5e` | 14435 |

### 7. massive_intent_en — `mteb/amazon_massive_intent` (config `en`)

**Config correction vs the fetch list:** the task suggested `en-US`, but the
`/splits` probe shows this dataset's configs are bare locale codes — the
English config is exactly `en` (splits: train/test/validation; validation not
fetched). `test` capped at 300 (dataset total 2974); `train` capped at 4000
(dataset total 11514).

| `splits.json` | — | `4c3cddb21dafe3d7bdd05c5ec675864b5fe2c4aea8d22eb8dd9ef37bf8edbd97` | 11354 |
| `test-000.json` | 100 | `76c8e4f0af89aeb071bb22ee7da8ec1d444c6530bb6d4c6b7732df73366ca5ee` | 16068 |
| `test-001.json` | 100 | `7c0281146ae7394d994b92e1c94abb38f61b8cb14c18fdb999f9fc171d1afb93` | 16558 |
| `test-002.json` | 100 | `7a52444e1d3539fe763213388f7c995a2e8d6deb88817290874e2310c534bb02` | 16977 |
| `train-000.json` | 100 | `b39777110119403c8674348eab6cc44965d72a91b3d7197c25402ee2a2d0cbac` | 16087 |
| `train-001.json` | 100 | `0bf91248eb5e37d87aff8786feb7854fa975c3bedfda399b834b41421997e3bb` | 16150 |
| `train-002.json` | 100 | `1f366dc2d8decfca45e2f908adbc9a34fe17c0d89370985833459575a034b8af` | 16564 |
| `train-003.json` | 100 | `e5110b06be6b62f1652b8a0b82ed7cd3c6a623284615907437b715844b09e57e` | 16337 |
| `train-004.json` | 100 | `ef5ee674746425e749e7c2fa8ba74f29dcff273e859574fcb665127e21fdc043` | 16908 |
| `train-005.json` | 100 | `d2cc597d0b993344abb2475d95308951197686b1e4854b50408b2005196d3423` | 16597 |
| `train-006.json` | 100 | `7791e3434f0a93d2441877ed1d5cba28b92db78fa8ed48f40c457bd25a5c58ee` | 16739 |
| `train-007.json` | 100 | `4400aa2cfa6860d1152d389a05a058163c67b08eddee65d80cf25df7b28b1acc` | 16819 |
| `train-008.json` | 100 | `ab5fb70b153e9bb1d2801438b7f333a5d17a9a11b876a997a1f4b37798c449c7` | 16979 |
| `train-009.json` | 100 | `fe0a5819ec1e64d2bb6139791f0d6bcc60dc4e5fd8dd5adbbe3802ef40b141fd` | 16464 |
| `train-010.json` | 100 | `c0d2ced57363cdc967c1737d63a2e85891b172ef6fb65d0622dc9cd9b391d9d5` | 16979 |
| `train-011.json` | 100 | `09495cc1ef6e264241b68bd1cd09b20448a0042cdf3faf2cb74d0134a51191ae` | 17764 |
| `train-012.json` | 100 | `886871206be2118b9bd449cce84f8e3941e4b1144329f7c584327f69439085d1` | 16653 |
| `train-013.json` | 100 | `de0835f36c216ce89dd1c272ab8d97e9afad8f5f538bf0d0874ff12de550e57b` | 16700 |
| `train-014.json` | 100 | `25bd0e9b6fe990d6d009436e015c24f62823c50d7cf8c40ec6850733aacd814f` | 16782 |
| `train-015.json` | 100 | `fdd10145c265fcb29f9e87f272bdd244215c2d9399af9de2af7d9190fd74b66e` | 16892 |
| `train-016.json` | 100 | `c2c7c731338f1d593f0ddab353f63ad6008abc3182d2a543bf69fcb1553142e6` | 16777 |
| `train-017.json` | 100 | `32aca44402ec3e0b7d5e417cf87759777276de586b65a8ddca0494300b1d7388` | 17146 |
| `train-018.json` | 100 | `e487346b69562f9335cc64f5d5bb36534681ec935da758947740251059d84188` | 16729 |
| `train-019.json` | 100 | `848c68e16b4b94f118736549454687f0afb71b7ef341c23bc63fb5cc62a1faf6` | 16876 |
| `train-020.json` | 100 | `2efb6e2eda958cad283b8a33b3f27a63d495c9c420ed3ee400a01baa62726991` | 17013 |
| `train-021.json` | 100 | `7154376df6255bce5f74b92959b522ae62e654529c5d7730838a29797753cef0` | 17282 |
| `train-022.json` | 100 | `7d6ee0d8e13fd26bf7f97efdad53c84ba13392d19dd10ee94d379150d5d66f03` | 17071 |
| `train-023.json` | 100 | `938584460c9d917900745ffb69efcea89fa84e05928f2c2ccc0b3311b285bba8` | 16733 |
| `train-024.json` | 100 | `bbf72e9dbdfaa8c90735d2ff3d67a58f91d817d389adc1b7c840f13da908e922` | 16787 |
| `train-025.json` | 100 | `558d1d98d402cb85d76f821f4abdd7082001a95c6d74abc04df2ad60cd90ea15` | 16968 |
| `train-026.json` | 100 | `d663c73acd29c5f64e5c75e1894bb1887f481a21b6f670fca1b84a40492c957c` | 16818 |
| `train-027.json` | 100 | `b13a4e9339fd121c8f036ab86e07694a5fc1bf77238e71670097ad7dfb85f78b` | 16528 |
| `train-028.json` | 100 | `4baa2b54e8559536e3c1408922f7e4ab51717583b462e0c06647e2548ff4b4ba` | 17169 |
| `train-029.json` | 100 | `7c9f898ff563cde49ae5f69060fb37275e01211978773286e6506b20fd9b688f` | 16853 |
| `train-030.json` | 100 | `67d5f40960e949a33c923cfd4da2b70307e82e7cdd148a9512c78fa4a035597b` | 16812 |
| `train-031.json` | 100 | `c3a66880133ea088ba21b2fc3705787e8b33ad9823f0fb4fffd99fa3d6da80b7` | 17093 |
| `train-032.json` | 100 | `8cbc11f2d37d47998cc83ed395b708f5134011fd99d4d8ed22dc291791d14f24` | 16563 |
| `train-033.json` | 100 | `35a118fff0da1dfe6db6a9ca693548aa318441be161530328e499a269d4627f7` | 16435 |
| `train-034.json` | 100 | `9c138c654d2d5653235198c72d604812f27bcd50481260f87a130c2b8b4f05f0` | 16520 |
| `train-035.json` | 100 | `5fe0f823347b0703b461c8b27c547c10932672d7d56488606d338e5b967b5ed9` | 16878 |
| `train-036.json` | 100 | `c475d80858db27d6d47d0e0a2a9697700e524dab72fc10f6d5146242e844fb68` | 16760 |
| `train-037.json` | 100 | `f6bbb55e11e0284918111148b1abf63cb5cc3a2b6247eb48948ca6c376c8e093` | 16692 |
| `train-038.json` | 100 | `9a78accd871a43fa891122d198ee6941378145ae965063c85275d953e622c1bd` | 16667 |
| `train-039.json` | 100 | `4c7f6f3b000e359b97b540e77ad4f4e91c04cd2998f3ec5d9d536d698731bbbb` | 16768 |

### 8. xnli_en — `facebook/xnli` (config `en`)

`test` capped at 300 (dataset total 5010); `train` capped at 4000 (dataset
total 392702).

| `test-000.json` | 100 | `e584c07f02b3fbc490ff68fa4fb840f1fd3db9394e3daebc8a8ad99d9956d73a` | 22041 |
| `test-001.json` | 100 | `77c43f62248c1f09d79c5b43e6211d8668839c80af672f22495e25dd6888f102` | 22025 |
| `test-002.json` | 100 | `a0b539fcc71115fcde307172fbd1b35883f756ddf35594a8bb25859e6e40670e` | 20090 |
| `train-000.json` | 100 | `9b5da50b5d355a3b2223bd138b49fdafa9c08b93337dcb02d02a75f9ef07fa3a` | 26946 |
| `train-001.json` | 100 | `68da255d8aa895a28b5d27cdcdacdb8448f690bbde51c18152c22f3359d7f373` | 26906 |
| `train-002.json` | 100 | `b7eee25d252b7dd5eb00a89e0688075118a443236e2db884ddd69f42bd742ab7` | 25293 |
| `train-003.json` | 100 | `2fe452ef5e929772c06f98fde955919bb7e40d362ff1a2716f266d06d63be663` | 26556 |
| `train-004.json` | 100 | `9da16241cb481fdfd26c48c47473cd37a5932a1f8ef5f60ecc284250b6631eda` | 26610 |
| `train-005.json` | 100 | `2fce299341b42d2811226e935b9f1a7c574dbc0f3074f2e10b9b52f4837cb02e` | 26726 |
| `train-006.json` | 100 | `20419eff637007637e9e9b82d872d220913bfbfcd15b1ed28f702e5cf3e23ea6` | 26227 |
| `train-007.json` | 100 | `beff341daffdbd40896c61cdc23d7dc8ecd326dd5f96776c8e53977c60c9f1e3` | 27754 |
| `train-008.json` | 100 | `5af832f9a21b25310f9f68fdc0e15d9279c2b5b231f0a8acb488471bf1d606f9` | 24137 |
| `train-009.json` | 100 | `46795d6466ba8702bba276ad8e7efe6b0cfcfc0e9fb152b9e9dbcb5b51db165c` | 25833 |
| `train-010.json` | 100 | `125bbc2e36b6589b847175700b4d7c42d983491f37190eeaa4c6e9f38963ad0a` | 25971 |
| `train-011.json` | 100 | `3ee37f21f0d6cc858fbe0c44cc727ecdeb308e93417f94b58ca78234726da6c6` | 25998 |
| `train-012.json` | 100 | `386fd1ec7eac3a195f26e6689ef569478d5ed784b3cbc17502ac922b38b5b8a7` | 27234 |
| `train-013.json` | 100 | `69d52461b4d7954076f2ffbdf8fadd0682764725a133f8fbabd06be057e05d78` | 24613 |
| `train-014.json` | 100 | `a07780e62888f31852ba1a150965fb4073f2b8893bd42801a6b4bc54d162adfb` | 25752 |
| `train-015.json` | 100 | `e7477b7256276dc229cd0826dfdea71790f02c3e4f1dbf859fd26f3ceeb993cd` | 26599 |
| `train-016.json` | 100 | `c516834a335db738f84ff0a4941b108df4deadaad22b5c8eda506d80ac15b42e` | 24689 |
| `train-017.json` | 100 | `f6497e533c3a33ab69a400988237ac311bd0fc485440ae015dca262b5eea6aeb` | 27401 |
| `train-018.json` | 100 | `ac775eddf17f15ac08a34377530b668630ea640f7bd1bcf013681d3a04a7ca9c` | 27572 |
| `train-019.json` | 100 | `0fb3b9cb2f3113f4b054fbe0eebd5c45bfbe185f980cd003f658ad9d1b1088a6` | 26721 |
| `train-020.json` | 100 | `1093ecda3c5e4933d8be5e8ca3458ad2038a4a59d23e30a157d0fb070f458879` | 27281 |
| `train-021.json` | 100 | `85c8c347e2c9e8c13b57cf068497a57db069fb5ef1af1d009d586616ef46879e` | 26245 |
| `train-022.json` | 100 | `2eea9dbe0dfb566925d5c6e9353c9a39552a3328758d427f56260e362ed687f8` | 26000 |
| `train-023.json` | 100 | `928e593b2d63bf3004ac19496595fd6c7f18b43a7fb0aaccb90ba16504663d43` | 23471 |
| `train-024.json` | 100 | `ac4bf1f468721a2b92c57ef02d49394e8d9a0f082ada1674f6c05d4e89181b6e` | 27080 |
| `train-025.json` | 100 | `b227889636dc2e800c192d0a00a8b2c411d430a6585b630dc89f45855298cafe` | 26039 |
| `train-026.json` | 100 | `afc626655e0cf11796410921b86b64b45bbd1ed2a7cf05400c810a1bb3bee85c` | 27345 |
| `train-027.json` | 100 | `38bb6e6663104a17d993cabfa044999103a6fdb2426303073991223f30ae1e3a` | 23436 |
| `train-028.json` | 100 | `149ddf0b7bf142d8a2338354b4caba300384820bf101a34e96304423b9c51b68` | 25203 |
| `train-029.json` | 100 | `c620471303bdbfece99171b047e8bf81161a4945dfe92302ea57d653f55f6cc6` | 26607 |
| `train-030.json` | 100 | `4f17debcf1b504fe7fbb1c17131146736bf580224aa7e8925dd30f2d53dd402f` | 25698 |
| `train-031.json` | 100 | `fce2caa13734bbab702513fe7639a01d665927e6bd775aee8dd9393fa3d621aa` | 26584 |
| `train-032.json` | 100 | `bc6c9f84839b21963cb790f18aaeef73622b223c7865783132bcbf6bf898904b` | 27767 |
| `train-033.json` | 100 | `78b1c95982639d3f92509d7fef475152271de290df1dee103903b2c92aaec778` | 25808 |
| `train-034.json` | 100 | `17e44813082c62f6f1620b37ca2af359837d15b8575e0f8030f45f079c2a6fb5` | 26305 |
| `train-035.json` | 100 | `5a0b0386c9c8690a6d8afc65101e792c7d6cfdd08e1aeb45e4acaba96ca584a3` | 25910 |
| `train-036.json` | 100 | `15c9b16bc3323b6b2f66ec02cdaf914ce174b0253dbae7c6b58f903f3b30e9de` | 24901 |
| `train-037.json` | 100 | `c8c5921fdd12d07f6b4896a2bfae28b41fe10b1c6fad125e3b35fb38c1c2b1f9` | 27120 |
| `train-038.json` | 100 | `d3f468237708ba8a5629b43869c8e32df42b49ba6f080d39da2882e2f556f9b8` | 26502 |
| `train-039.json` | 100 | `b733f76daa727016d9506cbf8dc3c715e94e581693679d030b044f1219092f39` | 24447 |

### 9. thai_wisesight — `pythainlp/wisesight_sentiment` (config `wisesight_sentiment`) — Plan 003 T3.1

The OpenThai board's WEAKEST published set (51.6 / ECE 0.353 — chosen for
honesty, not cherry-picking). 4-class Thai sentiment choice.
`test` capped at 400 (dataset total 2671); `train` capped at 4000 (dataset
total 21628). NOTE: the config name is `wisesight_sentiment` (the
`pythainlp/wisesight-sentiment` hyphenated spelling is gated — /splits 401).
Label distribution over the fetched slices: test 400 → 73 pos / 218 neu /
102 neg / 7 q; train 4000 → 699 / 2198 / 1014 / 89.

| `splits.json` | — | `cd5347a2713e75e97755174d0a9d5dd787c18c377d6e018e2567ef2901a66ced` | 314 |
| `test-000.json` | 100 | `6b73b14964b85de0efcf7fd6e58199412d28fc1ff27508ff13e7686c90899d9a` | 31172 |
| `test-001.json` | 100 | `d290bd876a20a7c7c317557c25097bd238bfcde81c3b70153c9b6a505932c695` | 35896 |
| `test-002.json` | 100 | `c7472e63aab52d546b35673bed0a227dbbe82d5f97a21347001aff12481c372b` | 29741 |
| `test-003.json` | 100 | `3a8e09f3503135dcf3a04203a340e3d442f8b567ace0568ddd1588d3755d870b` | 36331 |
| `train-000.json` | 100 | `fdd70d3209abdefe552a247eaa7eed5c3953437e8706103e0a8531daeed5369d` | 30848 |
| `train-001.json` | 100 | `b1c54e05ef36c83297fb3bef578f8bfd379b3024680e0e9bf89888563f1c6e69` | 33066 |
| `train-002.json` | 100 | `84fa72a31b2fa1aa9ae7a207edc3f36161b216c2152059c7d35ad386e9e959e9` | 37864 |
| `train-003.json` | 100 | `25c1e971576e88a4bb335ff3a74123e51c04360266c3a4958eb8df0e68c80cba` | 41726 |
| `train-004.json` | 100 | `c10790d7f9f7ae97a549bf5028094aeec37fc0ac60f2086c7ad2211c47b762ae` | 34055 |
| `train-005.json` | 100 | `a2458e3962a59171666cccfb0247207bdedd21c7011d8b3230902e7dbd1dddf2` | 29920 |
| `train-006.json` | 100 | `94366aa2f8b8fdb8a9b66dbf05efdb52730a4260e6741c7dcd4dc4701cb2b94d` | 28804 |
| `train-007.json` | 100 | `db20f6da07395bdee011c7da7780d06f4c5f589bea4e91ae91f929e5c73a3f21` | 31408 |
| `train-008.json` | 100 | `95e3645bef94299087ea72cc0eb73d213468fb666a681c048dc935820da9c979` | 33279 |
| `train-009.json` | 100 | `2240e7248a9c11bb2f5d32ea7d3389f257383b37dda01c98a9fb9ca49f854bf9` | 29243 |
| `train-010.json` | 100 | `6d5ba23b0b19645f1ce7ebcbcc77bbf7df6e2498ff60b11ac3fd79c1bdc10bb9` | 32562 |
| `train-011.json` | 100 | `e2722ed32797c6de1c8980c8036c1eca9d227dcdc0f8342fc4c43df73e377ee2` | 24336 |
| `train-012.json` | 100 | `4c43659b4e61c337cb7374414ec9dc6af6eafe485c33e1cbf220312d0f0050d0` | 27130 |
| `train-013.json` | 100 | `07d367dac7df9529885f8954b8eebadc8984af26dbb1e6e2a50899538e9dfbc8` | 26740 |
| `train-014.json` | 100 | `17035f3c4da510d2d17c68e20b46bf65c396c5be5255c4a0a84576823af8624b` | 29371 |
| `train-015.json` | 100 | `e891b3eb8b37832514b258aec5b16c60feaaa9829698ac8b66ed33eaa3ce3801` | 27241 |
| `train-016.json` | 100 | `051d211cdf94a0487753a90d3772dab8002303d0d316b1e0129de7812f0fb095` | 28876 |
| `train-017.json` | 100 | `8e93c0a3bc3410261d28fe89f7735febb1aaaf7860385b9a4b78734c32376ebf` | 36757 |
| `train-018.json` | 100 | `fef431d34dd4379be5a86b92ad2ba58dfe9703a3f8550fd24e9e8cf55b46e45e` | 33633 |
| `train-019.json` | 100 | `634fa661bcde0d618cd7b947dee880bad5a577b22af2f4279d47618ec7ed3cdc` | 33132 |
| `train-020.json` | 100 | `908b37e3bb8cb6396a9de8b1ad29be6586984093e0d07c1acdd1e26228aab591` | 27486 |
| `train-021.json` | 100 | `53dbb78a7401c1804fcf679b3ab84bf6106655b61dbba10edeea59d60f5c08a5` | 35299 |
| `train-022.json` | 100 | `215c6245cb28ea3733e5dd940898bd45ce6b4a2469b6edabb50e600ecf3f3e4f` | 26704 |
| `train-023.json` | 100 | `c5fb7be9b707cf8f73e5d39958edff3530abcf296b76fcf856de2b789e5eef37` | 30000 |
| `train-024.json` | 100 | `120dcd51b08c188afcf1cdade8b51818f7ce6e3c8d215696051a97522c9e7282` | 29433 |
| `train-025.json` | 100 | `3cb18748a40850288743f5e20fb1460283aa17d88a04245d88ad2ed289a5d094` | 28268 |
| `train-026.json` | 100 | `ae6ec5bbf1363c7365f5239a2635304e5008bf3c0ff1813b6512cd5c8bff079a` | 29761 |
| `train-027.json` | 100 | `354b41c175ce4cab7afc8fdced0100fc5f8289a9a32ea495e58f46f2946eae87` | 28113 |
| `train-028.json` | 100 | `e780ae48a971c7034f42c3ee90d865576fdb4f3d4e70031a9af41650fac72061` | 32057 |
| `train-029.json` | 100 | `508ca06c2ff472189ab7506c7a2d3a212f0df562537e54b04836afc93ed9a99e` | 25384 |
| `train-030.json` | 100 | `84d132f486bb92e5ee454fa61f43d4d5914c163f16389b645235ee48f5fb6bdb` | 32678 |
| `train-031.json` | 100 | `ab21d4449144a56eb8a1f640624e9669ec33692d50f89bf72761f85a1f8ecb28` | 32398 |
| `train-032.json` | 100 | `44391e80f6ba99cd442c1f04c5a55789904e823b9e2e26db3efbe93ea6392aab` | 33230 |
| `train-033.json` | 100 | `5c8fe3149f0499b6185ad92aa5660c72d5a368d2c3ef1d41a9de66306670e9d1` | 37144 |
| `train-034.json` | 100 | `6ad2e15d0002207f2c0f4d85b9938536d3839de652d5f437fda150ba68b94cb8` | 30977 |
| `train-035.json` | 100 | `caaef1d6e20fc800737fc5056b2608722092e2b1dbd2b7ac3867062b33c30e6b` | 25391 |
| `train-036.json` | 100 | `5a1ab047c6995d27ed76b3580e834801bf517abdbcaad73d9d261bec3a742598` | 33911 |
| `train-037.json` | 100 | `be46f8e3546a7444e8f45f5342399c4da1c74b273455331c3fede6f7c677af1b` | 34188 |
| `train-038.json` | 100 | `f0839fa78fcb8747475088d1c328b921f03de3f54bcae92f4916d881985d6ad9` | 30107 |
| `train-039.json` | 100 | `41c2a66e9f710056f53265d047ceece8cf56bff4cbd7a330e7e4224f3c412216` | 27172 |

### 10. thai_sib200 — `Davlan/sib200` (config `tha_Thai`) — Plan 003 T3.1

The SIB-200 Thai topical probe: 7-way topic choice, `category` a plain
STRING (no ClassLabel — the option universe derives from sorted unique
categories, the banking77 law). `test` and `train` fetched WHOLE (204 / 701
rows). Label distribution: test → 19 entertainment / 17 geography / 22
health / 30 politics / 51 science·technology / 25 sports / 40 travel; train
→ 65 / 58 / 77 / 102 / 176 / 85 / 138. NOTE: Davlan/sib200 carries 700+
language configs; `tha_Thai` is the Thai one (verified via /splits).

| `splits.json` | — | `cce2315aac3fdc32e88629f36a90fe78cc2317179eec351d4c3a655e117bf278` | 45670 |
| `test-000.json` | 100 | `27fab133e4d4c2c5f7f2c9d85e4744c17a2265ac97e52bf5be0f451ebfe3baa6` | 45193 |
| `test-001.json` | 100 | `78bb2eac9af22d9e5e409f6ccca7b5354e384883c12c8597ca66cbe0b9544e66` | 46042 |
| `test-002.json` | 4 | `40c4ce4b674c2b128a466ad3451688b1fd40d214ac7f5b9fddf0874b27d2c503` | 2019 |
| `train-000.json` | 100 | `57a69463aa8eb0af1fb7dad25180548434fac078734e6366f4908ebfd37b8354` | 44060 |
| `train-001.json` | 100 | `21ab9999352d44f7eb75dd4dd8be470996a8713b163568722109b31f8082754b` | 47196 |
| `train-002.json` | 100 | `8071464105156646617dc411ae7c3f3da4022bcdb3de217dbd5e9140234d18e6` | 45970 |
| `train-003.json` | 100 | `6dec520a497d87c159c935489679ead3dc883864f7b1b1c2019c5dceac4b1ceb` | 46851 |
| `train-004.json` | 100 | `b1ce11d7f1e217d9747d22e7b91c379e9a053ad167802d9e156d7a5903672c49` | 46991 |
| `train-005.json` | 100 | `5692b4104f96149db54c2a552fbe2eda55ba602c526043eb2ed6c05c231a2533` | 43802 |
| `train-006.json` | 100 | `d92d83feb172ddfe18b606eea21a807a01898d29c539eb4614db7a07a2069801` | 47607 |
| `train-007.json` | 1 | `a1b947c11b1b2fc73e592ad67d051449a014f35b331c8e6cc7f31d4da17e6ab3` | 784 |

### 11. wanli_en — `alisawuffles/WANLI` (config `default`) — the ESC re-source NLI lane (reflex issue 080; rethink Issue 024 / Research 002)

WANLI (Worker-and-AI NLI, AI2) re-enters the xnli_en niche under a clean
CC BY grant — the licence was verified AT SOURCE 2026-10-08 (AI2's own page
"License: CC BY"; HF tag `cc-by-4.0`), not from a hub-tag echo. `gold` is a
plain STRING (entailment/neutral/contradiction — no ClassLabel order to
verify at the port; the builder maps the string through the XNLI_KEYS order,
the option universe is FIXED, not derived). `test` fetched WHOLE (5,000 rows
— the eval cap lives in the harness, the banking77 posture); `train` capped
at TRAIN_CAP=20000 (dataset total 102,885). Label distribution over the
fetched slices: test 5,000 → 1,858 entailment / 2,397 neutral / 745
contradiction; train 20,000 → 7,548 entailment / 9,455 neutral / 2,997
contradiction.

**Open derivative question carried for the owner (verbatim from rethink
Research 002):** the corpus is GPT-3-generated with MultiNLI examples used as
in-context SEEDS (never copied); MNLI's own terms are not stated at its
primary page, so whether seed-influence reaches the generated text is an open
derivative question carried on this row for the owner.

Fetch 2026-10-08 (three passes — two 429 limiter walls mid-run, the skip
logic resumed exactly; final pass clean, 0 failed_requests). First
measurement: `.benchmarks/130_wanli_en_baseline` (A0 0.3100).

| `splits.json` | — | `864338d2b6d2fb60df0d55a07dc1682d3ab41450d896295fd2e84c44c3eceaaf` | 172 |

| `test-000.json` | 276f801f4c7a10c48446a119d179ca5d8098ffe26de4268be25514af7d2b7212 | 30315 |
| `test-001.json` | efa797c0c6f68b5725b230052ab462c20f26de8bd22ee4ffcdcec5ddc186982c | 28189 |
| `test-002.json` | dbda592600344f4a44446858fc0da22433f73f1cf62a3db196d87ef1d2c8a810 | 28612 |
| `test-003.json` | 3ecf8be3813cee8f996b8a466a7ddd0380ef94f9b13286bfbcdc68332df7fdec | 29354 |
| `test-004.json` | 0c91b800338106d98f316f1a48defff91a00f7071402966ec0fcac8f6fdacc7b | 29185 |
| `test-005.json` | d3816562697456a390906a3e263ca7522e9c6102f9bf8c119c3f9c3d0b326cca | 30784 |
| `test-006.json` | f72afee331792d6455aa1a4f891350c8a39fe09ae9aa1f24195db82f71aea724 | 28639 |
| `test-007.json` | c360859b01105e70681ae41e85b0e27e8fef55bcdb42fa2a35d4f5014469c3bd | 30143 |
| `test-008.json` | aefe54731e886db414cab98b8687a77766381b289c85182ded276e6e3739eb5e | 28969 |
| `test-009.json` | 64d40b6b3d8d1ed247888d2d3e007267392d91c9dea8118125aec4b24c876388 | 29559 |
| `test-010.json` | 650b74eada116bccae4c346090f58980cfe2fdf30f907a91209d9bec9eb1ee2f | 28483 |
| `test-011.json` | 6cad7a358307f0e28bbbe82c03bf11bc369d41bb9b5d35865a441dc3e2ef1752 | 29855 |
| `test-012.json` | 926fd568531dff1e10202b2d98e392ddc052a2e0afa90ccf80047e280468e856 | 29544 |
| `test-013.json` | f04b3711ff17c0bb9cd5869803d2830280e9917a3007772e6baa100a782722fd | 31581 |
| `test-014.json` | ac357937663bca16762147d01476cdd12f5d4f1152d81ae3bce95011f31f8911 | 30322 |
| `test-015.json` | 4c2e0e744a3073ef7e75541fe9a9ffe41a4c6c18d540a4f2f088c5925c55b587 | 29454 |
| `test-016.json` | 538db79706d8a62ced42e006b359071ad7de217b8e2e4c02117c6ca1fca8a839 | 30845 |
| `test-017.json` | b33e4cb3af9f9679d5c0cfb870ec1c9df55bb9aad41aadc772d3d6a6015377d6 | 29621 |
| `test-018.json` | fa2c7e4482c7b66779ad11ed75d61be8b931433fc9a80d7454a0c8e32888a139 | 29632 |
| `test-019.json` | b7bb308d0902c7704bbcef6750ddb05db40569e6dfdf0863a750d77aec96c238 | 29748 |
| `test-020.json` | 471b934ccf58b7e25e904f50ba8ffbabe98043e2d52d1087de489651f5d7616f | 30587 |
| `test-021.json` | f308325f21f82b5bc67dbebcffce9ffabb6dd706637cc6263704f12cc90fe51c | 29589 |
| `test-022.json` | 0613d330030a2405aa33b932fe5dfd709d955b54f82657fd80f0388b92cbab20 | 31149 |
| `test-023.json` | 825f807e8a466e1efe10a16c1b344e61eabd79928188a0b8cd2878051c5ac038 | 30189 |
| `test-024.json` | 3f58a9975e4267d1e69f1afdc544760f0f474b4d40a169b94f9158fd70abd50a | 29813 |
| `test-025.json` | 4294d2953156db8f0ab536d6fea7247508806c07c0e0323ae1fa0e1c5489f930 | 29934 |
| `test-026.json` | 5c920f0467917485a82e3c3a4cddfa859e7550b05828d7236bf86d38c613ed4e | 29307 |
| `test-027.json` | f5d8a881d5e90c185970b61d3c065cd8ed975c5d8864283c795fc8b4b8bac1de | 28849 |
| `test-028.json` | b54d8c07b6bd034744b50615cc0e0f37e4ce8eea250bff6c56714b0fcb004163 | 29386 |
| `test-029.json` | f0a24e594fa434509836599a7f85a0d3f73dc8c07d353840deef867d96399aef | 28971 |
| `test-030.json` | 39dd7da729d0e383c9a1ddb5c2169b8aa572097d5a1442a863d4099d12835383 | 29759 |
| `test-031.json` | da76296b6c3f9f6d54ab5f2c2411bc100ea5436113d22fe1709dbb345b2b8a5f | 28758 |
| `test-032.json` | 68adfad61b09ac64af4f6e79b1bceda400428da46e808a948b3fb82e6fe44544 | 28313 |
| `test-033.json` | 53680026e824b52248a2945e3855066ff7afa9b3a1944e461914463cd06e3f9c | 29554 |
| `test-034.json` | 5d44b02cea00d9321cdc11ebd0167d0dc30ed659de653704dbfdf8ea59a1c0b0 | 29923 |
| `test-035.json` | 3c41a8216b4a4dab4c344c040b2e33afbec0e96f48f3506bf1cdece0a76a6701 | 29234 |
| `test-036.json` | 383377907c43b3edd92b1e8381905e2567d813e6774e37d78b71e4382f3950a8 | 30702 |
| `test-037.json` | 6f2bf5905e03ee95d8d4dc9e1d1f502368cb9e9b6e31a4dc3fd51ccadd0b9650 | 29522 |
| `test-038.json` | d1500a33698276a6426881966b1de2f1cfe8c5eec721ede624ad5874b5eadf8a | 29159 |
| `test-039.json` | 7b40e32d945540d6c3c1b82dfc4d517af7ea9783adc85c17484079f12901750d | 29839 |
| `test-040.json` | 1676b68f72b59aa293e5df0e292862986cf6c893678de952399f85638c76ee18 | 28192 |
| `test-041.json` | 8e6a59881ea30675b9c2f7a1c0ec0350fb455b7f575ceebbbe94edf42946b698 | 29683 |
| `test-042.json` | 42534375846a37c41f9718b4dffd2128867c95cb7fa8311a055fd95c573a03b8 | 29712 |
| `test-043.json` | 50baf2fe8b81dcf8e518239c6fd81c02b4dfa4ebbed3d248338fee657d0eb402 | 29754 |
| `test-044.json` | 5cc4bc038479d9e679fda662897835be5d5439aac40c7d0eca2192d7b33176ad | 30167 |
| `test-045.json` | 80779710b51c621dc0d3fcd9faea646919768a583a0907ef719f6b091d2cbe9b | 29653 |
| `test-046.json` | 02fb6557ca67fcb2edada27c06022d09e9968711a0c2c9f2aa5c2b64799b660c | 29084 |
| `test-047.json` | 0d81c871aafc3d9710a312ec94dfc45c4b4b0ecac5be599ddabe860c12ebee5a | 28552 |
| `test-048.json` | 17131d7947db42ba77b8b91615e4443ff38db014a6b1d52502b1f2d994344cf9 | 29590 |
| `test-049.json` | 28233b3a0a3641088c5d41cdb0959f368cc1d33a438719a9486594a1cb32ed30 | 28841 |
| `train-000.json` | 17c286da6739b175eab0f7194653d6469558b7c16c94faaa1d86d42beb367412 | 30549 |
| `train-001.json` | 5c5901a3dfec6f88325a278a685dea4f985a5b835cce3a8985e4e187b725c036 | 29812 |
| `train-002.json` | 4363e8fb085778f27ce3ecf6dab77dc073fbad2e0326e901ee83023f3a7fe056 | 29508 |
| `train-003.json` | d6269929aaea575fd109fe2f484671e6da2b5f183393640d5bcc7627dfb6aaa4 | 29457 |
| `train-004.json` | 0959c7ed645df05280386d6cccaf216f3b060dc7a37567115a5f557285f5402d | 29606 |
| `train-005.json` | 6a430e21348755c7b58e347d6f4543841c4490aadb378cd0a0c0d1dab9a65d6d | 29393 |
| `train-006.json` | 3f794fcd9d17b25d27b064355c3fecd3601c1dd1ce8396bf5d91785b6a8d37ea | 30072 |
| `train-007.json` | a8480b225381a5eeb9ad28387c2241eab2f8c23917ab9b4b7ace2bd05b44869e | 29055 |
| `train-008.json` | 9fc30918e5671f360a57517c88bcc689d1276fe00f81419088430f38c7e7b1f2 | 29953 |
| `train-009.json` | ec2cc1780d0c2b295fc956197bac87115d0fa61e637d13c1255d42b55f7f1cb1 | 29915 |
| `train-010.json` | 116497ba198191cb05a2b70cfbac5717c9e1073f36ffadde940af92fabc15960 | 28830 |
| `train-011.json` | 16d5fd9a818d592a82cdb76849f579c01576024139324017003bd9a0de631522 | 28378 |
| `train-012.json` | 8ac185abe6ec78553eb5320e58f5e89125366fcc2375288d9922345d09f52ce6 | 30455 |
| `train-013.json` | 4414ac7f9c54213cf121ee93aa1ec3954418db188eb8eec295ecbb6c82caa459 | 30239 |
| `train-014.json` | 788183e70608cf8d6839ec426e4fe7efe225283d615d2b76545f6dd5ba55efc7 | 29202 |
| `train-015.json` | d522ebd5e5c10d0cb0209ddc4866ed42bb99bfbab9cf49c56700d5f14eaadde5 | 29755 |
| `train-016.json` | 19cadab367b6e120f2026166285b91b88fa927af0c3ecbeca003d0f9dde4b9a2 | 30100 |
| `train-017.json` | 1f93e91be1240fe9d1fab50f1e5541f491f5b954507b7120e7cae411fbc04696 | 30099 |
| `train-018.json` | ff7955073b79d7c7a0974564d6e01b8050f2b25735636eaf326fb140b602dac3 | 29403 |
| `train-019.json` | 9a8ef3568aae01060da315e0375a157417daabf89e694631aa2ca469c437b1a9 | 30066 |
| `train-020.json` | 46e710a5fed89c6652ff4e50953f5583975bf3a773282121296913949240cf82 | 29780 |
| `train-021.json` | d279bc57ea90740f683ae523f68a062b1132dbf68a3929b4d1ffd3cb86c77e0c | 30313 |
| `train-022.json` | a434b42078a6b22a64f26ebeef55343f8be53f5de0d5b491662f9bcc15d3f2fe | 28826 |
| `train-023.json` | d54813875c7206c583b673d6b35dd1bfe8cae900ee38057c87d4b5fe3723b944 | 30297 |
| `train-024.json` | da6e6120af73461b7919548fd0416f208e40bc4b95faac1d312bdc9022a05e85 | 28636 |
| `train-025.json` | ec59d55f9dbf9f63fd1d313249c0fdd00236ccf57a1099409c790ca7bb2abec7 | 28692 |
| `train-026.json` | 3a550c3fa3c1d12dd57b9c9b68cb7761e6ad8985313987a3ae86c97945d0809e | 28795 |
| `train-027.json` | df2ef251e9a2eb812527ccaf04709c0a99e22e1667bda5cc4bb5eb9204008440 | 29589 |
| `train-028.json` | c697645b647a74019989864a21d94375d7a457f7253e3e7eda26693bc5766c76 | 28275 |
| `train-029.json` | 3d5a17f0c0056188cc29e71ef97933cfeea1346bf039c6be4f13a1254ca23707 | 30505 |
| `train-030.json` | e3c152b10242e49fc7c1cffd0ba3c3a323018ee12dd9abb9a954cbc76143fc8d | 29574 |
| `train-031.json` | 7ee591fda76eb3f31b35abecad6df45f23dd6f8cbfab0158e0467d018b40ec3d | 30534 |
| `train-032.json` | 3d9b0ee0295628ad33496eb90888f1d35e1bb8594aa78c57161a2aa313ff8417 | 30400 |
| `train-033.json` | 7fd04a9109e68d9cc4570398be1b5ed518b285a46706bbb6a8dc6bd47edacb46 | 29802 |
| `train-034.json` | 69b85618cf76e3e798debf14a5e60453c2e8e01176ac006918361cc96fa72eef | 29594 |
| `train-035.json` | 309c32a2118aa6874e2e86caba1aef9763dfc27ac2718b9286995e96c85cf01c | 29091 |
| `train-036.json` | 589b40ecf354195ebc122b7860a142eebf31a01a84325de8b3232d7448d960de | 28108 |
| `train-037.json` | cbf408b16012735779dbe204b0ca4e67f043446a4b1e11cd663da7076cd44400 | 28886 |
| `train-038.json` | 5139b3033a2368c4c6127367818ad9c4d221e97a064e3f6efb1e0b5e0784465b | 29761 |
| `train-039.json` | b5aca46b0e87d9fcd3ac50ec20837ac4a232acde4e0a53bdaae1505b9cc5f730 | 29712 |
| `train-040.json` | 4efdd2bbc20073426ca2e53b69e3ad1019a61c0ffcb69f22625b4bc7dd5908de | 30554 |
| `train-041.json` | 19e339f84101cfd50564b23be75253171c60ec852618b052c01299385ad32f26 | 30181 |
| `train-042.json` | 6dbb4d1a0f65a5541873f9af82c32e2930766ea6c5d6d0c98f435dc179e982c4 | 30178 |
| `train-043.json` | a4aeff9254bedf4c963c91a9e0cd8518474d655f75be556c42e78d3308ec4a69 | 29003 |
| `train-044.json` | dda89f474705e04c1bf9f44971b1f3d10452ee4f8594e6de2feba8bf514bc068 | 29580 |
| `train-045.json` | aaf0d6cb50850648c92d02a1b8126b4c9bf779d752cf0faebce9858150edc852 | 30292 |
| `train-046.json` | c561da41baf2f490a94bd554f7b5d69351dd95a12989e61b7dc1ff81024b24f3 | 30358 |
| `train-047.json` | dc0d8f9ff662c0d4eb27006385c5939e9c163210103526e55e47c29514a526aa | 29274 |
| `train-048.json` | 153f5fdd786d2275c03dca2cc667f6055400c05b81b707578807a84620f6827b | 29549 |
| `train-049.json` | a4a905a11583d3d475d76219b754ff28a79189da029553f3942c50d25fae95c9 | 28585 |
| `train-050.json` | 175482ad0c3f741130c3ff1beeaf705b949da6c29ab762009801e4792596b71b | 30107 |
| `train-051.json` | db3c33cc103e5b61bee0f41d641671782dafa72c2f87ce5a88858b5d3e13ad1f | 29641 |
| `train-052.json` | 95495f0ef232d035dcdc4799232cdecb896e66cbb29e6e9928968b7e6e4a2968 | 29526 |
| `train-053.json` | 17b350b20e0c1de0e34e66b81c80c86010d9f4cf801b0cb7a99fc5ceb65432cf | 28866 |
| `train-054.json` | 63e5d9125de4d927cc8f865039c81b245a27409dc1349779a1f236c7d9ddebe4 | 29223 |
| `train-055.json` | f7f530c3be0ffd89d57891f65123ae3f81ccc3f639429f4f436d9b199fb4be76 | 29701 |
| `train-056.json` | adf544239b531fc4f5bcdf5c62d3569e38b7986334f864acc0fcd76ed8215155 | 29798 |
| `train-057.json` | a5ee654621653ccec777d9bb17c4e1ce18093afd5e8b6fbe1fd4e894d48f845f | 29098 |
| `train-058.json` | ca21b008746d0f7e1c048c975b70d4d2c37678548d502f970b772b22294fb98d | 29223 |
| `train-059.json` | 725f625b593842e89b7e2944d57ee58b45819143902828e32c8a9d50ef09d9fc | 30798 |
| `train-060.json` | 376ad42e0ab59c72d922b6304629b38590d2cc39d1558b25f85b5c12c49f8500 | 29112 |
| `train-061.json` | dca1c902d7b875b5e5c0e72fa7e140ac754b111e136e1bad895582d04a73db80 | 29436 |
| `train-062.json` | 82524667f285100596f660023f7368493d7f1e4ebe97f99006f8e0348ceecc1a | 28324 |
| `train-063.json` | 96e1ee664a6d2b875b1242a66af5e0ff6cd9f8d5b6e8f31fff7b513765608681 | 28849 |
| `train-064.json` | 7b95e55bc63633147a50674e6cedb22a0aece1d6801359aa038db83ffc1804a6 | 29688 |
| `train-065.json` | 323a7d5bbce5e567ce93e28564ef1e33c5f213b6ef57fb75f9509a99bffcb91b | 30078 |
| `train-066.json` | 095052f22c533952bfb432254ec2cc7de31811237b59e112f9d8a4a7c096acd1 | 30193 |
| `train-067.json` | 35fcfd7c1fb610e347c9c4cbbe72fae55aa5eda7c12c0debcbe990ef75c6f682 | 30464 |
| `train-068.json` | 9ce7b102028fcbdd823924e1d27c7d7ae418328d36511d935c78b01e37bfc398 | 29915 |
| `train-069.json` | 321323b042c73619e059883e4869d64e4dff7a32caee6437c2cdfa3a5fc761f1 | 29924 |
| `train-070.json` | 14c5cafb7387ad1634e24234370cf1d4c5fc708075bb965c116ef35b3c04eae9 | 28796 |
| `train-071.json` | e172ff0a42860eebf2bcdf6a61e37d452b60e28b283a3a0de0eac3ec5580a213 | 28745 |
| `train-072.json` | 0fc64dfa31957a5b321f7570d99721f4ab6eb81a984dc916fd7d1ebc82c391cd | 30687 |
| `train-073.json` | 3ccef5f6f17d909a07fc87ad45f99f33593e4e4feb94ca1eaa62fecc52115e7e | 29582 |
| `train-074.json` | 5f663324220f536404096a21a0648151b1c535a3d183b6d6b1e62ebf9d2e6185 | 29086 |
| `train-075.json` | bbe70b8207568113107d1d0c8e443bf62005a13819c7976f05cbbab87ac02502 | 29930 |
| `train-076.json` | 4eabb80e764e1a16d870f45927eb2d8cb6d8441c4deb7f353e34e00bf0f65ead | 29690 |
| `train-077.json` | 8f9de63d9da75db22c514f745dd9947c40e8138d7ca441c2462543ab80438807 | 29293 |
| `train-078.json` | 143c16879679f9763df8d1784e6334c2a8c5cba015d47358831a4cc5bb11ce9b | 29140 |
| `train-079.json` | 3f5d57f00e2a0355704b5d64bc14cfcfc0f40fc4bc880b19cc4fe61b1eee64b1 | 29824 |
| `train-080.json` | 37269453360de8499906f099b16a2fda6dd11880ef6628f9ea3f15805feabd90 | 30816 |
| `train-081.json` | 170bc55fbc53e6847850915a9cac745c9225719033d14c2c31e7285a00d969ad | 28960 |
| `train-082.json` | af70bae63f24d496f341b765ca7886fb8fe2c88397dffb8a58e492834e5a87d2 | 31557 |
| `train-083.json` | 03b0d3c2736cf525755ac9c26b59605d8b3f20dd75391c09e668417b77f98b42 | 29655 |
| `train-084.json` | af17581584439ae4a0079c56d90cbc3e924a47677963a9c15da13fc2028331b2 | 29666 |
| `train-085.json` | ceaef3068d96e02094fb757f43d2ed75f973f8d36816bbaf8f1b70c439110ea4 | 31962 |
| `train-086.json` | a515164209c1ecf3e9b76e1afac907bf392bb1437a82b2e8014e71f9e1bdda56 | 29952 |
| `train-087.json` | ac12fb035a2964694c92d83cfca08e8af8a2fda0e5ce4cb913c542a8890f9988 | 29688 |
| `train-088.json` | 679e324470cdcabb3b8f782fc5bc88a5746f56b635be5f95ca8a389f827444b3 | 29103 |
| `train-089.json` | 14f2869afc4d91aff9abd2ce3314be6014789eab0756436010f3ba449ce4053e | 28949 |
| `train-090.json` | 3741747291d3d719376ae5a850c5b316f15702a7cbc1c5f20af3900e47c63e99 | 29929 |
| `train-091.json` | 582ce066c9c434cf53cbca69255676b43f422802a6107b78ea9c9108f7363a77 | 29723 |
| `train-092.json` | 2b0b67dfcd6ef8e69ea1a335ded10d8eb42cc65defd68218ba4302b2dac97db2 | 30192 |
| `train-093.json` | c4a0a05277e9f375298909b21ba07f2e8bf78e3beafbd1ee8b3011999bf76b4e | 29193 |
| `train-094.json` | 1f00a2250599beb844d0855decbbf61c9047a5bf1b449271f66c156ed33fadb1 | 29139 |
| `train-095.json` | 6da041eaeec4b2a6e31c389813f0481e4791976cf901542947609c0e0c272d08 | 29971 |
| `train-096.json` | dce4289ef0f2bc36e9d4bb70629a9f7e3d4856e8c46b277a7d74abdee1dac032 | 28982 |
| `train-097.json` | f2965968583d7bd310862cd190b86b719ebb949cd946dd92d6e1a006fe0df939 | 29324 |
| `train-098.json` | 5d85c4dcc57a49bccb5f0ee42832ae43a11583c5a925bb578848ed22f32f907d | 29911 |
| `train-099.json` | d4db9c1eaad6a24a58f5e67a21cb0a7b31c1640bd6dcaf95a33457f85f409972 | 30509 |
| `train-100.json` | beb5f2c4e1f245d721c00426c26422f740e7899efba076082771c549c645c520 | 29096 |
| `train-101.json` | 1edb7074baaf4a2097f22e24081cb0bcf50ca7cab135dc91f3c19a0148bc69a9 | 29255 |
| `train-102.json` | 3e0cc3d83340a428d4cc85c48aa781beab8f74b1bbdd00c4aa29b5cd8f7ff93e | 28767 |
| `train-103.json` | e82fe479a0f7bf229cb59f4374b40e6dd29fa6526fe934b9930ccaedfa9bfaaa | 30167 |
| `train-104.json` | 42f0eff43085cf68a115ce1a8fd79a0a6dbba710236b7d11d54ee01cc1215bd8 | 29810 |
| `train-105.json` | cc5ac6d27972fbbafd9f0d97198a518cd1c41a0b329e5c332d04624af77c6c03 | 30000 |
| `train-106.json` | d45432cbf21aefecceb3f7d4b4af79b6eeb20a6d314e3929063c8deb9cf14693 | 30075 |
| `train-107.json` | 437f6ba47429bfa248e7fe0c9bd5b50ae04c48327100e819661f7f2f49ce97e0 | 29833 |
| `train-108.json` | dbce0bcc6b8075b6915ba72e4328928b8c4d62ca72af1b754b4aabafff1419c3 | 29464 |
| `train-109.json` | d586c4329b72b6226988d490c0f673f8bc17fedaa737c3c204a323e742267197 | 30282 |
| `train-110.json` | 208c92d997e7dc14462df3618c49ea923bd309e827a62be123434d8c0cbd2b7a | 29640 |
| `train-111.json` | 90eb7d0490b583d5a32b6abbcb7b820bde4a908d3d374c94377b8a3a99473641 | 29949 |
| `train-112.json` | 6d3d6463ccb9ce775cf9def42454577f671772325ddf7336f3d9a08cd36d2b24 | 29702 |
| `train-113.json` | e251697f6d07cf4d523aaf005ab4349ec48a768203e1dce496e4e347217d8fc8 | 29331 |
| `train-114.json` | 0fe1a1768adcc6f118a3ca4c99ed2dcc46d081e094a5330b9399e48cf5bfea84 | 29922 |
| `train-115.json` | 8a9afd241d1315a4f87311736cadda29311040312dfa010cb3609ce282589db0 | 28708 |
| `train-116.json` | b00f1bab0d9ae02d95367113220ff88c54a8ad31f7d024ca8e7965ec391cf15e | 29759 |
| `train-117.json` | 18c748121cd740403f8a76baa8f5c6a0cb988f5459ef898bc17c13bc9fa81d97 | 29611 |
| `train-118.json` | 43c2f2a57beb0732aa2ff40fcdfbcf9b24c65b57a48c148b73f82246f6199c43 | 30415 |
| `train-119.json` | 1f8da86bf1c1a41633fbead55a2d6f695d67d5c03eba9b8896ec6bc109949562 | 30047 |
| `train-120.json` | d247e8828053a1c207057d90f327383fdbd2afef1e65e49bcc5b66d4687b665b | 30399 |
| `train-121.json` | d3708453455673a14ebfe79075b4348bc465f6b88de53a756f0c2562c9512b91 | 29239 |
| `train-122.json` | 6f69e231ab2e0a5c4bbd31a7949ce644995798e991abcaa16f483cc9aac25cd5 | 29386 |
| `train-123.json` | 748f5648270bb9fcd70d734d6962f5dbef35edef474bfab645357e5063a6c550 | 30875 |
| `train-124.json` | 608b3fc5d067441e88761a47192c5b00ec9b97c2e7cff3e477a3e7f00c5cdf08 | 29305 |
| `train-125.json` | a7b7d90d5fe321bb7aa4ac873617a78de46a0ceb200464b1885514a6dc24345d | 30343 |
| `train-126.json` | fe383ea3a981931c1f4f475ce561d1552b6d2f5b367bf7aaa8fe37400b30f514 | 29357 |
| `train-127.json` | 662f7e7c061946c8f3807533749aed9490fe1e29c75e6a94da125d272f7e7c64 | 29711 |
| `train-128.json` | 0fe5330c9ba37e9ff0ebb5618f92a112c9e1dc1879c40b26c411f07d7b60cc83 | 30314 |
| `train-129.json` | 60f39a0c3a348a526780c1fd4cddba774cf8375f4ed5766fa42a2d0ca7d9c3d9 | 28961 |
| `train-130.json` | aae2b9445061786613a1feae83ae73bb917c4df466de704b0217982d6c89256c | 30109 |
| `train-131.json` | 61390b57fdb17865f28c80629bf0dcc6b71f89672036582f796978e8188fdb81 | 29055 |
| `train-132.json` | 0c3ecbdf0cb724028f08caa19bf1506df0111001289e9125415d8683d63fcc1f | 28387 |
| `train-133.json` | b98c84c87c1a043dd636b277714e6a24ea29c46e06a36e5fa461b5d1cce97a58 | 30181 |
| `train-134.json` | 9771e624ea05ce39301f9ea53b9f283a606686a930b3921a57f22b08edac7f87 | 30706 |
| `train-135.json` | 1a65da3443bc2f648f30c4448f81c9e29aa3b16148f22415f5f225586946dce2 | 29707 |
| `train-136.json` | 103c9b7da44f65dcdbf7ca26167a2880fb4b55a36bac17a20ae1c4c0bd2e35e4 | 29712 |
| `train-137.json` | 57f34cc838a1cf4bc74868a1fd6fdb1fcff7e41981144f83e6cf9b4c94d72b7f | 30882 |
| `train-138.json` | aa7322bf28bbd4df0936444ef79f6e91ad1954120be46c3011d1edcbd46b8f87 | 29698 |
| `train-139.json` | a4fbe1303546f90d1cb14c210e04ced59066e76c392e2a49b6c35bd4f32ca949 | 29723 |
| `train-140.json` | 9372e5fcf85b94b6e15328a5ca9e8846864f4f12c49aac89998726d7f408f57f | 29794 |
| `train-141.json` | bdb0873c18f667810b00eccf476af987c64fe0faf7fa67aa812ecd91bd80d95f | 29358 |
| `train-142.json` | 169eb136238bf5c6b4822d1976a065adfd69c5f1344b699f7ef866a0753c6151 | 30154 |
| `train-143.json` | fa8fb1e2f5a0c7b95f4a02f29c5e79cd2ef9e4674dd3bd07ada495c5e785de6b | 29587 |
| `train-144.json` | 1f5c94e55201baf618f520ee6d8e071fdc420be9be7e80bc302b644697bc5826 | 29395 |
| `train-145.json` | f2e25b084896dc27258da6e74564efd0dcced73c358b1aa4025b8efe733e67b1 | 28660 |
| `train-146.json` | fd26f3882d7ae9100d66114587943d44eca8933627b1b95e990ec2d01e8d2299 | 29751 |
| `train-147.json` | 63357e713ba0178351f30812f4ecdfbdbb9fdb92d1afb99282cc103cfa99d3e5 | 29812 |
| `train-148.json` | 63a76d7be59ef0fc95251017178180b0c40fb9220a19f35e2c1e3161245c3fe5 | 30136 |
| `train-149.json` | c7b17f83ddb61a9fa134eecbaaeffd2ccb618b10d206e1e3d31db84e5944caa6 | 31279 |
| `train-150.json` | 3f97247ed23479c5c7cb955973bff5aa5272b0525a32d0a53cd0d74af850f38a | 28922 |
| `train-151.json` | 224ad06a5fc143540bdc65152102d7b91dfa4505bde6aa3a3f0aac9c1eaf6ad7 | 29289 |
| `train-152.json` | 90a876f8949f1126180f3dc1ec41e9f40017f4891d0acd1b94ea3ebc5098d43f | 29644 |
| `train-153.json` | d12eb90e3a7b5741a97685f6263fa8831d33790bcd771baf2ffe1923bf46fc47 | 29407 |
| `train-154.json` | 41bf9bbd81c51a31d58b8bfc671515586e722294cf2087051bb16965075641d8 | 30531 |
| `train-155.json` | d7a19f8cbaf028654ee86e81e07bc4239dc2ce0546d63d830f1fb83071bc49cb | 29693 |
| `train-156.json` | 08cdb1d282f0ab8374acd95ab5ffc3d30f798e738411a3eed3de6527b97cc2fe | 29374 |
| `train-157.json` | 347bbf7bfdf69c420a9394e38b910b689949faeeb6d3284deef8c3e1aabcbdf2 | 30524 |
| `train-158.json` | 781578b02b69348972d781a308933c67631e790f1fcafbe25129fc5185233c91 | 29078 |
| `train-159.json` | f7a6d98d514a1a9408e654f8dac8810973a744bdeed17ee2d8b2f82695b16a7b | 29985 |
| `train-160.json` | 0e13c17f6ac0dcd3ce3be9cb749d868a84151596a036e51422c4b7cc83209fd0 | 29839 |
| `train-161.json` | c5654e5b2491351c4eb0c7d339c3577363379df3f4d3cb45396ac409b1591f9f | 30040 |
| `train-162.json` | d2555c1fd09656129e66fbbc33613cf3d39266bd7b045538064aa9bf52816296 | 31500 |
| `train-163.json` | c419536944cf54aea773cbe539d19b7b8a9f401aabb25a3ca8fced1a0da0cc47 | 30149 |
| `train-164.json` | e6af98b11d9d126b7e4c909bb8b21d7809383e944e0b26a9c5359b77674036b6 | 29409 |
| `train-165.json` | f63234421e484c78bbf7683a61be0a0a44a7490753962f09aeb9be921148d9cc | 29175 |
| `train-166.json` | 05ca2433668f2ec1cbd7134f708da29f10d7a16d724be2fa8aa007aba854a4c0 | 28483 |
| `train-167.json` | af4bae1d51370cced73e6c47c3f7daa0dbe1269ffd53f8dcadd254948b812e1e | 29542 |
| `train-168.json` | 6117a9fd723765490764d0ca815bfc2b41d43fd17476dfe19b9ddba177a4305a | 29935 |
| `train-169.json` | 75e72f5617c531a8b9c1b1638b9415ac8cfab2fce738819ead3cf2217f503df6 | 29653 |
| `train-170.json` | 10456029c27746ee2e5cc616aad8b365dee0906b14ec24f2520aac1aa2eb630e | 30354 |
| `train-171.json` | c78e14e4060dc3b99320c9fdd5b39c22d50800a4027ce8e390501ec4845c7819 | 29836 |
| `train-172.json` | 75067d7a011c97c962c658c3d48ab6ea6c48277bcf880705c016eecb0e61313f | 29093 |
| `train-173.json` | d721404e5aafeca5a2d8b3318f58ffe3024885d9a15a37a9f1d0b68c13b40c16 | 30229 |
| `train-174.json` | 11b74ad4fc66f7bda22da9c9aff06121542a4279b5c7191ddefcd96b51618c3f | 29767 |
| `train-175.json` | affc1210c8ec9bb2b4d2bc347cbf768106f94448f2e22bad4eca1b45c4385234 | 29981 |
| `train-176.json` | 8d83a03b1e60356c9e7e4901cbe01e78a03f7950bfdca8c7d36f0b9cc8acb604 | 29607 |
| `train-177.json` | 890278b6eb47bfa7f572fb2357cf6352fac03c619a8d9214344846fddd4e5c13 | 29868 |
| `train-178.json` | 98c3c86876fb05d30b8f2a132a0de6d2574bda393647a99c79e06f888878fe7a | 29976 |
| `train-179.json` | 029ba7cdb0af787453036bc118b587a068922fe14126b8e3498757c126fa7a2c | 29446 |
| `train-180.json` | 9f2d07ddbe377c41ebb312fe79a57a29cf21084a45231a5cde709bcbaac732d1 | 30041 |
| `train-181.json` | 2af5e54bd5989d860ffaaf2b1314d25eaf61772b0042ff2505d929311552f46d | 30527 |
| `train-182.json` | 8d15d03d019539a4f5645caf39c0fc3d544757981ace6913b312d35a3ab2d77c | 29051 |
| `train-183.json` | d8f82fd1d33d407b784466935107dc442f8427f207d604f2fe27381cbac6c948 | 30093 |
| `train-184.json` | 9a1ea53245d66b0c1ec5347e232949b93ccd75a5e3614af04c9b33b5da34af2f | 30331 |
| `train-185.json` | fefd866bf79db8c5472c903be5c0f031f70fbff126c4cbdd5dc97b95c0a2e7ec | 30349 |
| `train-186.json` | 5dce34d818a4fa445c2099a35f7c9f04702ed71ced620b2f50be42d74fe4b084 | 30447 |
| `train-187.json` | 4431fa6bada2452dd531284478be093971f7e8b020cdb5913197f39315071b00 | 31093 |
| `train-188.json` | a10168c84799010f5121c2f37919ef2740db932fc2af8305b8ae38a4b29f6eda | 30690 |
| `train-189.json` | b5215080d891ab5ac91ca7699c9ebcbef9c0c3943f61f9962ffc9105eb4dbc35 | 28481 |
| `train-190.json` | aa46475e9d7bc9d5078cbc255102f0e1413bcd6f4badaa4be8f1e67becab14c2 | 28643 |
| `train-191.json` | 686fbe4acc677cb474f1d627505f36f17d42abb39a86110ff5b2e77b10611b0f | 29909 |
| `train-192.json` | 7cbf2de58d30547b602ce003389f708f289e16e2e5647451f39320881930ec51 | 28919 |
| `train-193.json` | 7e1e72d380963a8dd56c18c70e6bcc7a2bdefbf06957b101962f158df8a6f53b | 29512 |
| `train-194.json` | 5072cdeabc1d2462f2ca1a8eaa1e454aa125df02442b3a8d7791512353c50b46 | 28842 |
| `train-195.json` | 1dd8e09d8c626bc5d9e9462387b95b82c2ab27cfd444ef3527d71356136cc3ad | 29491 |
| `train-196.json` | e2ab0ab4194d829a7fba5a67c8743754613835c06cf86a097c77cd5b2ec4ee32 | 30262 |
| `train-197.json` | 25f0dc42341759b0715402032636d324fb362e6a1109263486cf487803d8148c | 30522 |
| `train-198.json` | 2f834c8f6d7ebce03e7feae70f428bc9a3181d0a868b3c227cd18e6b4681c030 | 29463 |
| `train-199.json` | 66586999f12ca74a42ec17b23964c3aa67fb3205f71db38b2b579729333c61b5 | 28818 |

## ClassLabel orders (verified)

Quote the exact `features` names arrays from the fetched JSON (first page
file, `.features[] | select(.type._type == "ClassLabel")`):

- **ag_news** `label`: `["World", "Sports", "Business", "Sci/Tech"]`
- **emotion** `label`: `["sadness", "joy", "love", "anger", "fear", "surprise"]`
- **xnli_en** `label`: `["entailment", "neutral", "contradiction"]`
- **thai_wisesight** `category`: `["pos", "neu", "neg", "q"]` (Plan 003;
  the builder's fixed criteria order)

Suites whose features carry NO ClassLabel (semantics verified another way):

- **sst5** — `label` is `Value(int64)` (no names array) plus a `label_text`
  string column. The int↔text pairing was verified empirically over all 600
  fetched test rows: `0=very negative, 1=negative, 2=neutral, 3=positive,
  4=very positive`.
- **prompt_injections** — `label` is `Value(int64)` (no names array); the
  dataset card README is a stub ("More Information needed"). Verified
  empirically: test labels are exactly {0, 1} (56 / 60). Semantics per the
  companion model card (`deepset/deberta-v3-base-injection`, which trains on
  this dataset): the task is INJECTION vs LEGITIMATE detection — `1 =
  injection, 0 = legitimate/benign`. Treat that mapping as card/model-derived
  (the features themselves carry no names).
- **massive_intent_en** — `label` and `label_text` are both plain strings
  (the MTEB port does not use a ClassLabel; in the fetched rows
  `label == label_text`, the intent slug). Distinct `label_text` values in
  the fetched test rows (sort -u): **30**
  (`alarm_query, alarm_remove, alarm_set, audio_volume_down,
  audio_volume_mute, audio_volume_other, audio_volume_up, datetime_convert,
  datetime_query, general_greet, general_joke, general_quirky,
  iot_cleaning, iot_coffee, iot_hue_lightchange, iot_hue_lightdim,
  iot_hue_lightoff, iot_hue_lighton, iot_hue_lightup, iot_wemo_off,
  iot_wemo_on, music_dislikeness, music_likeness, music_query,
  music_settings, news_query, play_music, takeaway_order, takeaway_query,
  weather_query`).

### typed_decisions — typed-decision vocabulary (no ClassLabel; a multi-head `choice`/`score`/`noul` decision set)

`gold` is a JSON object of typed decision heads — exactly the riir-reflex
`decision_wire` shapes. Head/type/label vocabulary extracted over ALL fetched
rows (test 400 + train 800, `sort -u`):

| workflow | head | type | labels |
|---|---|---|---|
| agent_trace_observability | action | choice | continue / human_review / observe / stop |
| agent_trace_observability | needs_review | noul | false / true |
| agent_trace_observability | outcome | choice | failure / harmful / partial / success |
| agent_trace_observability | risk | score | 0 / 1 / 2 / 3 |
| agent_trace_observability | urgency | score | 0 / 1 / 2 / 3 |
| customer_service | action | choice | answer_directly / close_no_action / escalate_to_human / execute_refund / request_information |
| customer_service | category | choice | account / billing / delivery / refund / technical |
| customer_service | churn_risk | score | 0 / 1 / 2 / 3 |
| customer_service | needs_human | noul | false / true |
| customer_service | urgency | score | 0 / 1 / 2 / 3 |
| invoice_processing | discrepancy_severity | score | 0 / 1 / 2 / 3 |
| invoice_processing | disposition | choice | approve / hold / manual_review / reject |
| invoice_processing | duplicate | noul | false / true |
| invoice_processing | matches_order | noul | false / true |
| invoice_processing | urgency | score | 0 / 1 / 2 / 3 |
| security_incidents | credential_compromise | noul | false / true |
| security_incidents | disposition | choice | close_benign / contain / investigate / monitor |
| security_incidents | severity | score | 0 / 1 / 2 / 3 / 4 |
| security_incidents | true_positive | noul | false / true |
| security_incidents | urgency | score | 0 / 1 / 2 / 3 |

Each `gold` head also carries `confidence`, `probabilities` (per label) and,
for score heads, a `score` value; the row's `state` column is a JSON string
with the full ticket context (account, thread, orders), `questions` /
`factors` / `label_agreement` carry the annotation provenance.

### banking77 (PolyAI reference — provenance only; the fetched mirror is mteb)

PolyAI's features per the hub's `dataset_infos.json` (NOT datasets-server —
see Gaps): `text` string + `label` ClassLabel with **77 names**
(`activate_my_card, age_limit, apple_pay_or_google_pay, atm_support,
automatic_top_up, balance_not_updated_after_bank_transfer,
balance_not_updated_after_cheque_or_cash_deposit, beneficiary_not_allowed,
cancel_transfer, card_about_to_expire, card_acceptance, card_arrival,
card_delivery_estimate, card_linking, card_not_working,
card_payment_fee_charged, card_payment_not_recognised,
card_payment_wrong_exchange_rate, card_swallowed, cash_withdrawal_charge,
cash_withdrawal_not_recognised, change_pin, compromised_card,
contactless_not_working, country_support, declined_card_payment,
declined_cash_withdrawal, declined_transfer,
direct_debit_payment_not_recognised, disposable_card_limits,
edit_personal_details, exchange_charge, exchange_rate, exchange_via_app,
extra_charge_on_statement, failed_transfer, fiat_currency_support,
get_disposable_virtual_card, get_physical_card, getting_spare_card,
getting_virtual_card, lost_or_stolen_card, lost_or_stolen_phone,
order_physical_card, passcode_forgotten, pending_card_payment,
pending_cash_withdrawal, pending_top_up, pending_transfer, pin_blocked,
receiving_money, Refund_not_showing_up, request_refund,
reverted_card_payment?, supported_cards_and_currencies, terminate_account,
top_up_by_bank_transfer_charge, top_up_by_card_charge,
top_up_by_cash_or_cheque, top_up_failed, top_up_limits, top_up_reverted,
topping_up_by_card, transaction_charged_twice, transfer_fee_charged,
transfer_into_account, transfer_not_received_by_recipient, transfer_timing,
unable_to_verify_identity, verify_my_identity, verify_source_of_funds,
verify_top_up, virtual_card_not_working, visa_or_mastercard,
why_verify_identity, wrong_amount_of_cash_received,
wrong_exchange_rate_for_cash_withdrawal`). Split sizes per the same file:
train 10003 / test 3080.

## Gaps

- ~~banking77~~ — **RESOLVED 2026-09-22** by retargeting the fetch at the
  parquet-backed mirror `mteb/banking77` (the reference's own bench_apps
  variant). `PolyAI/banking77` itself REMAINS unavailable on the
  datasets-server (`/rows` 404, `/splits` 500, script-based dataset; verified
  2026-09-22 across several hours) — recorded here so nobody re-attempts it
  expecting a different answer. The mirror's `label` column is an int64
  (NOT a ClassLabel) and carries `label_text` — option keys derive from the
  data (sorted unique `label_text`), which is exactly the bench_apps
  protocol. Everything else fetched clean.
- One transient 429 wall during the first banking77 train fetch (page 25);
  the idempotent re-run completed the split to the 4000-row cap.

## Notes

- `.raw/` is gitignored (`/.raw/` in `.gitignore`); the files are NOT
  committed. Re-running `scripts/fetch_datasets.sh` regenerates them: the
  script is idempotent (existing pages with the expected row count are
  skipped; `FORCE=1` refetches), so a re-run resumes exactly where it left
  off.
- **Byte-identity claim:** the datasets-server serves a versioned snapshot of
  each dataset, so same-revision `/rows` responses are stable — verified
  2026-09-22 by re-fetching `prompt_injections/test-000.json` and comparing
  blake3 digests (identical). The digests in this manifest are the
  regression pin: after any re-fetch, `b3sum` over the new files must
  reproduce them. If a dataset revision is re-generated upstream (or HF
  returns unstable row ordering), digests will legitimately change — update
  this manifest in the same commit and note the revision change.
- **Rate limiting:** unauthenticated datasets-server requests 429 after
  roughly ~50–130 rapid calls from one IP. The default politeness sleep is
  0.3s; long runs need `SLEEP=1.5` or higher (this fetch used `SLEEP=6` for
  the bulk of the files after two 429 walls), plus the script's built-in
  5s backoff retry per page. A 429 wall never loses work — re-run after a
  cooldown and the skip logic resumes.
- **Provenance:** all files fetched 2026-09-22 by `scripts/fetch_datasets.sh`
  (240 page files + 3 probe files, 23862 rows across 7 suites; the only
  failures across all runs were the two banking77 splits — see Gaps).
HEADER

echo "wrote $M"

**Fetched files (mteb mirror):**

| file | bytes | blake3 |
|---|---|---|
| `test-000.json` | 14341 | `d1c37dc3ade2413f7f305691c8748a5464f64797ddf951f668be31ad5ae43057` |
| `test-001.json` | 17067 | `3de77c645056e956b9ebed272e5635ddfbd0dd07dc7fe8f554b61a4e6f279814` |
| `test-002.json` | 16613 | `86f6d29b1b78bdad5486dd00fd038214f9f086089a79976a9fa4d602d2744192` |
| `test-003.json` | 14445 | `b2d5ccabb8dd0b8ec923e583fec57be8aed23a4cf852e2a56b92a50920d41727` |
| `test-004.json` | 14610 | `7fb4a2b0f78645877d65de21414a863c45100965f36edebe9157c2ac296a8cea` |
| `train-000.json` | 14379 | `e88c59a6d286416278529a9c115b2a2b1bf36e69bdebf47b1a395e1228d1dc9d` |
| `train-001.json` | 14847 | `9f254c6d463de9a1e3a64b4b662a0a765fa254123b8a9a33fb9a89e0bcf09d08` |
| `train-002.json` | 14868 | `e714c1ac4412acc955d030f27df147ddc858b444cbfc2b4e27fae1e2857a3e84` |
| `train-003.json` | 13998 | `30a4bc1799ecc96203911e3829ea91e87f873dc53d11ffff1b141c3a195fcfcb` |
| `train-004.json` | 18735 | `d3ac49b9c5036aeeeaa6860743f1fae1d3e76b0392541af557307f0314326447` |
| `train-005.json` | 18287 | `b9b71e337ddb3745142f166cb74825d8a1def952217faec7edc739ad78b21ada` |
| `train-006.json` | 17209 | `8475636dcd5a3f01bcbf2dc3fee16c23e6528d1a852d4c3d7d76145ab165142f` |
| `train-007.json` | 18775 | `b2bbfa73035b620c73da1cf90dbb2f4af5dc9f5f13abad3be5e6c937d99f5312` |
| `train-008.json` | 18219 | `c44f2de6fb97d55f253927e848d3aa9e2b62b5ecd17d3eba7629d76dcea9bc8f` |
| `train-009.json` | 15502 | `4aea8c5ba21a04bed2655f8079bd3ab6b89cd49352f865c2a2ae70e61164305a` |
| `train-010.json` | 15045 | `c1d28cff5a5cda8b289ae006b42ee938e73d99921a98c835d8d4f75cd8389f03` |
| `train-011.json` | 14812 | `bfc1c0c2901074b0c6c95e85ecfcb558f9a6968ccb4f1029c435b79fb2bb2134` |
| `train-012.json` | 14581 | `ec360ca8fec3f505d15243fff70eb43b45f361a2521c8290a9aa1f3f95ea9cae` |
| `train-013.json` | 15156 | `ef2918daf1b23c968c816ac718c9627cef1c2a755d6d289252e710aa8a480b49` |
| `train-014.json` | 15194 | `086416b09391fcf482c726b87c2c0e656289b17b5346324e22d4367d792bafe5` |
| `train-015.json` | 14519 | `e96088570d9c423f9526f5626a9370fdb3084d3532c838da7c0aac47b68e4458` |
| `train-016.json` | 13921 | `937e54f2c2b164446cbf8086cc815e6562b6a16561328e323518464d82bbf8dd` |
| `train-017.json` | 14965 | `f09815dfd65ad92498379de0398bbd4f37681cea76b11644ebf13f4497fb1f5d` |
| `train-018.json` | 15956 | `4d5294f174d7a0a57ba3d58c7ac3792b2767a423dd1fbb17e2e3e660caabc579` |
| `train-019.json` | 15925 | `b250af1b1725a050aecdff3228c2c2e71db05bdb136c76d3db5eeaaa175329d0` |
| `train-020.json` | 16317 | `5f3c6c72951ae0cd12892092619ad07fbdb4aa79333e8cc8f94f3d87c78ef92a` |
| `train-021.json` | 17259 | `219b5c4d1a0b5ab58b0e66247e4bca18ace95631036320b1ff3ca1d941c32629` |
| `train-022.json` | 14882 | `c7b0380eb3a21f751fccf6cb4b35990b957c080f9823ccf6e8f5ee4db6251199` |
| `train-023.json` | 17336 | `4c32a427ce6327512887f7bc6ce2239271fd10f281a24be2f80da2fe41f8d794` |
| `train-024.json` | 18475 | `e5f53a2cfcfb4892bed523850ab14f206f4f832ad176e9b3b86287dc6f817ef0` |
| `train-025.json` | 18083 | `30afc256ab0072d268c658e241870ba9b599e55517102c3e36964d3906cc1cc9` |
| `train-026.json` | 17935 | `8f0218bf2ae57e82a3a7585ad69600f83749996011dccd20ac41cf216fe11a6d` |
| `train-027.json` | 20663 | `76a4e1d6e43827614a59002f18f4adc7ab80594fcabbc7fceca5e4c5d538f27b` |
| `train-028.json` | 19894 | `da56f041246fed6ee9df7972e3773e3aef87c0dd8db360dcf8fb7e9a9b0b2a8d` |
| `train-029.json` | 16476 | `39bd8f1e8f46c192b47a34e0476a597c88c49b9b02d23ffa90b4711618251efe` |
| `train-030.json` | 14700 | `e3108b6200d8319b89564325260b00d9cb0b32661497da80a22a171c849518e7` |
| `train-031.json` | 15368 | `a7af8c852cde9e1aef0817cd659e22bfb57b9b97150cc4961f9e1cbeb7f9a324` |
| `train-032.json` | 17064 | `db948e1a94479e8a0cde6e3e6b75bccf836870dab86215ddd2dc01705c0cc1ba` |
| `train-033.json` | 20588 | `7990d0535446b243eb87249c13d27ce49628f17de73c571c98363dbafa4a68d5` |
| `train-034.json` | 20294 | `bb1880ff676e923844483f9396e628cd367189376235aff6895e4524cf09c958` |
| `train-035.json` | 18400 | `1a7d534c57b90e8c1217ef2112e9dbf776b369af83af70334d5aa9308f50eaee` |
| `train-036.json` | 16665 | `717e6582d79eeb0cb7b81c53e30e807e3988edcd5a5ce1b5be5f7319e8f1acee` |
| `train-037.json` | 14925 | `f9e54d14be31fd759b41816b927cf64b4118a5d15f309590e96ef1b9388390bc` |
| `train-038.json` | 15467 | `c149ffadd9e7bd5f13f3c2397549cc145c35050ba7e32c14c0e1d816d6fa5b6c` |
| `train-039.json` | 16644 | `49f90e92d91506419e0f92f3d426e3620bd45029333313d5125b43417e575ef6` |

## Full train pull (Issue 038 T2 / Issue 039 — the fair posture, 2026-09-26)

`OUT=.raw/datasets_t20k TRAIN_CAP=20000 SLEEP=4 scripts/fetch_datasets.sh`
(the canonical 4000-row pages were copied in first; the fetch resumes past
them). The harness reads it with `--datasets-dir .raw/datasets_t20k`. Test
splits and every non-train page are byte-identical to the canonical set
above. The 4000-row cap truncated the label universe on the label-sorted
banking77 (32/77) and massive (42/60) mirrors (Issue 039), so published
modelless rows use this pull from Bench 051 on.

Aggregate digest = `b3sum` over the sorted `b3sum train-*.json` lines of
that suite (re-derive with `cd <suite> && b3sum train-*.json | sort -k2 | b3sum`):

| suite | train pages | train rows | aggregate blake3 |
|---|---|---|---|
| ag_news | 200 | 20000 (of 120000) | `614e091d968c6d34223f0948b084a1c38380f1c1fd95180f78a24c70174e85b4` |
| emotion | 160 | 16000 (all) | `d02dff7da0b759abfafa3d32370d1a8e5d664b866fbf76ce3f13641d226b19a2` |
| sst5 | 86 | 8544 (all) | `ebfb0317ea5e171d52f743a8a2d4768a28d9ce60e7e0894b82490f324d9db07a` |
| banking77 | 100 | 9993 (all) | `f511dac4c61bb8d9cac376c3ed9cbeb4a7d53099b01ab652e1972361dd6ee848` |
| massive_intent_en | 116 | 11514 (all) | `9a6844027f1b1aef2bdf827e8bfd186d7c2ae251f7f8482ce5c12185e1b360c3` |
| xnli_en | 200 | 20000 (of 392702) | `5804bb68e1bd16af0288cc9698ae08ac54718c414aca88993b5b30914d922a38` |
| typed_decisions | 12 | 1200 (all; Issue 052 lift) | `83ea62fb0b240f66e959850c625d4337176ef068dd03d6c3d862e2f4d947d88a` |

prompt_injections is unchanged (its cap already covers the train split:
546 of 546). typed_decisions was lifted to the full train split by
Issue 052 (Bench 078, 2026-09-28): TRAIN cap 800 → 1200 — the 800-row cap
had stopped inside the security_incidents train block (offsets 900–1199)
plus invoice rows 200–299. Pages 000–007 byte-verified unchanged; the four
new pages ride the typed section above. Aggregate digest recomputed:

## ag_news full-pull measurement dir (issue 041 / Bench 065 — NOT a column)

`SUITES=ag_news OUT=.raw/datasets_agnews_full TRAIN_CAP=120000
scripts/fetch_datasets.sh` (2026-09-27, the SUITES filter added for it).
A measurement SIDECAR for the volume-lever question — the canonical
published basis stays the t20k pull above (Bench 065 measured the lever
NEGATIVE: +0.25 pt at the count tables, ±0.00 at the corpus cap; the
harness never reads this dir without `--datasets-dir`).

- 1200 train pages (120,000 rows = the source total) + the 4 test pages
  (byte-identical to the canonical table above).
- train-000..199 are byte-identical to the t20k rows of the same names
  (204/204 verified dir-vs-dir); pages 0200–1199 are new bytes.
- Whole-dir fold (b3sum over the sorted per-file b3sum lines, all 1204
  pages): `59617a3ec0335ec6abc9ae5d554b2c64374fe2ea75c3df518fce522ca7d9d702`.
- Per-file BLAKE3 table: `.benchmarks/065_agnews_full_volume/ag_news_full_digests.b3.txt`.
- Limiter note: the datasets-server walls after ~300 pages per burst at
  sustained pull; 4 burst→cooldown→resume cycles (the skip logic resumes
  exactly, final burst 0 failed).

## Sampling law (Issue 039 T2 — the stratified split, Bench 052)

The FILES on disk are unchanged by this section — the sampling law is how
the HARNESS consumes them (`src/harness/suites.rs::stratified_split`):

- **Test sample** = label-stratified round-robin over the WHOLE test split
  (budget = the registry test cap; first-appearance label order, dataset
  order within each label; deterministic, no RNG, no seed). The first-N law
  it replaces was a label PREFIX on label-sorted mirrors: banking77's 500
  first test rows spanned 13 of 77 labels, massive's 300 spanned 30 of 60
  (measured, Issue 039) — every lane scored a non-representative slice and
  the questions' remaining labels carried only their self-doc fallback
  corpus. `budget == 0` (or ≥ all rows) is the identity split — the
  uncapped suites (typed_decisions, prompt_injections) are byte-identical
  to the first-N law.
- **Cal slice** = the same stratified law over the train rows. The old
  first-N cal slice was label-clustered (2/4 ag_news labels, ~2/77
  banking77, in its first 200 rows — the measured reason the Issue-013
  selection instruments exist).
- **Corpus pool** = the train rows MINUS the stratified cal front
  (excluded by CONSTRUCTION via the split's `rest` envelope, not by
  position). The positional `train[cal_cap..]` cut it replaces was only
  correct while the cal slice was the first-N prefix — on a label-clustered
  train mirror it also ORPHANED every label whose whole block sat inside
  the prefix (banking77's first label had ZERO pool docs at the full pull).
- **Fallback guard (Issue 039 T3)**: an engine build whose option label has
  NO train docs in the pool now discloses the label names loud (stderr +
  the suite's results row + the markdown ⛔ line) — the self-doc fallback
  count that silently inflated pre-T1 numbers is never silent again.

Case COUNTS are unchanged (the budget is the registry cap); the labels the
cases cover, the cal slices, the corpus pools — and therefore every
published number — move from Bench 052 on. The readout-selection lever
(T4) was measured NEGATIVE at 052 and demoted to a report-only candidate
table; the shipped Dispatch readout runs everywhere.

**Cross-host byte discipline (Issue 039 T5):** the 4090's
`.raw/datasets_t20k` copy was verified byte-identical to this tree before
the 2026-09-27 re-run — 977/977 files, sorted SHA256-manifest diff empty
(the box's own re-fetch had died on curl 429s; the queued "already copied
there" premise was stale). Copy-then-verify is the standing posture for
cross-host runs: same bytes is a claim, not an assumption.
