//! Issue 081 T2b probe: measure the corpus-distance gate's geometry for a
//! memory-control corpus — where does a REAL request sit relative to the
//! gate's midpoint, and how does that compare to the corpus docs themselves?
//!
//! Usage (from the repo root, against a corpus dir):
//!   cargo run --release --example jevmem_gate_probe -- <corpus-dir> <utterance>...
//! Each utterance is rendered into a real observation-question request
//! (canonical JSON state + the admission questions' criteria) and measured
//! against every domain's gate. The docs' own rows are measured too (the
//! in-corpus reference band).

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: jevmem_gate_probe <corpus-dir> [utterance...]");
        std::process::exit(2);
    };
    let utterances: Vec<String> = args.collect();
    let corpus = riir_reflex::corpus::load_dir(&PathBuf::from(&dir)).expect("corpus loads");
    let cfg = riir_reflex::engine::EngineConfig {
        score_threshold: 0.0,
        ..riir_reflex::engine::EngineConfig::default()
    };
    // The serve's EMBED_DIM posture; 2-domain corpora are the probe shape.
    type Eng<const N: usize> =
        riir_reflex::engine::DecisionEngine<N, { riir_reflex::embed::EMBED_DIM }>;
    let eng: Eng<2> = riir_reflex::engine::DecisionEngine::build_specs(corpus.specs.clone(), cfg)
        .expect("engine builds");
    println!("domains: {:?}", eng.domain_names());
    let embedder = riir_reflex::embed::Embedder;
    // Doc rows: re-embed each doc (the same bytes the spec build embedded).
    for (di, name) in eng.domain_names().iter().enumerate() {
        let gate = eng.gate(di);
        println!("domain {name}:");
        // A real observation request over each utterance (the systemone ctx
        // shape: state JSON + instructions + criteria).
        let yes = "A fact, experience, plan, preference, procedure or unresolved need attributable to a participant.";
        let no = "Only a greeting, acknowledgement, generic pleasantry, or content with no recallable detail.";
        let q = "Does `observation` contain a specific detail worth retaining for future conversation recall?";
        for utt in &utterances {
            let state = format!(
                "{{\"exact_duplicate\":false,\"observation\":{utt:?},\"recent_memories\":[]}}"
            );
            let ctx = format!("{state}\n{q}\ntrue: {yes}\nfalse: {no}");
            let mut emb = [0.0f32; riir_reflex::embed::EMBED_DIM];
            embedder.embed_into(ctx.as_bytes(), &mut emb);
            let ms = gate.max_similarity(&emb);
            let conf = gate.abstain_confidence(&emb);
            println!("  request: max_sim={ms:.4} abstain_conf={conf:.4}  <- {utt:.48}");
        }
        // The docs' own band (first few docs re-embedded).
        let dir_dom = PathBuf::from(&dir).join(name);
        let mut docs: Vec<String> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir_dom) {
            let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            paths.sort();
            for p in paths.iter().take(3) {
                if let Ok(t) = std::fs::read_to_string(p) {
                    docs.push(t.trim().to_string());
                }
            }
        }
        for (i, d) in docs.iter().enumerate() {
            let mut emb = [0.0f32; riir_reflex::embed::EMBED_DIM];
            embedder.embed_into(d.as_bytes(), &mut emb);
            let ms = gate.max_similarity(&emb);
            let conf = gate.abstain_confidence(&emb);
            println!("  doc[{i}]:    max_sim={ms:.4} abstain_conf={conf:.4}");
        }
    }
}
