# Brazilian Portuguese

Speak and Sing read Brazilian Portuguese when you choose **PT**, next to the lyrics in Sing and
beside the text controls in Speak. It is one setting for both screens and the plugin. English
remains the default. Since Portuguese and English use Latin letters, the lyrics router can't tell
them apart ([language-front-ends.md](language-front-ends.md)). Portuguese accents take priority over
ambiguous Pinyin readings; Pinyin-only tone marks and numbered Pinyin remain available.

`crates/chimera/src/phonetics/portuguese.rs` maps pt-BR spelling by rule, without a dictionary.
Syllables keep their original spelling as the note text.

## Writing it

- Write Portuguese spelling, including accents and `ç`: `pão`, `mãe`, `você`, `coração`.
- In Sing, the word is syllabified for notes (`pa-re-de`, `fi-lho`, `can-tar`). Typed hyphens
  force a split; a `-` by itself holds the previous vowel as a melisma.
- `chimera syllables "texto em português" pt` prints the phonemes and one JSON syllable per note.

## What it does

| Spelling | Reading |
|---|---|
| t/d before `i`, and before unstressed final `e` | `tʃ` / `dʒ` (`tia`, `parede`) |
| `lh`, `nh`, `ch` | `ʎ`, `ɲ`, `ʃ` |
| `x`, `ç`, `qu/gu` before `e/i` | `ʃ`, `s`, `k/g` |
| `rr` and word-initial `r`; `r` between vowels; syllable-final `r` | `h`; `ɾ`; `h` |
| `s` between vowels; `ss`; final `s` | `z`; `s`; `s` |
| syllable-final `l` | `w` |
| `ã`, `õ`; `am/em/im/om/um` before a consonant or at a word end | nasal vowels; final `-am/-ão` adds `w`, final `-em` adds `j` |
| unstressed final `o/e` | `u/i` |
| stress | written accents take precedence; otherwise penultimate, except final stress on `r/l/z/x/i/u/im/um` and their plurals |

This is a compact spelling front end, not a morphology-aware dictionary. It does not yet model
regional lexical differences, all Portuguese diphthong and hiatus exceptions, or number expansion.
A native pt-BR speaker's review is especially welcome.
