use super::*;

pub(super) fn doc() -> GrammarDoc {
    GrammarDoc {
        language: "georgian".to_string(),
        intro: "Georgian (ქართული) is the literary language of the Kartvelian family — unrelated to \
                Indo-European, so almost nothing transfers from English, Russian, or the Romance \
                languages. It is agglutinative (words are built by stacking meaningful pieces), has no \
                grammatical gender and no articles, marks roles with seven noun cases and postpositions \
                rather than prepositions, and folds subject- and object-agreement into a single verb. \
                Two features trip up nearly everyone: the case of the subject changes with the verb's \
                tense, and the verb system is vast. The sections below run from the easy wins toward \
                those two walls."
            .to_string(),
        sections: vec![
            section(
                "Sounds",
                vec![
                    para(
                        "Five vowels — a e i o u — each a single steady value. There is no vowel length \
                         and no reduction: unstressed vowels keep their quality, unlike Russian.",
                    ),
                    note(
                        "The real trap is the consonants. Where English has one sound, Georgian often \
                         has three: voiced, aspirated (a puff of air), and ejective (a sharp, \
                         glottalized 'popped' release made with no airflow from the lungs). The \
                         contrast is phonemic — swap one for another and you've said a different word.",
                    ),
                    table(
                        "The stop & affricate triplets",
                        &["Voiced", "Aspirated", "Ejective"],
                        &[
                            &["ბ — b", "ფ — p", "პ — p'"],
                            &["დ — d", "თ — t", "ტ — t'"],
                            &["გ — g", "ქ — k", "კ — k'"],
                            &["ძ — dz", "ც — ts", "წ — ts'"],
                            &["ჯ — j", "ჩ — ch", "ჭ — ch'"],
                        ],
                    ),
                    para(
                        "Outside the triplets sit the uvulars: ყ is an ejective /q'/ with no plain \
                         partner, alongside the fricatives ღ /gh/ and ხ /kh/. Georgian also tolerates \
                         consonant clusters English never would — მწვანე (mtsvane, 'green'), ფრჩხილი \
                         (frchkhili, 'fingernail') — and they're pronounced exactly as written. There \
                         are no silent letters.",
                    ),
                ],
            ),
            section(
                "Nouns: seven cases, no gender",
                vec![
                    para(
                        "A noun has no gender and takes no article — კაცი is 'man', 'a man', or 'the \
                         man' from context. What it carries instead is a case ending marking its role \
                         in the clause. Using კაცი ('man') as the model consonant-stem noun:",
                    ),
                    table(
                        "The seven cases — კაცი 'man'",
                        &["Case", "Form", "Job"],
                        &[
                            &["Nominative", "კაცი — k'atsi", "subject (most tenses); the citation form"],
                            &["Ergative", "კაცმა — k'atsma", "subject of a transitive verb in the aorist"],
                            &["Dative", "კაცს — k'atss", "indirect object; direct object in the present; many postpositions"],
                            &["Genitive", "კაცის — k'atsis", "possession — 'of the man'"],
                            &["Instrumental", "კაცით — k'atsit", "'by means of, with' (a tool)"],
                            &["Adverbial", "კაცად — k'atsad", "'as / into a man' — role or transformation"],
                            &["Vocative", "კაცო — k'atso", "direct address — 'O man!'"],
                        ],
                    ),
                    note(
                        "Vowel-stem nouns (e.g. დედა 'mother') take slightly shorter endings — ergative \
                         -მ rather than -მა, and so on — but the seven-case skeleton is identical.",
                    ),
                ],
            ),
            section(
                "The first wall: case follows the verb's tense",
                vec![
                    para(
                        "This is the feature with no English analogue. The case of the subject — and of \
                         the object — is not fixed; it depends on which 'series' of tenses the verb is \
                         in. Georgian sorts its tense-aspect-mood forms (called screeves) into three \
                         series, and each series hands out cases differently. For a transitive verb:",
                    ),
                    table(
                        "Case by series (transitive verb)",
                        &["Series", "Example tenses", "Subject", "Direct object"],
                        &[
                            &["I", "present, future, imperfect", "Nominative", "Dative"],
                            &["II", "aorist (simple past), optative", "Ergative", "Nominative"],
                            &["III", "perfect, pluperfect", "Dative", "Nominative"],
                        ],
                    ),
                    para("Watch one sentence — 'the man writes / wrote the letter' — move through the series:"),
                    ex(
                        "კაცი წერს წერილს",
                        "k'atsi ts'ers ts'erils",
                        "The man writes the letter. — Series I: subject NOM, object DAT",
                    ),
                    ex(
                        "კაცმა დაწერა წერილი",
                        "k'atsma dats'era ts'erili",
                        "The man wrote the letter. — Series II: subject ERG, object NOM",
                    ),
                    ex(
                        "კაცს დაუწერია წერილი",
                        "k'atss dauts'eria ts'erili",
                        "The man has (evidently) written the letter. — Series III: subject DAT, object NOM",
                    ),
                    note(
                        "'The man' is კაცი, კაცმა, or კაცს purely because of the verb's tense — nothing \
                         about the man changed. This is why case endings can't be learned in isolation \
                         from the verb: they're two halves of one system. (Intransitive and 'inversion' \
                         verbs follow still other patterns — see the verb section.)",
                    ),
                ],
            ),
            section(
                "Postpositions, not prepositions",
                vec![
                    para(
                        "Where English puts a word before the noun ('in the house'), Georgian attaches \
                         a postposition after it, and each one governs a particular case. They fuse \
                         onto the noun, so one written word often equals an English preposition + \
                         article + noun.",
                    ),
                    table(
                        "Common postpositions",
                        &["Ending", "Meaning", "Example"],
                        &[
                            &["-ში", "in", "სახლში — sakhlshi — in the house"],
                            &["-ზე", "on, about", "მაგიდაზე — magidaze — on the table"],
                            &["-თან", "at, by, with (a person)", "დედასთან — dedastan — at mother's"],
                            &["-თვის", "for (+ genitive)", "ბავშვისთვის — bavshvistvis — for the child"],
                            &["-დან", "from (out of)", "სახლიდან — sakhlidan — from the house"],
                        ],
                    ),
                ],
            ),
            section(
                "The second wall: the verb",
                vec![
                    para("Georgian verbs are the steepest climb in the language, for several reasons at once:"),
                    bullets(&[
                        "Polypersonal agreement — one verb agrees with its subject AND its object(s). A single word can be a whole clause.",
                        "Preverbs — a prefix (და-, მი-, მო-, გა-, შე-…) that adds direction and usually flips the verb to perfective / future.",
                        "Version vowels — a pre-radical vowel (a-, i-, u-, e-) that marks who the action is for.",
                        "Screeves & series — roughly eleven screeves grouped into the three series above; each verb is really a family of related forms.",
                    ]),
                    para("Agreement in one word — the prefix can encode the object's person:"),
                    ex("ვხედავ", "v-khedav", "I see (it). — v- marks the 1st-person subject"),
                    ex("გხედავ", "g-khedav", "I see you. — g- marks the 2nd-person object"),
                    ex("მხედავ", "m-khedav", "you see me. — m- marks the 1st-person object"),
                    para("Direction by preverb:"),
                    ex("მოდის / მიდის", "modis / midis", "he comes (here) / he goes (away) — mo- toward, mi- away"),
                    para("Who benefits, by version vowel:"),
                    ex("ვაკეთებ / ვუკეთებ", "v-a-keteb / v-u-keteb", "I make it / I make it for him — a- vs u- version"),
                    note(
                        "Don't try to derive verb forms from rules at first. Learn high-frequency verbs \
                         as whole paradigms — the way you'd learn irregular verbs — and let the patterns \
                         surface from exposure. This is the single biggest time sink in Georgian; budget \
                         for it deliberately.",
                    ),
                ],
            ),
            section(
                "Numbers are base-20",
                vec![
                    para(
                        "Georgian counts in twenties (vigesimal) — like French quatre-vingts, taken all \
                         the way. Above twenty, a number is a multiple of 20 plus a remainder.",
                    ),
                    table(
                        "Counting by twenties",
                        &["#", "Georgian", "Built from"],
                        &[
                            &["10", "ათი — ati", "—"],
                            &["20", "ოცი — otsi", "—"],
                            &["30", "ოცდაათი — otsdaati", "20 + 10"],
                            &["40", "ორმოცი — ormotsi", "2 × 20"],
                            &["50", "ორმოცდაათი — ormotsdaati", "2 × 20 + 10"],
                            &["60", "სამოცი — samotsi", "3 × 20"],
                            &["80", "ოთხმოცი — otkhmotsi", "4 × 20"],
                            &["100", "ასი — asi", "—"],
                        ],
                    ),
                    para("So 47 is ორმოცდაშვიდი (ormotsda-shvidi) = 2 × 20 + 7. Alien for a week, automatic after that."),
                ],
            ),
            section(
                "Word order & plurals",
                vec![
                    para(
                        "Because case endings already mark who does what, word order is flexible. The \
                         neutral order is subject–object–verb, but constituents move freely for emphasis \
                         without changing the grammar.",
                    ),
                    para(
                        "The everyday plural is -ები: სახლი → სახლები (sakhli → sakhlebi, 'houses'). An \
                         older plural (-ნი in the nominative, -თა in the oblique) survives in formal and \
                         literary registers — you'll read it long before you need to produce it.",
                    ),
                ],
            ),
        ],
        drills: vec![
            mc(
                "In the aorist (Series II), the subject of a transitive verb takes which case?",
                &["Nominative", "Ergative", "Dative", "Genitive"],
                1,
                "Series II flips the subject to the ergative (narrative) case: კაცმა, not კაცი.",
            ),
            tin(
                "Give the ergative of კაცი (“man”).",
                &["კაცმა"],
                "Consonant-stem nouns take -მა in the ergative: კაცი → კაცმა.",
            ),
            mc(
                "In the present tense (Series I), the direct object takes which case?",
                &["Nominative", "Dative", "Genitive", "Instrumental"],
                1,
                "Series I: subject nominative, direct object dative — კაცი წერს წერილს.",
            ),
            mc(
                "Which case marks the subject in the perfect (Series III)?",
                &["Nominative", "Ergative", "Dative", "Vocative"],
                2,
                "Series III inverts: the logical subject goes to the dative — კაცს დაუწერია.",
            ),
            tin(
                "Write the number 20 in Georgian.",
                &["ოცი"],
                "Georgian counts in twenties; 20 is ოცი (otsi), the base of the system.",
            ),
            mc(
                "Georgian numerals are built on which base?",
                &["base 10", "base 12", "base 20", "base 60"],
                2,
                "Vigesimal: 40 is ორმოცი (2×20), 60 is სამოცი (3×20).",
            ),
            mc(
                "ბ, ფ, and პ differ in…",
                &["vowel length", "voicing, aspiration, and ejection", "pitch", "stress"],
                1,
                "Voiced ბ /b/, aspirated ფ /p/, ejective პ /p'/ — the three-way contrast English lacks.",
            ),
            tin(
                "What case does the postposition -თვის (“for”) govern?",
                &["genitive", "gen"],
                "-თვის takes the genitive: ბავშვისთვის (“for the child”).",
            ),
            tin(
                "How many noun cases does Georgian have? (digit)",
                &["7", "seven"],
                "Nominative, ergative, dative, genitive, instrumental, adverbial, vocative.",
            ),
            tin(
                "Type the Georgian for “I see you” — one word.",
                &["გხედავ"],
                "Polypersonal: the prefix გ- marks the 2nd-person object, so the whole clause is one verb.",
            ),
        ],
    }
}
