use super::*;

pub(super) fn doc() -> GrammarDoc {
    GrammarDoc {
        language: "russian".to_string(),
        intro: "Russian (русский) is an East Slavic language in the Indo-European family — so unlike \
                Georgian, plenty transfers if you know another Slavic or case-heavy language, but for an \
                English speaker the walls are real. It's written in Cyrillic, has no articles, marks roles \
                with six cases across three genders, makes adjectives agree with their nouns, and organizes \
                its whole verb system around aspect rather than tense. Two features cost the most effort: \
                the case system (six cases, with gender-dependent endings) and verbal aspect (every verb is \
                really an imperfective/perfective pair). The sections below run from the script to those two \
                walls."
            .to_string(),
        sections: vec![
            section(
                "Sounds & script",
                vec![
                    para(
                        "Cyrillic has 33 letters. Many map to familiar sounds, but a few are false friends: \
                         е is 'ye', н is 'n', р is 'r', с is 's', у is 'u', в is 'v', and х is a throaty 'kh'.",
                    ),
                    note(
                        "Stress is mobile, unpredictable, and unmarked in ordinary text — and it matters, \
                         because unstressed vowels reduce. Unstressed о is pronounced like 'a': молоко \
                         ('milk') comes out as 'malakó'. You learn each word's stress along with the word.",
                    ),
                    para(
                        "Most consonants come in a hard and a soft (palatalized) pair. Softness is shown by \
                         the following vowel letter (я ё ю е и) or by the soft sign ь, and the contrast is \
                         phonemic — it distinguishes words.",
                    ),
                    para(
                        "Voiced consonants devoice at the end of a word: хлеб ('bread') ends in a 'p' sound, \
                         год ('year') in a 't'. So, as with the vowels, spelling and pronunciation part ways.",
                    ),
                ],
            ),
            section(
                "Nouns: three genders, no articles",
                vec![
                    para(
                        "Every noun has one of three genders, usually readable from its ending, and there \
                         are no articles — стол is 'a table' or 'the table' from context.",
                    ),
                    table(
                        "Telling gender from the ending",
                        &["Gender", "Typical endings", "Examples"],
                        &[
                            &["Masculine", "consonant, -й, some -ь", "стол (table), музей (museum), словарь (dictionary)"],
                            &["Feminine", "-а, -я, some -ь", "книга (book), земля (land), ночь (night)"],
                            &["Neuter", "-о, -е, -мя", "окно (window), море (sea), имя (name)"],
                        ],
                    ),
                    note(
                        "The soft sign -ь is the catch: it ends both some masculine nouns (словарь) and some \
                         feminine ones (ночь), so for -ь words you simply learn the gender with the word.",
                    ),
                ],
            ),
            section(
                "The first wall: six cases",
                vec![
                    para(
                        "A noun changes its ending to mark its role. There are six cases, and because the \
                         endings depend on gender and declension they aren't one-size-fits-all — but the jobs \
                         are constant. Using стол ('table', masculine) as one model:",
                    ),
                    table(
                        "The six cases — стол 'table'",
                        &["Case", "Form", "Job"],
                        &[
                            &["Nominative", "стол", "subject; the dictionary form"],
                            &["Genitive", "стола", "'of'; absence; after many prepositions and the numbers 5+"],
                            &["Dative", "столу", "indirect object — 'to / for'; after к, по"],
                            &["Accusative", "стол", "direct object; direction (в/на + accusative)"],
                            &["Instrumental", "столом", "'by means of / with' a tool; after с ('together with')"],
                            &["Prepositional", "столе", "ONLY after prepositions — location and 'about' (в, на, о)"],
                        ],
                    ),
                    note(
                        "Animacy bites in the accusative: for an animate masculine noun the accusative copies \
                         the genitive, not the nominative. 'I see a table' is Я вижу стол, but 'I see a man' \
                         is Я вижу человека (genitive-shaped). Every animate noun does this in the plural.",
                    ),
                ],
            ),
            section(
                "Adjectives agree",
                vec![
                    para(
                        "Adjectives agree with their noun in gender, number, and case — so a single adjective \
                         has many forms.",
                    ),
                    ex(
                        "новый стол / новая книга / новое окно / новые столы",
                        "nóvyj stol / nóvaya kníga / nóvoye oknó / nóvyye stolý",
                        "new table / new book / new window / new tables — one adjective, four agreements",
                    ),
                    para(
                        "And the adjective declines through all six cases beside the noun, so 'with a new \
                         book' shifts both words: с новой книгой (instrumental).",
                    ),
                ],
            ),
            section(
                "Prepositions govern cases",
                vec![
                    para(
                        "Each preposition demands a particular case — and some demand different cases for \
                         different meanings. The preposition and the ending work as a unit.",
                    ),
                    table(
                        "Common prepositions and their cases",
                        &["Preposition", "Case", "Meaning / example"],
                        &[
                            &["в, на", "prepositional", "location: в столе (in the table), на столе (on the table)"],
                            &["в, на", "accusative", "direction: на стол (onto the table)"],
                            &["у", "genitive", "'at / by / have': у меня (I have)"],
                            &["с", "instrumental", "'together with': с другом (with a friend)"],
                            &["к", "dative", "'toward': к столу (toward the table)"],
                        ],
                    ),
                    note(
                        "в and на take the prepositional for being somewhere but the accusative for moving \
                         there — same preposition, different case, different meaning.",
                    ),
                ],
            ),
            section(
                "The second wall: verbal aspect",
                vec![
                    para(
                        "Russian verbs are built around aspect, not tense. Almost every verb is a pair: an \
                         imperfective (process, repetition, ongoing) and a perfective (a single completed \
                         action with a result).",
                    ),
                    bullets(&[
                        "Imperfective — писать ('to write / be writing'): has present, past, and future.",
                        "Perfective — написать ('to write and finish'): one completed act; it has NO present tense.",
                        "Because the perfective has no present, its present-tense forms mean the future: напишу = 'I will write (and complete it)'.",
                        "The past tense agrees in gender and number, not person — it descends from an old participle.",
                    ]),
                    para("The past agreeing by gender is the surprise:"),
                    ex(
                        "он писал / она писала / оно писало / они писали",
                        "on pisál / oná pisála / onó pisálo / oní pisáli",
                        "he / she / it / they were writing — gender & number, never person",
                    ),
                    para("The present tense uses personal endings; verbs fall into two conjugations:"),
                    ex(
                        "я читаю, ты читаешь, он читает",
                        "ya chitáyu, ty chitáyesh, on chitáyet",
                        "I / you / he read — 1st conjugation (-ю / -ешь / -ет)",
                    ),
                    ex(
                        "я говорю, ты говоришь, он говорит",
                        "ya govoryú, ty govorísh, on govorít",
                        "I / you / he speak — 2nd conjugation (-ю / -ишь / -ит)",
                    ),
                    note(
                        "Aspect is the single biggest investment in Russian: you don't learn a verb, you learn \
                         a pair, and the wrong aspect changes the meaning. Treat it like the Georgian verb — \
                         learn pairs as units, not from rules.",
                    ),
                ],
            ),
            section(
                "Numbers govern case",
                vec![
                    para(
                        "Russian numbers don't just sit in front of a noun — they dictate its case, and the \
                         rule changes with the number.",
                    ),
                    table(
                        "What case a number forces",
                        &["Number", "Noun form", "Example"],
                        &[
                            &["1 (один / одна / одно)", "nominative singular", "один стол (one table)"],
                            &["2, 3, 4", "genitive singular", "два стола, три книги (two tables, three books)"],
                            &["5 and up", "genitive plural", "пять столов, десять книг (five tables, ten books)"],
                        ],
                    ),
                    note(
                        "So 'two tables' and 'five tables' use different forms of the same noun: два стола \
                         (genitive singular) vs пять столов (genitive plural). Compound numbers follow their \
                         last word, and 2 has a feminine form — два стола but две книги.",
                    ),
                ],
            ),
            section(
                "Word order",
                vec![
                    para(
                        "Because the endings already mark who does what, word order is flexible. The neutral \
                         order is subject–verb–object, but words move freely for emphasis — the new or \
                         important information tends to fall last.",
                    ),
                    para(
                        "And, like Georgian, there are no articles: context and word order — not 'a' / 'the' — \
                         signal whether a noun is new or already known.",
                    ),
                ],
            ),
        ],
        drills: vec![
            mc(
                "Russian nouns decline for how many cases?",
                &["four", "five", "six", "seven"],
                2,
                "Nominative, genitive, dative, accusative, instrumental, prepositional.",
            ),
            mc(
                "Russian verbs are organized around which contrast?",
                &["past vs present", "imperfective vs perfective aspect", "active vs passive", "hard vs soft"],
                1,
                "Almost every verb is an aspect pair — писать (process) / написать (completed).",
            ),
            tin(
                "Give the genitive singular of стол (“table”).",
                &["стола"],
                "Masculine -а in the genitive: стол → стола. It's also the form after 2–4 and many prepositions.",
            ),
            mc(
                "After пять (5), the counted noun takes which form?",
                &["nominative singular", "genitive singular", "genitive plural", "accusative"],
                2,
                "5 and up take the genitive plural: пять столов. (2–4 take the genitive singular.)",
            ),
            mc(
                "After два, три, четыре (2–4), the noun takes…",
                &["nominative plural", "genitive singular", "genitive plural", "dative"],
                1,
                "два стола, три книги — genitive singular. 5+ switches to genitive plural.",
            ),
            mc(
                "The Russian past tense agrees with its subject in…",
                &["person and number", "gender and number", "case only", "nothing"],
                1,
                "он писал / она писала / они писали — gender and number, not person.",
            ),
            tin(
                "What gender is the noun книга (“book”)? (one word)",
                &["feminine", "fem", "f"],
                "Nouns ending in -а are feminine: книга.",
            ),
            mc(
                "A perfective verb such as написать has no…",
                &["past tense", "present tense", "future tense", "infinitive"],
                1,
                "The perfective has no present; its present-form endings express the future (напишу = “I will write”).",
            ),
            mc(
                "For an animate masculine noun, the accusative copies the…",
                &["nominative", "genitive", "dative", "instrumental"],
                1,
                "Я вижу человека (genitive-shaped), but Я вижу стол (nominative-shaped) for an inanimate noun.",
            ),
            mc(
                "Which case appears ONLY after a preposition?",
                &["genitive", "dative", "instrumental", "prepositional"],
                3,
                "The prepositional never stands alone — в столе, на столе, о столе.",
            ),
            tin(
                "Give the “я” (I) present form of читать (“to read”).",
                &["читаю"],
                "1st-conjugation present: я читаю, ты читаешь, он читает.",
            ),
            tin(
                "How many letters are in the Russian (Cyrillic) alphabet? (digit)",
                &["33"],
                "Thirty-three — including ь and ъ, which mark softness/hardness rather than standing for sounds.",
            ),
        ],
    }
}
