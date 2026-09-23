//! Dependency-free language/script detection (port of `laya/lang.py` at
//! the pinned sha) — the pure logic that routes between the checkpoints.
//!
//! Routing needs ONE decision: is this English Latin text, or something
//! the English checkpoint cannot read? (Measured upstream: the English
//! checkpoint COLLAPSES to near-random on non-Latin scripts — Hindi 0.100
//! on 20-option intent vs 0.050 random — while reporting high confidence.
//! Script is the primary signal; the Latin language guess is a
//! stopword/diacritic heuristic and is best-effort BY DESIGN.)
//!
//! Honesty laws carried from the reference (each mirrors a recorded
//! upstream defect — #35 Romanian-to-English misrouting, the Turkish
//! "para"→Spanish guess): an unidentified Latin language is NOT English;
//! a 0-0 stopword tie names no language; one shared function word is
//! never a detection; state KEYS are ignored (they are usually English).

use serde_json::Value;

/// Unicode blocks the English checkpoint (50k English BPE) cannot read.
#[rustfmt::skip]
const SCRIPT_RANGES: &[(&str, &[(u32, u32)])] = &[
    ("greek",      &[(0x0370, 0x03FF), (0x1F00, 0x1FFF)]),
    ("cyrillic",   &[(0x0400, 0x052F), (0x2DE0, 0x2DFF), (0xA640, 0xA69F)]),
    ("armenian",   &[(0x0530, 0x058F)]),
    ("hebrew",     &[(0x0590, 0x05FF)]),
    ("arabic",     &[(0x0600, 0x06FF), (0x0750, 0x077F), (0x08A0, 0x08FF), (0xFB50, 0xFDFF), (0xFE70, 0xFEFF)]),
    ("devanagari", &[(0x0900, 0x097F), (0xA8E0, 0xA8FF)]),
    ("bengali",    &[(0x0980, 0x09FF)]),
    ("gurmukhi",   &[(0x0A00, 0x0A7F)]),
    ("gujarati",   &[(0x0A80, 0x0AFF)]),
    ("oriya",      &[(0x0B00, 0x0B7F)]),
    ("tamil",      &[(0x0B80, 0x0BFF)]),
    ("telugu",     &[(0x0C00, 0x0C7F)]),
    ("kannada",    &[(0x0C80, 0x0CFF)]),
    ("malayalam",  &[(0x0D00, 0x0D7F)]),
    ("sinhala",    &[(0x0D80, 0x0DFF)]),
    ("thai",       &[(0x0E00, 0x0E7F)]),
    ("lao",        &[(0x0E80, 0x0EFF)]),
    ("tibetan",    &[(0x0F00, 0x0FFF)]),
    ("myanmar",    &[(0x1000, 0x109F)]),
    ("georgian",   &[(0x10A0, 0x10FF)]),
    ("ethiopic",   &[(0x1200, 0x137F)]),
    ("khmer",      &[(0x1780, 0x17FF)]),
    ("hangul",     &[(0x1100, 0x11FF), (0x3130, 0x318F), (0xAC00, 0xD7AF)]),
    ("kana",       &[(0x3040, 0x309F), (0x30A0, 0x30FF), (0x31F0, 0x31FF)]),
    ("han",        &[(0x3400, 0x4DBF), (0x4E00, 0x9FFF), (0xF900, 0xFAFF)]),
];

/// Weighted function words per Latin-script language. Latin languages
/// overlap heavily (de/la/le/un/e/que), so each hit is weighted by count
/// and a margin over English is required before calling anything
/// non-English.
#[rustfmt::skip]
const STOP: &[(&str, &[&str])] = &[
    ("en", &["the", "and", "is", "are", "was", "were", "to", "of", "in", "for", "with", "that",
             "this", "it", "you", "have", "has", "not", "but", "on", "at", "be", "as", "from",
             "will", "can", "would", "there", "their", "what", "which", "please", "we", "i"]),
    ("fr", &["le", "la", "les", "des", "une", "est", "pour", "dans", "que", "qui", "avec", "sur",
             "pas", "plus", "nous", "vous", "être", "cette", "mais", "sont", "ont", "aux", "ce"]),
    ("de", &["der", "die", "das", "und", "ist", "ein", "eine", "den", "dem", "nicht", "mit", "für",
             "auf", "von", "zu", "sich", "auch", "werden", "wurde", "haben", "sind", "oder", "aber"]),
    ("es", &["el", "los", "las", "que", "por", "con", "para", "una", "es", "se", "del", "como",
             "pero", "son", "está", "este", "esta", "todo", "más", "muy", "hay", "sus"]),
    ("pt", &["os", "as", "que", "em", "um", "uma", "para", "com", "não", "é", "se", "do", "da",
             "dos", "das", "mas", "são", "está", "este", "esta", "muito", "pelo", "pela"]),
    ("it", &["il", "lo", "gli", "che", "di", "per", "con", "non", "è", "si", "del", "della", "sono",
             "questo", "questa", "anche", "come", "più", "sono", "nella", "alla"]),
    ("nl", &["het", "een", "van", "is", "op", "te", "dat", "niet", "met", "voor", "zijn", "aan",
             "door", "maar", "ook", "worden", "deze", "naar", "wordt"]),
    // Romanian words its Romance neighbours do not share, so `ro` cannot
    // steal a French/Spanish/Italian/Portuguese state; the diacritic
    // signal carries the rest.
    ("ro", &["și", "să", "este", "sunt", "care", "pentru", "din", "dar", "după", "până", "fără",
             "ale", "lui", "în", "fost", "acum", "vreau", "trebuie", "foarte", "acest", "această",
             "acesta", "aceasta", "mi", "ți", "vă", "nu"]),
];

/// Letters ordinary English does not use — the signal that catches a
/// Latin-script language we hold no stopwords for at all.
const NON_EN_DIACRITICS: &str = "àâäãáåçéèêëíìîïñóòôöõøúùûüýÿßæœ\
     ăâîșțşţ\
     ąćęłńśźż\
     čďěňřšťůž\
     őű\
    ğı\
     āēģīķļņūž\
     đ";

/// A diacritic rate above this is evidence the text is not English even
/// when no stopword list matches.
pub const NON_EN_DIACRITIC_RATE: f64 = 0.02;

/// Collect the string leaves of a state (str / dict / list), depth-capped —
/// detection must see real CONTENT, never the (usually English) keys.
fn iter_text<'a>(state: &'a Value, depth: usize, out: &mut Vec<&'a str>) {
    if depth > 6 {
        return;
    }
    match state {
        Value::String(s) => out.push(s),
        Value::Object(map) => {
            for v in map.values() {
                iter_text(v, depth + 1, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                iter_text(v, depth + 1, out);
            }
        }
        _ => {}
    }
}

/// Flatten a state into the text used for detection (keys ignored — they
/// are usually English). Port of `state_text` (`max_chars` 4000).
pub fn state_text(state: &Value) -> String {
    let mut leaves = Vec::new();
    iter_text(state, 0, &mut leaves);
    leaves.join(" ").chars().take(4000).collect()
}

fn script_of_char(cp: u32) -> Option<&'static str> {
    if cp < 0x0250 || (0x1E00..=0x1EFF).contains(&cp) {
        return Some("latin");
    }
    for (name, ranges) in SCRIPT_RANGES {
        if ranges.iter().any(|(lo, hi)| *lo <= cp && cp <= *hi) {
            return Some(name);
        }
    }
    None
}

/// Fraction of alphabetic characters per detected script.
pub fn script_profile(text: &str) -> std::collections::HashMap<String, f64> {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for ch in text.chars() {
        if !ch.is_alphabetic() {
            continue;
        }
        if let Some(name) = script_of_char(ch as u32) {
            *counts.entry(name).or_insert(0) += 1;
        }
    }
    let total: usize = counts.values().sum();
    if total == 0 {
        return std::collections::HashMap::new();
    }
    counts
        .into_iter()
        .map(|(k, v)| (k.to_string(), v as f64 / total as f64))
        .collect()
}

/// Dominant script of `text`: `'latin'`, `'han'`, … or `'unknown'` when
/// there are no letters. Ties resolve by the table's first-then-latin
/// insertion order exactly like Python's `max` (first key wins a tie —
/// the counts dict holds `latin` LAST, so a true latin/other tie names
/// the other script; lengths equal ⇒ probability ~0 and the reference
/// has the same shape).
pub fn detect_script(text: &str) -> String {
    let prof = script_profile(text);
    if prof.is_empty() {
        return "unknown".to_string();
    }
    let mut best: Option<(&str, f64)> = None;
    // Python's max over dict items iterates INSERTION order (latin first —
    // it is seeded into the counts dict first there), so on a tie the
    // earliest-inserted key wins: replicate by strict `>` comparison over
    // the same order.
    for (name, _) in SCRIPT_RANGES {
        if let Some(v) = prof.get(*name) {
            let better = match best {
                None => true,
                Some((_, bv)) => *v > bv,
            };
            if better {
                best = Some((name, *v));
            }
        }
    }
    let latin_v = prof.get("latin").copied().unwrap_or(0.0);
    match best {
        Some((name, bv)) if bv > latin_v => name.to_string(),
        _ => "latin".to_string(),
    }
}

/// Evidence behind the Latin-script language guess.
#[derive(Debug, Clone)]
pub struct LatinProfile {
    /// Best-effort language code; `None` = undecided (a DIFFERENT answer
    /// from "English" — only one of them is safe for the English
    /// checkpoint).
    pub language: Option<String>,
    pub english_hits: usize,
    pub diacritic_rate: f64,
    pub looks_non_english: bool,
}

/// Port of `latin_profile`: function-word scores per language with the
/// reference's exact margins.
pub fn latin_profile(text: &str) -> LatinProfile {
    fn score_of(lg: &str, words: &[String]) -> usize {
        let Some((_, list)) = STOP.iter().find(|(n, _)| *n == lg) else {
            return 0;
        };
        // Python counts OCCURRENCES (`sum(1 for w in words if w in sw)`),
        // not unique matches — a repeated function word is repeated
        // evidence.
        words.iter().filter(|x| list.contains(&x.as_str())).count()
    }

    let words: Vec<String> = text
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    let lowered = text.to_lowercase();
    let diac = lowered
        .chars()
        .filter(|c| NON_EN_DIACRITICS.contains(*c))
        .count();
    let diac_rate = diac as f64 / lowered.len().max(1) as f64;
    let non_english = diac_rate >= NON_EN_DIACRITIC_RATE;
    if words.len() < 4 {
        return LatinProfile {
            language: None,
            english_hits: 0,
            diacritic_rate: diac_rate,
            looks_non_english: non_english,
        };
    }
    let score_of = |lg: &str| score_of(lg, &words);
    let en = score_of("en");
    let mut best_lg: Option<&str> = None;
    let mut best = 0usize;
    for (lg, _) in STOP {
        if *lg == "en" {
            continue;
        }
        let s = score_of(lg);
        // Python's max-with-default keeps the FIRST maximal entry; a 0-0
        // tie is no evidence for any language.
        if s > best {
            best = s;
            best_lg = Some(lg);
        }
    }
    if best == 0 {
        best_lg = None;
    }

    let lang = if let Some(lg) = best_lg {
        if best >= std::cmp::max(2, en + 2) {
            // a non-English language needs a clear margin over English
            // function words
            Some(lg.to_string())
        } else if non_english && best >= std::cmp::max(2, en) {
            // two hits required even with diacritics — one shared function
            // word ("para" in Turkish text) once named Spanish on the
            // diacritics alone
            Some(lg.to_string())
        } else {
            None
        }
    } else if en > 0 && !non_english {
        Some("en".to_string())
    } else {
        None
    };
    LatinProfile {
        language: lang,
        english_hits: en,
        diacritic_rate: diac_rate,
        looks_non_english: non_english,
    }
}

/// Best-effort language code for Latin-script text, `None` when undecided.
pub fn guess_latin_language(text: &str) -> Option<String> {
    latin_profile(text).language
}

/// Full detection result for a state (`analyse`).
#[derive(Debug, Clone)]
pub struct Detection {
    pub script: String,
    pub script_profile: std::collections::HashMap<String, f64>,
    pub language: Option<String>,
    pub is_english: bool,
    /// Undecided is NOT English — the flag is the honesty surface.
    pub language_undecided: bool,
    pub diacritic_rate: f64,
    pub non_latin_fraction: f64,
}

/// Port of `analyse`: the decision tree the router consumes.
pub fn analyse(state: &Value) -> Detection {
    let text = state_text(state);
    let prof = script_profile(&text);
    let script = detect_script(&text);
    let non_latin = if prof.is_empty() {
        0.0
    } else {
        ((1.0 - prof.get("latin").copied().unwrap_or(0.0)) * 1e4).round() / 1e4
    };
    if script == "unknown" {
        return Detection {
            script: "unknown".into(),
            script_profile: prof,
            language: None,
            is_english: true,
            language_undecided: true,
            diacritic_rate: 0.0,
            non_latin_fraction: 0.0,
        };
    }
    if script != "latin" {
        return Detection {
            script,
            script_profile: prof,
            language: None,
            is_english: false,
            language_undecided: true,
            diacritic_rate: 0.0,
            non_latin_fraction: non_latin,
        };
    }
    let prof_lat = latin_profile(&text);
    let lang = prof_lat.language.clone();
    // Undecided is not English. When nothing identifies the language,
    // non-English letters are enough to prefer the multilingual
    // checkpoint; text with no such letters (including short English)
    // still goes to the English one.
    let undecided = lang.is_none();
    let english = lang.as_deref() == Some("en") || (undecided && !prof_lat.looks_non_english);
    Detection {
        script: "latin".into(),
        script_profile: prof,
        language: lang,
        is_english: english,
        language_undecided: undecided,
        diacritic_rate: (prof_lat.diacritic_rate * 1e4).round() / 1e4,
        non_latin_fraction: non_latin,
    }
}

/// True when the English checkpoint can be expected to read this state.
pub fn is_english(state: &Value) -> bool {
    analyse(state).is_english
}

/// Behavioral pins mirrored from the reference's own `tests/test_router.py`
/// (pure logic — no weights): script detection, the honesty laws, the
/// state flattening, and the KNOWN GAP kept visible on purpose.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn script_detection_matches_reference_suite() {
        let cases: &[(&str, &str, &str)] = &[
            (
                "english",
                "The customer was charged twice and wants a refund.",
                "latin",
            ),
            ("armenian", "Հայերեն", "armenian"),
            ("armenian uppercase", "ՀԱՅԵՐԵՆ", "armenian"),
            ("armenian punctuation only", "։֊", "unknown"),
            (
                "french",
                "Le client a été facturé deux fois et demande un remboursement.",
                "latin",
            ),
            (
                "hindi",
                "ग्राहक से दो बार शुल्क लिया गया और वह धनवापसी चाहता है।",
                "devanagari",
            ),
            (
                "japanese",
                "お客様は二重に請求されたため返金を希望しています。",
                "kana",
            ),
            ("chinese", "客户被重复扣款要求退款", "han"),
            ("korean", "고객이 두 번 청구되어 환불을 원합니다", "hangul"),
            (
                "arabic",
                "تم خصم المبلغ مرتين من العميل ويريد استرداد الأموال",
                "arabic",
            ),
            ("tamil", "வாடிக்கையாளரிடம் இருமுறை கட்டணம் வசூலிக்கப்பட்டது", "tamil"),
            (
                "russian",
                "С клиента дважды сняли деньги и он хочет возврат",
                "cyrillic",
            ),
            ("thai", "ลูกค้าถูกเรียกเก็บเงินสองครั้งและต้องการเงินคืน", "thai"),
            (
                "greek",
                "Ο πελάτης χρεώθηκε δύο φορές και θέλει επιστροφή χρημάτων",
                "greek",
            ),
            ("hebrew", "הלקוח חויב פעמיים ורוצה החזר כספי", "hebrew"),
            ("empty", "", "unknown"),
            ("digits only", "12345 6789", "unknown"),
        ];
        for (label, text, want) in cases {
            assert_eq!(detect_script(text), *want, "script/{label}");
        }
    }

    #[test]
    fn english_vs_not_matches_reference_suite() {
        let cases: &[(&str, &str, bool)] = &[
            (
                "plain english",
                "Please refund the duplicate charge on invoice 4411 today.",
                true,
            ),
            ("armenian", "Հայերեն", false),
            ("english short", "refund me", true),
            ("hindi", "ग्राहक से दो बार शुल्क लिया गया", false),
            ("japanese", "お客様は二重に請求されました", false),
            ("russian", "С клиента дважды сняли деньги", false),
            (
                "french long",
                "Le client a été facturé deux fois et il demande un remboursement pour la facture qui a été payée le mois dernier avec la carte de crédit",
                false,
            ),
            (
                "german long",
                "Der Kunde wurde zweimal belastet und möchte eine Rückerstattung für die Rechnung die nicht korrekt ist und auch nicht bezahlt wurde",
                false,
            ),
            // #35: an unidentified language must never be assumed English.
            (
                "romanian",
                "Gătește-mi o rețetă de sarmale de post pentru mâine.",
                false,
            ),
            (
                "romanian invoice",
                "Am fost taxat de două ori pentru factura din luna martie și vreau banii",
                false,
            ),
            (
                "polish",
                "Klient został obciążony dwukrotnie i chce zwrot pieniędzy za fakturę",
                false,
            ),
            (
                "czech",
                "Zákazníkovi byla částka účtována dvakrát a žádá o vrácení peněz",
                false,
            ),
            (
                "turkish",
                "Müşteriden iki kez ücret alındı ve para iadesi istiyor lütfen yardım",
                false,
            ),
            (
                "vietnamese",
                "Khách hàng đã bị thu phí hai lần và muốn được hoàn tiền ngay",
                false,
            ),
            // English with the odd loanword must not tip over.
            (
                "english with loanwords",
                "We visited a cafe in Zurich and the naive assumption about the invoice was wrong, so please refund the duplicate charge",
                true,
            ),
        ];
        for (label, text, want) in cases {
            assert_eq!(is_english(&json!(text)), *want, "is_english/{label}");
        }
    }

    #[test]
    fn undecided_is_flagged_not_dressed_up() {
        let turkish = json!("Müşteriden iki kez ücret alındı ve para iadesi istiyor");
        let d = analyse(&turkish);
        assert!(d.language_undecided);
        assert_eq!(d.language, None);
        let english = json!("Please refund the duplicate charge on the invoice");
        assert!(!analyse(&english).language_undecided);
        let ro = json!("Gătește-mi o rețetă de sarmale");
        assert!(analyse(&ro).diacritic_rate > 0.02);
        assert_eq!(
            analyse(&json!("Please refund the duplicate charge today")).diacritic_rate,
            0.0
        );
    }

    #[test]
    fn zero_tie_invents_no_language() {
        assert_eq!(guess_latin_language("Cât e ora acum la Tokyo"), None);
    }

    #[test]
    fn known_gap_kept_visible_on_purpose() {
        // The reference keeps this gap visible: Romanian short enough to
        // carry no diacritics and an English function word still reads as
        // English. A real LID model is the fix, not more stopwords.
        assert!(is_english(&json!("Care este ora in Tokyo?")));
    }

    #[test]
    fn latin_language_guess_matches_reference_suite() {
        let cases: &[(&str, &str, Option<&str>)] = &[
            (
                "english",
                "The customer was charged twice and wants a refund for this invoice",
                Some("en"),
            ),
            (
                "french",
                "Le client a ete facture deux fois et il demande un remboursement pour la facture",
                Some("fr"),
            ),
            (
                "german",
                "Der Kunde wurde zweimal belastet und moechte eine Rueckerstattung fuer die Rechnung",
                Some("de"),
            ),
            (
                "spanish",
                "El cliente fue cobrado dos veces y quiere que le devuelvan el dinero por la factura",
                Some("es"),
            ),
            ("too short", "refund", None),
        ];
        for (label, text, want) in cases {
            assert_eq!(
                guess_latin_language(text).as_deref(),
                *want,
                "latin_lang/{label}"
            );
        }
        assert_eq!(
            guess_latin_language(
                "Please refund the duplicate charge on invoice 4411 today because we have \
                 been waiting for three days and nobody has replied to us"
            )
            .as_deref(),
            Some("en"),
            "long english stays en"
        );
    }

    #[test]
    fn state_flattening_ignores_keys() {
        assert!(state_text(&json!({"body": "charged twice", "n": 3})).contains("charged twice"));
        assert!(state_text(&json!({"a": {"b": ["deep"]}})).contains("deep"));
        assert!(state_text(&json!(["x", {"y": "z"}])).contains("x"));
        assert_eq!(state_text(&json!(null)), "");
        // English keys around Hindi content stay non-English.
        let d = analyse(&json!({"subject": "नमस्ते", "body": "ग्राहक से दो बार शुल्क लिया गया"}));
        assert!(!d.is_english);
    }

    #[test]
    fn script_profiles_are_ratios() {
        let p = script_profile("Հայերեն");
        assert_eq!(p.get("armenian").copied(), Some(1.0));
        // non_latin_fraction: 6 armenian + 3 latin of 9 letters = 0.7
        let d = analyse(&json!("Հայերեն abc"));
        assert!(
            (d.non_latin_fraction - 0.7).abs() < 1e-9,
            "got {}",
            d.non_latin_fraction
        );
    }
}
