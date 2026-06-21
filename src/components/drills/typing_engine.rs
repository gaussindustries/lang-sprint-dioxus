//! Shared, Dioxus-free typing/grading core for the drills. Pure so it's
//! testable. The WPM test can adopt the timing/cursor side of this in a later
//! pass; for now it carries what the reading drill needs that the speed test
//! doesn't — turning a dictionary gloss into accepted answers and grading a
//! typed response against them.

/// Levenshtein edit distance over characters (script-agnostic).
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Normalize for fair comparison: trim, lowercase, collapse inner whitespace.
pub fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Drop parenthetical asides: "long (in length)" -> "long".
fn strip_parens(s: &str) -> String {
    let mut depth = 0i32;
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Split a dictionary gloss into individual accepted answers, plus a
/// parenthetical-stripped variant of each. `"way / road / path"` ->
/// `["way","road","path"]`; `"long (in length)"` -> `["long (in length)","long"]`.
pub fn accepted_answers(gloss: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in gloss.split(|c| c == '/' || c == ',' || c == ';') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if !out.iter().any(|x| x == p) {
            out.push(p.to_string());
        }
        let bare = strip_parens(p);
        if !bare.is_empty() && bare != p && !out.iter().any(|x| *x == bare) {
            out.push(bare);
        }
    }
    out
}

/// Grade a typed answer against accepted forms. Exact (normalized) match -> 1.0;
/// otherwise scaled by the best edit distance relative to length (a single typo
/// on a longer word still scores high). Range `0.0..=1.0`.
pub fn grade_answer(typed: &str, accepted: &[String]) -> f32 {
    let t = normalize(typed);
    if t.is_empty() {
        return 0.0;
    }
    let mut best = 0.0f32;
    for a in accepted {
        let an = normalize(a);
        if an.is_empty() {
            continue;
        }
        if t == an {
            return 1.0;
        }
        let dist = levenshtein(&t, &an);
        let len = an.chars().count().max(1);
        let score = 1.0 - dist as f32 / len as f32;
        if score > best {
            best = score;
        }
    }
    best.clamp(0.0, 1.0)
}

/// Tokenize an L2 sentence into word surfaces, stripping surrounding punctuation
/// but keeping letters/marks. Works for Georgian, Cyrillic, and Latin.
pub fn tokenize(sentence: &str) -> Vec<String> {
    sentence
        .split(|c: char| c.is_whitespace())
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance_basics() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("same", "same"), 0);
    }

    #[test]
    fn gloss_splits_into_answers() {
        assert_eq!(
            accepted_answers("way / road / path"),
            vec!["way", "road", "path"]
        );
        let a = accepted_answers("he/she/it, that");
        assert!(a.contains(&"he".to_string()) && a.contains(&"that".to_string()));
        let b = accepted_answers("long (in length)");
        assert!(
            b.contains(&"long".to_string()),
            "paren-stripped variant present"
        );
    }

    #[test]
    fn grading_exact_typo_and_wrong() {
        let acc = accepted_answers("house / home");
        assert_eq!(grade_answer("home", &acc), 1.0);
        assert_eq!(grade_answer("HOUSE", &acc), 1.0); // case-insensitive
        assert!(grade_answer("hous", &acc) > 0.7); // one missing char on 5
        assert!(grade_answer("zzzzz", &acc) < 0.3); // nonsense
        assert_eq!(grade_answer("", &acc), 0.0);
    }

    #[test]
    fn tokenize_strips_punctuation() {
        assert_eq!(
            tokenize("კაცი ქუჩაში დადიოდა."),
            vec!["კაცი", "ქუჩაში", "დადიოდა"]
        );
        assert_eq!(tokenize("  hello,  world!  "), vec!["hello", "world"]);
    }
}
