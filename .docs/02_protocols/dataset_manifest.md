# Benchmark dataset manifest — Plan 603 T1.5 fetch layer

Raw row snapshots for the riir-reflex benchmark harness (G1/G2 decision
benches). Fetched from the HF datasets-server `/rows` API (page size 100,
offset paging) by [`scripts/fetch_datasets.sh`](../scripts/fetch_datasets.sh)
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
  300/1000), `train` splits capped at 4000 (typed_decisions train 800);
  `cap=all` suites page until the API returns fewer than 100 rows.

## Suites

### 1. typed_decisions — `LocalLLaMA/typed-decisions` (config `all`)

Config `all` verified via the `/splits` probe (the other configs are the four
single-workflow views: `agent_trace_observability`, `customer_service`,
`invoice_processing`, `security_incidents`). `test` fetched to exhaustion:
400/400 rows. `train` capped at 800 (dataset total 1200).

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

## ClassLabel orders (verified)

Quote the exact `features` names arrays from the fetched JSON (first page
file, `.features[] | select(.type._type == "ClassLabel")`):

- **ag_news** `label`: `["World", "Sports", "Business", "Sci/Tech"]`
- **emotion** `label`: `["sadness", "joy", "love", "anger", "fear", "surprise"]`
- **xnli_en** `label`: `["entailment", "neutral", "contradiction"]`

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

typed_decisions and prompt_injections are unchanged (their caps already
cover their train splits: 800 of 1200 by design, and 546 of 546).

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
