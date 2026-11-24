//! Brazilian Portuguese spelling → phonemes and sung syllables.
//!
//! This is a spelling-based São Paulo/Rio front end, not a dictionary. It keeps the spelling
//! visible on each sung note while assigning the note's nucleus, onset, and coda in IPA.

use super::english::{Phrase, is_phrase_punct, split_keeping};
use super::mandarin::MandarinLexicon;
use super::syllables::{FrontEnd, Lang, lyrics_in};
use super::{Intonation, Segment, Syllable};

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn is_vowel(c: char) -> bool {
    "aeiouáàéíóúâêôãõ".contains(c)
}

fn has_written_stress(c: char) -> bool {
    "áéíóúâêôãõ".contains(c)
}

fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic() || "áàâãéêíóõúçÁÀÂÃÉÊÍÓÕÚÇ".contains(c)
}

fn front_vowel(c: char) -> bool {
    matches!(c, 'e' | 'é' | 'ê' | 'i' | 'í')
}

fn vowel_ipa(c: char, final_unstressed: bool) -> &'static str {
    if final_unstressed {
        return match c {
            'a' => "ə",
            'e' => "i",
            _ => "",
        };
    }
    match c {
        'a' | 'á' | 'à' | 'â' => "a",
        'ã' => "ə",
        'e' | 'ê' => "e",
        'é' => "ɛ",
        'i' | 'í' => "i",
        'o' | 'ô' => "o",
        'ó' => "ɔ",
        'õ' => "o",
        'u' | 'ú' => "u",
        _ if final_unstressed => "i",
        _ => "ə",
    }
}

fn is_falling_pair(a: char, b: char) -> bool {
    matches!(a, 'a' | 'e' | 'o' | 'á' | 'é' | 'ó' | 'ã' | 'õ') && matches!(b, 'i' | 'u')
}

fn silent_u(w: &[char], i: usize) -> bool {
    w.get(i) == Some(&'u')
        && i > 0
        && matches!(w[i - 1], 'q' | 'g')
        && w.get(i + 1).is_some_and(|c| matches!(c, 'e' | 'i'))
}

#[derive(Clone)]
struct Phone {
    ipa: &'static str,
    letter: usize,
    vowel: bool,
    nasalized: bool,
}

fn orthographic_syllable_starts(w: &[char], nuclei: &[usize], nasal_m: &[bool]) -> Vec<usize> {
    let mut starts = vec![0];
    for pair in nuclei.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        let spelled_glide_onset = (left + 1..right).find(|&i| {
            matches!(w[i], 'q' | 'g')
                && w.get(i + 1) == Some(&'u')
                && (silent_u(w, i + 1)
                    || w.get(i + 2)
                        .is_some_and(|c| matches!(c, 'a' | 'o' | 'á' | 'ó' | 'ã' | 'õ')))
        });
        if let Some(i) = spelled_glide_onset {
            starts.push(i);
            continue;
        }
        let mut consonants = Vec::new();
        let mut i = left + 1;
        while i < right {
            if nasal_m.get(i) == Some(&true) || silent_u(w, i) {
                i += 1;
                continue;
            }
            if is_vowel(w[i]) {
                i += 1;
                continue;
            }
            consonants.push(i);
            i += 1;
        }
        if consonants.is_empty() {
            starts.push(right);
            continue;
        }

        let last = *consonants.last().unwrap();
        let digraph_onset = consonants.len() >= 2
            && matches!((w[last - 1], w[last]), ('l', 'h') | ('n', 'h') | ('c', 'h'));
        let double_split =
            consonants.len() >= 2 && matches!((w[last - 1], w[last]), ('r', 'r') | ('s', 's'));
        let cluster_onset = consonants.len() >= 2
            && matches!(
                (w[last - 1], w[last]),
                ('b', 'r' | 'l')
                    | ('c', 'r' | 'l')
                    | ('d', 'r')
                    | ('f', 'r' | 'l')
                    | ('g', 'r' | 'l')
                    | ('p', 'r' | 'l')
                    | ('t', 'r')
                    | ('v', 'r')
            );
        starts.push(if double_split {
            last
        } else if digraph_onset || cluster_onset {
            last - 1
        } else {
            last
        });
    }
    starts
}

fn nasal_m_flags(w: &[char]) -> (Vec<bool>, Vec<bool>, Vec<bool>) {
    let mut nasal_m = vec![false; w.len()];
    let mut nasal_vowel = vec![false; w.len()];
    let mut added_glide = vec![false; w.len()];
    for i in 0..w.len() {
        if !matches!(w[i], 'a' | 'e' | 'i' | 'o' | 'u')
            || w.get(i + 1) != Some(&'m')
            || w.get(i + 2).is_some_and(|c| is_vowel(*c))
        {
            continue;
        }
        nasal_m[i + 1] = true;
        nasal_vowel[i] = true;
        if i + 2 == w.len() {
            match w[i] {
                'a' => added_glide[i] = true,
                'e' => added_glide[i] = true,
                _ => {}
            }
        }
    }
    (nasal_m, nasal_vowel, added_glide)
}

fn word_phones(typed: &str) -> (Vec<char>, Vec<char>, Vec<Phone>, Vec<usize>, Option<usize>) {
    let typed_letters: Vec<char> = typed.chars().filter(|c| is_letter(*c)).collect();
    let w: Vec<char> = typed_letters.iter().copied().map(lower).collect();
    let (nasal_m, mut nasal, added_glide) = nasal_m_flags(&w);
    for (i, c) in w.iter().enumerate() {
        if matches!(c, 'ã' | 'õ') {
            nasal[i] = true;
        }
    }

    let mut glides = vec![None; w.len()];
    for i in 0..w.len() {
        if silent_u(&w, i) {
            continue;
        }
        let c = w[i];
        if matches!(c, 'i' | 'u') && !has_written_stress(c) {
            if i > 0 && is_falling_pair(w[i - 1], c) {
                glides[i] = Some(if c == 'i' { "j" } else { "w" });
            }
        }
        if c == 'u'
            && i > 0
            && matches!(w[i - 1], 'q' | 'g')
            && w.get(i + 1)
                .is_some_and(|n| matches!(n, 'a' | 'o' | 'á' | 'ó' | 'ã' | 'õ'))
        {
            glides[i] = Some("w");
        }
        if matches!(c, 'ã' | 'õ') {
            if let Some(next) = w.get(i + 1) {
                if matches!((c, *next), ('ã', 'o') | ('õ', 'o')) {
                    glides[i + 1] = Some("w");
                } else if matches!((c, *next), ('ã', 'e') | ('õ', 'e')) {
                    glides[i + 1] = Some("j");
                }
            }
        }
    }

    let nuclei: Vec<usize> = (0..w.len())
        .filter(|&i| is_vowel(w[i]) && !silent_u(&w, i) && glides[i].is_none())
        .collect();
    if nuclei.is_empty() {
        return (w, typed_letters, Vec::new(), Vec::new(), None);
    }

    let starts = orthographic_syllable_starts(&w, &nuclei, &nasal_m);
    let last_nucleus = *nuclei.last().unwrap();
    let final_unstressed_e = w.last() == Some(&'e') && !has_written_stress(w[last_nucleus]);
    let final_unstressed_o = w.last() == Some(&'o') && !has_written_stress(w[last_nucleus]);
    let mut stress = nuclei.iter().position(|&i| has_written_stress(w[i]));
    if stress.is_none() {
        let lower_word: String = w.iter().collect();
        let singular = lower_word.strip_suffix('s').unwrap_or(&lower_word);
        let final_stress = ["r", "l", "z", "x", "i", "u", "im", "um"]
            .iter()
            .any(|ending| singular.ends_with(ending));
        stress = Some(if final_stress {
            nuclei.len() - 1
        } else {
            nuclei.len().saturating_sub(2)
        });
    }

    let mut phones = Vec::with_capacity(w.len() + 1);
    let mut i = 0;
    while i < w.len() {
        let c = w[i];
        let next = w.get(i + 1).copied();
        if silent_u(&w, i) || nasal_m[i] {
            i += 1;
            continue;
        }
        if let Some(glide) = glides[i] {
            phones.push(Phone {
                ipa: glide,
                letter: i,
                vowel: false,
                nasalized: false,
            });
            i += 1;
            continue;
        }
        if is_vowel(c) {
            let is_final_e = c == 'e' && i + 1 == w.len() && final_unstressed_e;
            let is_final_o = c == 'o' && i + 1 == w.len() && final_unstressed_o;
            let ipa = if is_final_o {
                "u"
            } else if nasal[i] && c == 'a' {
                "ə"
            } else {
                vowel_ipa(c, is_final_e || (i + 1 == w.len() && c == 'a'))
            };
            phones.push(Phone {
                ipa,
                letter: i,
                vowel: true,
                nasalized: nasal[i],
            });
            i += 1;
            continue;
        }
        let followed_by_i = matches!(next, Some('i' | 'í'));
        let final_e = next == Some('e') && i + 2 == w.len() && !has_written_stress('e');
        let at = i;
        let (ipa, advance) = match (c, next) {
            ('l', Some('h')) => ("ʎ", 2),
            ('n', Some('h')) => ("ɲ", 2),
            ('c', Some('h')) => ("ʃ", 2),
            ('r', Some('r')) => ("h", 2),
            ('s', Some('s')) => ("s", 2),
            ('q', Some('u')) if w.get(i + 2).is_some_and(|x| matches!(x, 'e' | 'i')) => ("k", 2),
            ('g', Some('u')) if w.get(i + 2).is_some_and(|x| matches!(x, 'e' | 'i')) => ("g", 2),
            ('t', _) if followed_by_i || final_e => ("tʃ", 1),
            ('d', _) if followed_by_i || final_e => ("dʒ", 1),
            ('ç', _) => ("s", 1),
            ('c', _) if next.is_some_and(front_vowel) => ("s", 1),
            ('c', _) => ("k", 1),
            ('g', _) if next.is_some_and(front_vowel) => ("ʒ", 1),
            ('g', _) => ("g", 1),
            ('j', _) => ("ʒ", 1),
            ('x', _) => ("ʃ", 1),
            ('r', _) if i == 0 => ("h", 1),
            ('r', _) if i > 0 && w[i - 1] == 'r' => ("h", 1),
            ('r', _) if i > 0 && is_vowel(w[i - 1]) && next.is_some_and(is_vowel) => ("ɾ", 1),
            ('r', _) if next.is_some_and(is_vowel) => ("ɾ", 1),
            ('r', _) => ("h", 1),
            ('s', _) if i > 0 && is_vowel(w[i - 1]) && next.is_some_and(is_vowel) => ("z", 1),
            ('s', _) => ("s", 1),
            ('l', _) if next.is_none_or(|n| !is_vowel(n)) => ("w", 1),
            ('l', _) => ("l", 1),
            ('m', _) => ("m", 1),
            ('n', _) => ("n", 1),
            ('p', _) => ("p", 1),
            ('b', _) => ("b", 1),
            ('f', _) => ("f", 1),
            ('v', _) => ("v", 1),
            ('t', _) => ("t", 1),
            ('d', _) => ("d", 1),
            ('q', _) => ("k", 1),
            ('h', _) => ("", 1),
            _ => ("", 1),
        };
        if !ipa.is_empty() {
            let letter = if matches!((c, next), ('r', Some('r')) | ('s', Some('s'))) {
                at + 1
            } else {
                at
            };
            phones.push(Phone {
                ipa,
                letter,
                vowel: false,
                nasalized: false,
            });
        }
        i += advance;
    }
    for i in 0..w.len() {
        if added_glide[i] && !phones.iter().any(|p| p.letter == i && !p.vowel) {
            phones.push(Phone {
                ipa: if w[i] == 'a' { "w" } else { "j" },
                letter: i,
                vowel: false,
                nasalized: false,
            });
        }
    }
    phones.sort_by_key(|p| p.letter);
    (w, typed_letters, phones, starts, stress)
}

fn syllabify_word(typed: &str) -> Vec<Syllable> {
    let (_, typed_letters, phones, starts, explicit_stress) = word_phones(typed);
    if starts.is_empty() || phones.is_empty() {
        return Vec::new();
    }
    let nuclei: Vec<usize> = phones
        .iter()
        .filter(|p| p.vowel)
        .map(|p| p.letter)
        .collect();
    let stress = explicit_stress.and_then(|i| nuclei.get(i).copied());
    let note_text = |a: usize, b: usize| {
        typed_letters
            .get(a..b)
            .unwrap_or_default()
            .iter()
            .collect::<String>()
    };
    let mut out: Vec<Syllable> = starts
        .iter()
        .enumerate()
        .map(|(i, &a)| Syllable {
            text: note_text(a, starts.get(i + 1).copied().unwrap_or(typed_letters.len())),
            ..Default::default()
        })
        .collect();
    let mut voiced = vec![false; out.len()];
    for phone in phones {
        let k = starts.iter().rposition(|&a| a <= phone.letter).unwrap_or(0);
        let seg = Segment {
            ipa: phone.ipa.to_string(),
            stress: Some(u8::from(phone.vowel && stress == Some(phone.letter))),
            nasalized: phone.nasalized.then_some(true),
            ..Default::default()
        };
        if phone.vowel {
            out[k].nucleus = seg;
            voiced[k] = true;
        } else if voiced[k] {
            out[k].coda.push(seg);
        } else {
            out[k].onset.push(seg);
        }
    }
    out
}

/// One word's phones, primarily exposed for focused pronunciation tests.
pub fn word(typed: &str) -> Vec<Segment> {
    let (_, _, phones, _, stress) = word_phones(typed);
    let vowel_indices: Vec<usize> = phones
        .iter()
        .filter(|p| p.vowel)
        .map(|p| p.letter)
        .collect();
    phones
        .into_iter()
        .map(|p| Segment {
            ipa: p.ipa.to_string(),
            stress: Some(u8::from(
                p.vowel && stress.is_some_and(|s| vowel_indices.get(s) == Some(&p.letter)),
            )),
            nasalized: p.nasalized.then_some(true),
            ..Default::default()
        })
        .collect()
}

/// The Portuguese lyrics front end. Hyphens split notes explicitly; other words are syllabified
/// from Portuguese spelling.
pub struct Portuguese;

impl FrontEnd for Portuguese {
    fn starts(&self, c: char) -> bool {
        is_letter(c)
    }

    fn continues(&self, c: char) -> bool {
        is_letter(c) || c == '-'
    }

    fn word(&self, tok: &str) -> Vec<Syllable> {
        tok.split('-').flat_map(syllabify_word).collect()
    }
}

/// Lyrics → sung syllables with Latin script read as Brazilian Portuguese.
pub fn lyrics_to_syllables(text: &str) -> Vec<Syllable> {
    lyrics_in(
        text,
        &|_: &str, _: bool| Vec::<Segment>::new(),
        &MandarinLexicon::new(),
        Lang::Pt,
    )
}

/// Brazilian Portuguese text → phrase inputs for Speak.
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let mut phrases = Vec::new();
    let mut words = Vec::new();
    let mut flush = |delim: &str, words: &mut Vec<String>| {
        let mut segments = Vec::new();
        let mut said = Vec::new();
        for w in words.drain(..) {
            let mut word_segments = word(&w);
            if let Some(last) = word_segments.last_mut() {
                last.word_end = Some(true);
            }
            segments.extend(word_segments);
            said.push(
                w.chars()
                    .filter(|c| is_letter(*c))
                    .map(lower)
                    .collect::<String>(),
            );
        }
        if segments.is_empty() {
            return;
        }
        let pause = if delim.contains(['.', '!', '?']) {
            0.35
        } else if !delim.is_empty() {
            0.18
        } else {
            0.3
        };
        phrases.push(Phrase {
            segments,
            intonation: Intonation::Statement,
            pause,
            words: said,
        });
    };
    for part in split_keeping(text, is_phrase_punct) {
        if !part.is_empty() && part.chars().all(is_phrase_punct) {
            flush(&part, &mut words);
        } else {
            words.extend(
                part.split(|c: char| !is_letter(c) && c != '-')
                    .filter(|w| w.chars().any(is_letter))
                    .map(str::to_string),
            );
        }
    }
    flush("", &mut words);
    phrases
}
