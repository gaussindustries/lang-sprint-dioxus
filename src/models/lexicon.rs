// src/models/lexicon.rs
//
// The lexicon is the single source of truth for words. The "1000 most common"
// list is no longer a separate concept — every entry carries a frequency `rank`
// as metadata, and any subset (the top-N drill list, a rank window, a POS
// filter, a dictionary search) is derived in post via the `Lexicon` query API.
//
// The entry field names match the old `FrequencyWord` exactly, so the existing
// `1000.json` files deserialize without any migration. New fields are all
// `#[serde(default)]`, so old JSON stays valid; they're forward hooks for the
// dictionary / audio / OCR-ingestion work.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One lexical entry (a head word plus its metadata).
///
/// `en` is kept singular for now to stay compatible with the current data and
/// call sites; it becomes `glosses: Vec<String>` in the dictionary phase.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LexEntry {
    /// Frequency rank within the language (1 = most common).
    /// Subsets like "top 1000" are derived by sorting/filtering on this.
    pub rank: u32,

    /// L1 gloss / English meaning.
    pub en: String,

    /// L2 head form (dictionary form).
    pub word: String,

    /// Part of speech, if tagged (e.g. "noun", "verb", "adjective").
    #[serde(default)]
    pub pos: Option<String>,

    /// Optional example sentence in L2.
    #[serde(default)]
    pub example: Option<String>,

    // ── forward hooks (all optional; absent in current JSON) ──────────────
    /// Pronunciation asset path, for the listening drill.
    #[serde(default)]
    pub audio: Option<String>,

    /// Freeform tags: topic, source document, level, etc.
    #[serde(default)]
    pub tags: Vec<String>,

    /// True for entries the user added (vs the seeded frequency list).
    #[serde(default)]
    pub user_added: bool,

    /// Full nominal declension. For Georgian this is the 7-case paradigm
    /// (singular / plural / attributive "with noun"). Optional + `serde(default)`,
    /// so entries without it — i.e. all current data — stay valid.
    #[serde(default)]
    pub declension: Option<Declension>,
}

impl LexEntry {
    /// L2 head form with any "a / b" alternates stripped to the first form.
    /// Mirrors the `clean_word` logic the drills use when generating prompts.
    pub fn head(&self) -> &str {
        self.word.split('/').next().unwrap_or(&self.word).trim()
    }
}

/// The seven Georgian grammatical cases, in canonical paradigm order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeoCase {
    Nominative,
    Ergative,
    Dative,
    Genitive,
    Instrumental,
    Adverbial,
    Vocative,
}

impl GeoCase {
    /// All cases in display order.
    pub const ALL: [GeoCase; 7] = [
        GeoCase::Nominative,
        GeoCase::Ergative,
        GeoCase::Dative,
        GeoCase::Genitive,
        GeoCase::Instrumental,
        GeoCase::Adverbial,
        GeoCase::Vocative,
    ];

    /// English case name (proper case), for the table's first column.
    pub fn label(self) -> &'static str {
        match self {
            GeoCase::Nominative => "Nominative",
            GeoCase::Ergative => "Ergative",
            GeoCase::Dative => "Dative",
            GeoCase::Genitive => "Genitive",
            GeoCase::Instrumental => "Instrumental",
            GeoCase::Adverbial => "Adverbial",
            GeoCase::Vocative => "Vocative",
        }
    }

    /// One-line "what this case is for", shown as a hover tooltip.
    pub fn blurb(self) -> &'static str {
        match self {
            GeoCase::Nominative => {
                "The dictionary form. Marks the subject with present/future-series \
                 verbs and with intransitives."
            }
            GeoCase::Ergative => {
                "Marks the subject of a transitive verb in the aorist (past) series — \
                 Georgian's split ergativity."
            }
            GeoCase::Dative => {
                "The 'to / for' case: the indirect object, and the direct object in \
                 some tense series."
            }
            GeoCase::Genitive => "Possession or association — the 'of' case (whose? of what?).",
            GeoCase::Instrumental => {
                "'By means of' — the tool, material, or means by which something is done."
            }
            GeoCase::Adverbial => {
                "Transformation or manner — 'as', 'into', 'in the form of'; also the \
                 translative."
            }
            GeoCase::Vocative => {
                "Direct address — calling out to someone or something ('O friend!')."
            }
        }
    }
}

/// The three column forms a Georgian word takes per case.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CaseForms {
    #[serde(default)]
    pub singular: String,
    #[serde(default)]
    pub plural: String,
    /// Attributive form, used when the word directly modifies a following noun.
    #[serde(default, rename = "with_noun")]
    pub with_noun: String,
}

impl CaseForms {
    pub fn is_empty(&self) -> bool {
        self.singular.trim().is_empty()
            && self.plural.trim().is_empty()
            && self.with_noun.trim().is_empty()
    }
}

/// A full case × number paradigm. The field names are the JSON keys the lexicon
/// stores, so the data file reads exactly like the matrix it represents.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Declension {
    #[serde(default)]
    pub nominative: CaseForms,
    #[serde(default)]
    pub ergative: CaseForms,
    #[serde(default)]
    pub dative: CaseForms,
    #[serde(default)]
    pub genitive: CaseForms,
    #[serde(default)]
    pub instrumental: CaseForms,
    #[serde(default)]
    pub adverbial: CaseForms,
    #[serde(default)]
    pub vocative: CaseForms,
}

impl Declension {
    /// Borrow the forms for a given case.
    pub fn forms(&self, case: GeoCase) -> &CaseForms {
        match case {
            GeoCase::Nominative => &self.nominative,
            GeoCase::Ergative => &self.ergative,
            GeoCase::Dative => &self.dative,
            GeoCase::Genitive => &self.genitive,
            GeoCase::Instrumental => &self.instrumental,
            GeoCase::Adverbial => &self.adverbial,
            GeoCase::Vocative => &self.vocative,
        }
    }

    /// True if at least one cell is non-empty (gate for showing the table).
    pub fn any_filled(&self) -> bool {
        GeoCase::ALL.iter().any(|&c| !self.forms(c).is_empty())
    }
}

/// A collection of [`LexEntry`] with derive-in-post query helpers.
///
/// All queries return owned `Vec<LexEntry>` (cheap clones; the data is small)
/// so callers can hand them straight to drills/props without lifetime fuss.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lexicon {
    entries: Vec<LexEntry>,
}

impl Lexicon {
    pub fn from_entries(entries: Vec<LexEntry>) -> Self {
        Self { entries }
    }

    /// Parse a JSON array of entries (the format of `1000.json`).
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            entries: serde_json::from_str(json)?,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Borrow every entry, unsorted (insertion order).
    pub fn all(&self) -> &[LexEntry] {
        &self.entries
    }

    /// Every entry, sorted by ascending rank (most common first).
    pub fn by_rank(&self) -> Vec<LexEntry> {
        let mut v = self.entries.clone();
        v.sort_by_key(|e| e.rank);
        v
    }

    /// The `n` most common entries, rank-sorted. This is "the top 1000".
    pub fn top(&self, n: usize) -> Vec<LexEntry> {
        self.by_rank().into_iter().take(n).collect()
    }

    /// Entries whose rank falls in `[lo, hi]` inclusive, rank-sorted.
    /// Bounds are swapped if given out of order.
    pub fn within_rank(&self, lo: u32, hi: u32) -> Vec<LexEntry> {
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        let mut v: Vec<LexEntry> = self
            .entries
            .iter()
            .filter(|e| e.rank >= lo && e.rank <= hi)
            .cloned()
            .collect();
        v.sort_by_key(|e| e.rank);
        v
    }

    /// Entries whose POS is in `pos` (case-insensitive).
    /// An empty `pos` slice means "no filter" and returns everything.
    pub fn by_pos(&self, pos: &[String]) -> Vec<LexEntry> {
        if pos.is_empty() {
            return self.entries.clone();
        }
        let want: Vec<String> = pos.iter().map(|p| p.to_lowercase()).collect();
        self.entries
            .iter()
            .filter(|e| {
                e.pos
                    .as_ref()
                    .is_some_and(|p| want.contains(&p.to_lowercase()))
            })
            .cloned()
            .collect()
    }

    /// The distinct POS tags present, sorted. Drives the POS-filter UI.
    pub fn pos_tags(&self) -> Vec<String> {
        let mut set: BTreeSet<String> = BTreeSet::new();
        for e in &self.entries {
            if let Some(p) = &e.pos {
                set.insert(p.clone());
            }
        }
        set.into_iter().collect()
    }

    /// Highest rank present (i.e. the size of the frequency list), or 1.
    pub fn max_rank(&self) -> u32 {
        self.entries.iter().map(|e| e.rank).max().unwrap_or(1)
    }

    /// Case-insensitive substring search over L2 head form and L1 gloss.
    /// This is the dictionary-lookup entry point. Exact head-form matches sort
    /// first, then by rank.
    pub fn search(&self, query: &str) -> Vec<LexEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let mut hits: Vec<LexEntry> = self
            .entries
            .iter()
            .filter(|e| e.word.to_lowercase().contains(&q) || e.en.to_lowercase().contains(&q))
            .cloned()
            .collect();
        hits.sort_by(|a, b| {
            let a_exact = (a.word.to_lowercase() == q) as u8;
            let b_exact = (b.word.to_lowercase() == q) as u8;
            b_exact.cmp(&a_exact).then(a.rank.cmp(&b.rank))
        });
        hits
    }

    /// First entry whose L2 head form matches exactly (case-insensitive).
    pub fn get(&self, lemma: &str) -> Option<&LexEntry> {
        let l = lemma.to_lowercase();
        self.entries.iter().find(|e| e.word.to_lowercase() == l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(rank: u32, word: &str, en: &str, pos: Option<&str>) -> LexEntry {
        LexEntry {
            rank,
            word: word.into(),
            en: en.into(),
            pos: pos.map(str::to_string),
            ..Default::default()
        }
    }

    fn fixture() -> Lexicon {
        Lexicon::from_entries(vec![
            e(3, "ის", "he/she/it", Some("pronoun")),
            e(1, "გამარჯობა", "hello", Some("interjection")),
            e(2, "ყოფნა", "to be", Some("verb")),
        ])
    }

    #[test]
    fn top_is_rank_sorted() {
        let ranks: Vec<u32> = fixture().top(2).iter().map(|x| x.rank).collect();
        assert_eq!(ranks, vec![1, 2]);
    }

    #[test]
    fn within_rank_is_inclusive_and_order_agnostic() {
        assert_eq!(fixture().within_rank(2, 3).len(), 2);
        assert_eq!(fixture().within_rank(3, 2).len(), 2);
    }

    #[test]
    fn search_prefers_exact_head_form() {
        assert_eq!(fixture().search("ის").first().unwrap().word, "ის");
    }

    #[test]
    fn pos_tags_are_sorted_and_distinct() {
        assert_eq!(
            fixture().pos_tags(),
            vec!["interjection", "pronoun", "verb"]
        );
    }

    #[test]
    fn from_json_parses_minimal_entries() {
        let json = r#"[{"rank":1,"en":"hello","word":"გამარჯობა","pos":"interjection"}]"#;
        let lex = Lexicon::from_json(json).unwrap();
        assert_eq!(lex.len(), 1);
        assert_eq!(lex.all()[0].user_added, false);
    }

    #[test]
    fn declension_parses_and_absent_cells_default() {
        let json = r#"[
            {"rank":1,"en":"man","word":"კაცი","pos":"noun",
             "declension":{"nominative":{"singular":"კაცი","plural":"კაცები"}}}
        ]"#;
        let lex = Lexicon::from_json(json).unwrap();
        let d = lex.all()[0].declension.clone().unwrap();
        assert_eq!(d.nominative.singular, "კაცი");
        assert_eq!(d.nominative.plural, "კაცები");
        assert!(d.nominative.with_noun.is_empty());
        assert!(d.dative.is_empty());
        assert!(d.any_filled());
    }
}
