//! The on-device model that learns as you type. It starts empty.
//!
//! - Letter model: counts of each letter after the previous 0–3 letters of the
//!   word (accents folded, ñ kept apart), blended with recursive Dirichlet
//!   smoothing (α = 1.5) from the uniform distribution up.
//! - Word model: word counts and word-pair counts, for suggestions.

use std::collections::HashMap;

/// Letters the letter model knows about.
pub const ALPHABET: &str = "abcdefghijklmnñopqrstuvwxyz";
const ALPHA_LEN: usize = 27;
const SMOOTHING: f64 = 1.5;
const MAX_WORDS: usize = 5000;
const PRUNE: usize = 500;

/// Folds case and accents: "Canción" → "cancion". Keeps ñ.
pub fn fold(word: &str) -> String {
    word.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            c => c,
        })
        .collect()
}

pub(crate) fn alpha_index(c: char) -> Option<usize> {
    ALPHABET.chars().position(|a| a == c)
}

/// Letter probabilities for the next letter.
#[derive(Clone, Debug)]
pub struct Prediction {
    p: [f64; ALPHA_LEN],
    /// How many times the one-letter context has been seen; below 8 the
    /// prediction is not trusted.
    pub support: u32,
}

impl Prediction {
    /// Probability of a letter (any case, accents folded). Unknown letters give `None`.
    pub fn prob(&self, letter: char) -> Option<f64> {
        let f = fold(&letter.to_string()).chars().next()?;
        alpha_index(f).map(|i| self.p[i])
    }

    /// Letters from most to least likely.
    pub fn ranked(&self) -> Vec<(char, f64)> {
        let mut v: Vec<(char, f64)> = ALPHABET.chars().zip(self.p).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v
    }
}

/// Everything the keyboard has learnt. Plain counts; nothing leaves the device
/// unless the host saves it somewhere.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    contexts: HashMap<String, HashMap<char, u32>>,
    words: HashMap<String, u32>,
    pairs: HashMap<String, HashMap<String, u32>>,
    /// Corrections the user undid. Each one makes correction more cautious.
    pub rejections: u32,
    /// Letters learnt so far. Correction waits until there are 60.
    pub letters: u32,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct words learnt.
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Learns a finished word (as it ended up after any backspacing) and the word before it.
    pub fn learn_word(&mut self, word: &str, previous: Option<&str>) {
        let word = word.to_lowercase();
        if word.is_empty() || word.chars().any(char::is_whitespace) {
            return;
        }
        let previous = previous.filter(|p| !p.chars().any(char::is_whitespace));
        let s: Vec<char> = std::iter::once('_').chain(fold(&word).chars()).collect();
        for i in 1..s.len() {
            let x = s[i];
            if alpha_index(x).is_none() {
                continue;
            }
            for k in 0..=i.min(3) {
                let ctx: String = s[i - k..i].iter().collect();
                *self.contexts.entry(ctx).or_default().entry(x).or_default() += 1;
            }
            self.letters += 1;
        }
        *self.words.entry(word.clone()).or_default() += 1;
        if let Some(prev) = previous {
            let prev = prev.to_lowercase();
            if !prev.is_empty() {
                *self.pairs.entry(prev).or_default().entry(word).or_default() += 1;
            }
        }
        if self.words.len() > MAX_WORDS {
            let mut by_use: Vec<(String, u32)> =
                self.words.iter().map(|(w, n)| (w.clone(), *n)).collect();
            by_use.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            for (w, _) in by_use.into_iter().take(PRUNE) {
                self.words.remove(&w);
                self.pairs.remove(&w);
            }
        }
    }

    /// Probabilities for the next letter of `word_so_far` (the letters typed of the
    /// current word; empty at the start of a word).
    pub fn predict(&self, word_so_far: &str) -> Prediction {
        let h: Vec<char> = std::iter::once('_')
            .chain(fold(word_so_far).chars())
            .collect();
        let mut p = [1.0 / ALPHA_LEN as f64; ALPHA_LEN];
        let mut support = 0;
        for k in 0..=h.len().min(3) {
            let ctx: String = h[h.len() - k..].iter().collect();
            let Some(counts) = self.contexts.get(&ctx) else {
                continue;
            };
            let n: u32 = counts.values().sum();
            for (i, l) in ALPHABET.chars().enumerate() {
                let c = *counts.get(&l).unwrap_or(&0) as f64;
                p[i] = (c + SMOOTHING * p[i]) / (n as f64 + SMOOTHING);
            }
            if k == 1 {
                support = n;
            }
        }
        Prediction { p, support }
    }

    /// Up to `n` words: completions of `prefix` (scored by uses + 4 × uses after
    /// `previous`), or, with an empty prefix, words that tend to follow `previous`.
    pub fn suggest(&self, prefix: &str, previous: Option<&str>, n: usize) -> Vec<String> {
        let empty = HashMap::new();
        let after = previous
            .and_then(|p| self.pairs.get(&p.to_lowercase()))
            .unwrap_or(&empty);
        let score = |w: &str| {
            self.words.get(w).copied().unwrap_or(0) + 4 * after.get(w).copied().unwrap_or(0)
        };
        let mut cands: Vec<&String> = if prefix.is_empty() {
            after.keys().collect()
        } else {
            let fp = fold(prefix);
            let plen = prefix.chars().count();
            self.words
                .keys()
                .filter(|w| w.chars().count() > plen && fold(w).starts_with(&fp))
                .collect()
        };
        cands.sort_by(|a, b| score(b).cmp(&score(a)).then_with(|| a.cmp(b)));
        cands.into_iter().take(n).cloned().collect()
    }

    /// Forgets everything.
    pub fn clear(&mut self) {
        *self = Model::default();
    }

    /// Saves the model as plain UTF-8 text, one count per line.
    pub fn to_text(&self) -> String {
        let mut out = format!(
            "wheel-keyboard-model 1\nrejections {}\nletters {}\n",
            self.rejections, self.letters
        );
        let mut ctx: Vec<_> = self.contexts.iter().collect();
        ctx.sort_by(|a, b| a.0.cmp(b.0));
        for (c, counts) in ctx {
            let mut counts: Vec<_> = counts.iter().collect();
            counts.sort();
            for (l, n) in counts {
                out += &format!("c |{c} {l} {n}\n");
            }
        }
        let mut words: Vec<_> = self.words.iter().collect();
        words.sort();
        for (w, n) in words {
            out += &format!("w {w} {n}\n");
        }
        let mut pairs: Vec<_> = self.pairs.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        for (a, next) in pairs {
            let mut next: Vec<_> = next.iter().collect();
            next.sort();
            for (b, n) in next {
                out += &format!("b {a} {b} {n}\n");
            }
        }
        out
    }

    /// Loads a model saved with [`Model::to_text`]. Returns `None` if the text is not one.
    /// Unknown lines are skipped, so newer files still load.
    pub fn from_text(text: &str) -> Option<Model> {
        let mut lines = text.lines();
        if lines.next()?.trim() != "wheel-keyboard-model 1" {
            return None;
        }
        let mut m = Model::default();
        for line in lines {
            let f: Vec<&str> = line.split(' ').collect();
            match f.as_slice() {
                ["rejections", n] => m.rejections = n.parse().ok()?,
                ["letters", n] => m.letters = n.parse().ok()?,
                ["c", ctx, l, n] => {
                    let ctx = ctx.strip_prefix('|')?;
                    let mut lc = l.chars();
                    let l = lc.next()?;
                    if lc.next().is_some() || ctx.chars().count() > 3 {
                        return None;
                    }
                    m.contexts
                        .entry(ctx.to_string())
                        .or_default()
                        .insert(l, n.parse().ok()?);
                }
                ["w", w, n] => {
                    m.words.insert(w.to_string(), n.parse().ok()?);
                }
                ["b", a, b, n] => {
                    m.pairs
                        .entry(a.to_string())
                        .or_default()
                        .insert(b.to_string(), n.parse().ok()?);
                }
                _ => {}
            }
        }
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_uniform() {
        let p = Model::new().predict("");
        assert!((p.prob('a').unwrap() - 1.0 / 27.0).abs() < 1e-9);
        assert_eq!(p.support, 0);
    }

    #[test]
    fn learns_letters_after_context() {
        let mut m = Model::new();
        for _ in 0..16 {
            m.learn_word("queso", None);
        }
        let p = m.predict("qu");
        assert!(p.prob('e').unwrap() > 0.9);
        assert_eq!(p.support, 16);
        assert_eq!(m.letters, 80);
    }

    #[test]
    fn folds_accents_but_keeps_enye() {
        assert_eq!(fold("Canción"), "cancion");
        assert_eq!(fold("Año"), "año");
    }

    #[test]
    fn suggests_completions_and_next_words() {
        let mut m = Model::new();
        m.learn_word("hola", None);
        m.learn_word("mundo", Some("hola"));
        assert_eq!(m.suggest("ho", None, 3), vec!["hola"]);
        assert_eq!(m.suggest("", Some("hola"), 3), vec!["mundo"]);
    }

    #[test]
    fn round_trips_as_text() {
        let mut m = Model::new();
        m.learn_word("año", None);
        m.learn_word("nuevo", Some("año"));
        m.rejections = 2;
        let back = Model::from_text(&m.to_text()).unwrap();
        assert_eq!(back, m);
        assert!(Model::from_text("nope").is_none());
    }

    #[test]
    fn keeps_at_most_5000_words() {
        let mut m = Model::new();
        for i in 0..5001 {
            m.learn_word(&format!("w{i}"), None);
        }
        assert_eq!(m.word_count(), 4501);
    }
}
