use crate::wiki_scraper::parse_names::parse_word_table;

const VALID_WORDS: &str = "
*There are probably ones we haven't uncovered, so feel free to edit and add if there is something not on the list!
==Adjectives==
''Below is a list of Adjectives.''
{| class=\"sortable mw-collapsible mw-collapsed wikitable\"
|+Prefix
||Adorable
|-
|Aggresive [sic]
|-
|Amazing
|-
|Attractive
|-
|}

";

const INVALID_WORDS: &str = "
==Nouns==
''Below is a list of nouns.''
{| class=\"sortable mw-collapsible mw-collapsed wikitable\"
|+Noun
|Abs # This is ignored. Abs is still valid. <---
|-
|Super Auto # Inner space is fine
|-
|_Aunties___
|-
|1Aunties1
|-
|}
";

const INVALID_CATEG: &str = "
==Dogs==
''Below is a list of dogs.''
{| class=\"sortable mw-collapsible mw-collapsed wikitable\"
|+Dog
|Otto
|-
|}
";

#[test]
fn test_parse_categs() {
    let mut valid_words = vec![];
    let mut invalid_words = vec![];
    let mut invalid_categs = vec![];

    parse_word_table(VALID_WORDS, &mut valid_words);
    parse_word_table(INVALID_WORDS, &mut invalid_words);
    parse_word_table(INVALID_CATEG, &mut invalid_categs);

    assert_eq!(valid_words.len(), 4);
    assert_eq!(invalid_words.len(), 2);
    assert!(invalid_categs.is_empty());
}
