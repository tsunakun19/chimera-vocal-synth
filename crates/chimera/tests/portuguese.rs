//! Brazilian Portuguese spelling rules and sung syllables.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::inventory::phoneme;
use chimera::phonetics::mandarin::MandarinLexicon;
use chimera::phonetics::planner::is_vowel;
use chimera::phonetics::portuguese::{lyrics_to_syllables, to_phrases, word};
use chimera::phonetics::syllables::{lyrics_in, Lang};
use chimera::phonetics::{Segment, Syllable};

fn ipa(parts: &[Segment]) -> String {
    parts.iter().map(|s| s.ipa.as_str()).collect::<String>()
}

fn shown(s: &Syllable) -> String {
    format!("{}.{}.{}", ipa(&s.onset), s.nucleus.ipa, ipa(&s.coda))
}

fn text(input: &str) -> Vec<String> {
    lyrics_to_syllables(input)
        .into_iter()
        .map(|s| s.text)
        .collect()
}

fn stressed(input: &str) -> Vec<String> {
    lyrics_to_syllables(input)
        .into_iter()
        .filter(|s| s.nucleus.stress == Some(1))
        .map(|s| s.text)
        .collect()
}

#[test]
fn pt_br_examples_syllabify_and_keep_the_spelling() {
    assert_eq!(
        text("tia dia carro cantar pão mãe filho ninho casa você Brasil parede"),
        [
            "ti", "a", "di", "a", "car", "ro", "can", "tar", "pão", "mãe", "fi", "lho", "ni",
            "nho", "ca", "sa", "vo", "cê", "Bra", "sil", "pa", "re", "de"
        ],
    );

    let syllables =
        lyrics_to_syllables("tia dia carro cantar pão mãe filho ninho casa você Brasil parede");
    assert_eq!(shown(&syllables[0]), "tʃ.i.");
    assert_eq!(shown(&syllables[2]), "dʒ.i.");
    assert_eq!(shown(&syllables[5]), "h.u.");
    assert_eq!(shown(&syllables[7]), "t.a.h");
    assert_eq!(shown(&syllables[8]), "p.ə.w");
    assert_eq!(syllables[8].nucleus.nasalized, Some(true));
    assert_eq!(shown(&syllables[9]), "m.ə.j");
    assert_eq!(syllables[9].nucleus.nasalized, Some(true));
    assert_eq!(shown(&syllables[11]), "ʎ.u.");
    assert_eq!(shown(&syllables[13]), "ɲ.u.");
    assert_eq!(shown(&syllables[15]), "z.ə.");
    assert_eq!(shown(&syllables[17]), "s.e.");
    assert_eq!(shown(&syllables[19]), "z.i.w");
    assert_eq!(shown(&syllables[22]), "dʒ.i.");
}

#[test]
fn vowel_reduction_nasal_endings_and_stress_rules() {
    let bem = lyrics_to_syllables("bem");
    assert_eq!(shown(&bem[0]), "b.e.j");
    assert_eq!(bem[0].nucleus.nasalized, Some(true));
    let falam = lyrics_to_syllables("falam");
    assert_eq!(text("falam"), ["fa", "lam"]);
    assert_eq!(shown(&falam[1]), "l.ə.w");
    assert_eq!(falam[1].nucleus.nasalized, Some(true));
    assert_eq!(word("parede").last().unwrap().ipa, "i");
    assert_eq!(
        word("carro")
            .iter()
            .map(|s| s.ipa.as_str())
            .collect::<Vec<_>>(),
        ["k", "a", "h", "u"]
    );

    assert_eq!(
        stressed("tia dia carro cantar pão mãe filho ninho casa você Brasil parede"),
        ["ti", "di", "car", "tar", "pão", "mãe", "fi", "ni", "ca", "cê", "sil", "re"],
    );
}

#[test]
fn hiatus_and_qu_gu_glides_keep_their_own_note_boundaries() {
    assert_eq!(
        text("Rio água quase guitarra"),
        ["Ri", "o", "á", "gua", "qua", "se", "gui", "tar", "ra"]
    );
    let agua = lyrics_to_syllables("água");
    assert_eq!(shown(&agua[1]), "gw.ə.");
    assert_eq!(stressed("Rio água"), ["Ri", "á"]);
}

#[test]
fn spelling_rules_cover_digraphs_and_voicing() {
    let phrases = to_phrases("tia dia carro cantar pão mãe filho ninho casa você Brasil parede");
    let ipa: Vec<_> = phrases[0].segments.iter().map(|s| s.ipa.as_str()).collect();
    assert!(ipa.contains(&"tʃ"));
    assert!(ipa.contains(&"dʒ"));
    assert!(ipa.contains(&"h"));
    assert!(ipa.contains(&"ɲ"));
    assert!(ipa.contains(&"ʎ"));
    assert!(ipa.contains(&"z"));
    let passo = lyrics_to_syllables("passo");
    assert_eq!(text("passo"), ["pas", "so"]);
    assert_eq!(shown(&passo[1]), "s.u.");
    assert!(phrases[0]
        .segments
        .iter()
        .all(|s| is_vowel(&s.ipa, BodyMix::MALE) || phoneme(&s.ipa, BodyMix::MALE).is_some()));
}

#[test]
fn portuguese_is_selected_only_for_the_latin_script_language() {
    let no_dictionary = |_: &str, _: bool| Vec::<Segment>::new();
    let chosen = lyrics_in("casa", &no_dictionary, &MandarinLexicon::new(), Lang::Pt);
    assert_eq!(
        chosen.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
        ["ca", "sa"]
    );
    let english = lyrics_in("casa", &no_dictionary, &MandarinLexicon::new(), Lang::En);
    assert!(english.iter().all(|s| s.nucleus.ipa != "ə"));
    let agua = lyrics_in("água", &no_dictionary, &MandarinLexicon::new(), Lang::Pt);
    assert_eq!(
        agua.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
        ["á", "gua"]
    );
    assert_eq!(
        agua[1]
            .onset
            .iter()
            .map(|s| s.ipa.as_str())
            .collect::<String>(),
        "gw"
    );
    assert_eq!(
        lyrics_in("pão", &no_dictionary, &MandarinLexicon::new(), Lang::Pt)[0].text,
        "pão"
    );
}

#[test]
fn portuguese_accents_win_over_ambiguous_pinyin() {
    let no_dictionary = |_: &str, _: bool| Vec::<Segment>::new();
    let words = lyrics_in(
        "pó nó só",
        &no_dictionary,
        &MandarinLexicon::new(),
        Lang::Pt,
    );
    assert_eq!(
        words.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
        ["pó", "nó", "só"]
    );
    assert_eq!(shown(&words[0]), "p.ɔ.");
    assert_eq!(shown(&words[1]), "n.ɔ.");
    assert_eq!(shown(&words[2]), "s.ɔ.");

    let pinyin = lyrics_in("nǐ", &no_dictionary, &MandarinLexicon::new(), Lang::Pt);
    assert_eq!(pinyin.len(), 1);
    assert_eq!(pinyin[0].nucleus.stress, None);
}
