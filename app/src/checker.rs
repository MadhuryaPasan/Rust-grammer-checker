#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Category {
    Grammar,
    Spelling,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum EnglishVariant {
    #[default]
    UnitedStates,
    UnitedKingdom,
}

impl EnglishVariant {
    pub fn from_setting(value: &str) -> Self {
        match value {
            "UK English" => Self::UnitedKingdom,
            _ => Self::UnitedStates,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::UnitedStates => "US English",
            Self::UnitedKingdom => "UK English",
        }
    }
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Self::Grammar => "Grammar",
            Self::Spelling => "Spelling",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Issue {
    pub start: usize,
    pub end: usize,
    pub incorrect: String,
    pub suggestion: String,
    pub category: Category,
    pub rule: String,
    pub explanation: String,
}

struct Rule {
    incorrect: &'static str,
    suggestion: &'static str,
    category: Category,
    rule: &'static str,
    explanation: &'static str,
}

const RULES: &[Rule] = &[
    Rule {
        incorrect: "alot",
        suggestion: "a lot",
        category: Category::Spelling,
        rule: "Compound words",
        explanation:
            "“A lot” is always written as two words. “Alot” is not a standard English word.",
    },
    Rule {
        incorrect: "teh",
        suggestion: "the",
        category: Category::Spelling,
        rule: "Commonly confused words",
        explanation: "This looks like a transposition typo. The standard spelling is “the”.",
    },
    Rule {
        incorrect: "recieve",
        suggestion: "receive",
        category: Category::Spelling,
        rule: "I before E",
        explanation: "In “receive”, the letters follow “c”, so the spelling is “ei”.",
    },
    Rule {
        incorrect: "definately",
        suggestion: "definitely",
        category: Category::Spelling,
        rule: "Common spelling",
        explanation: "“Definitely” is spelled with “-finite-”, not “-finate-”.",
    },
    Rule {
        incorrect: "seperate",
        suggestion: "separate",
        category: Category::Spelling,
        rule: "Common spelling",
        explanation: "“Separate” has “a” in both the second and third syllables.",
    },
    Rule {
        incorrect: "occured",
        suggestion: "occurred",
        category: Category::Spelling,
        rule: "Double consonants",
        explanation: "The final syllable of “occur” is stressed, so add a second “r” before “-ed”.",
    },
    Rule {
        incorrect: "untill",
        suggestion: "until",
        category: Category::Spelling,
        rule: "Common spelling",
        explanation: "“Until” has one final “l”.",
    },
    Rule {
        incorrect: "becuase",
        suggestion: "because",
        category: Category::Spelling,
        rule: "Common spelling",
        explanation: "The correct spelling is “because”.",
    },
    Rule {
        incorrect: "grammer",
        suggestion: "grammar",
        category: Category::Spelling,
        rule: "Common spelling",
        explanation: "“Grammar” ends in “-ar”, not “-er”.",
    },
    Rule {
        incorrect: "sentance",
        suggestion: "sentence",
        category: Category::Spelling,
        rule: "Common spelling",
        explanation: "“Sentence” has an “e” in its final syllable.",
    },
    Rule {
        incorrect: "their is",
        suggestion: "there is",
        category: Category::Grammar,
        rule: "There, their, and they're",
        explanation: "Use “there” to introduce or point to something. “Their” shows possession.",
    },
    Rule {
        incorrect: "your welcome",
        suggestion: "you're welcome",
        category: Category::Grammar,
        rule: "Your and you're",
        explanation: "“You're” means “you are”. “Your” indicates that something belongs to you.",
    },
    Rule {
        incorrect: "could of",
        suggestion: "could have",
        category: Category::Grammar,
        rule: "Could have, not could of",
        explanation:
            "Use “could have” (often contracted to “could've”). “Could of” comes from a mishearing.",
    },
    Rule {
        incorrect: "I has",
        suggestion: "I have",
        category: Category::Grammar,
        rule: "Subject–verb agreement",
        explanation: "The subject “I” takes “have”, not “has”.",
    },
    Rule {
        incorrect: "they is",
        suggestion: "they are",
        category: Category::Grammar,
        rule: "Subject–verb agreement",
        explanation: "The plural subject “they” takes “are”, not “is”.",
    },
];

const UK_SPELLINGS_FOR_US: &[Rule] = &[
    Rule {
        incorrect: "colour",
        suggestion: "color",
        category: Category::Spelling,
        rule: "US English spelling",
        explanation: "US English uses “color”; UK English uses “colour”.",
    },
    Rule {
        incorrect: "favourite",
        suggestion: "favorite",
        category: Category::Spelling,
        rule: "US English spelling",
        explanation: "US English uses “favorite”; UK English uses “favourite”.",
    },
    Rule {
        incorrect: "centre",
        suggestion: "center",
        category: Category::Spelling,
        rule: "US English spelling",
        explanation: "US English uses “center”; UK English uses “centre”.",
    },
    Rule {
        incorrect: "organise",
        suggestion: "organize",
        category: Category::Spelling,
        rule: "US English spelling",
        explanation: "US English usually uses “organize”; UK English commonly uses “organise”.",
    },
    Rule {
        incorrect: "travelling",
        suggestion: "traveling",
        category: Category::Spelling,
        rule: "US English spelling",
        explanation: "US English uses “traveling”; UK English uses “travelling”.",
    },
];

const US_SPELLINGS_FOR_UK: &[Rule] = &[
    Rule {
        incorrect: "color",
        suggestion: "colour",
        category: Category::Spelling,
        rule: "UK English spelling",
        explanation: "UK English uses “colour”; US English uses “color”.",
    },
    Rule {
        incorrect: "favorite",
        suggestion: "favourite",
        category: Category::Spelling,
        rule: "UK English spelling",
        explanation: "UK English uses “favourite”; US English uses “favorite”.",
    },
    Rule {
        incorrect: "center",
        suggestion: "centre",
        category: Category::Spelling,
        rule: "UK English spelling",
        explanation: "UK English uses “centre”; US English uses “center”.",
    },
    Rule {
        incorrect: "organize",
        suggestion: "organise",
        category: Category::Spelling,
        rule: "UK English spelling",
        explanation: "UK English commonly uses “organise”; US English uses “organize”.",
    },
    Rule {
        incorrect: "traveling",
        suggestion: "travelling",
        category: Category::Spelling,
        rule: "UK English spelling",
        explanation: "UK English uses “travelling”; US English uses “traveling”.",
    },
];

pub fn analyze_with_variant(text: &str, variant: EnglishVariant) -> Vec<Issue> {
    let regional_rules = match variant {
        EnglishVariant::UnitedStates => UK_SPELLINGS_FOR_US,
        EnglishVariant::UnitedKingdom => US_SPELLINGS_FOR_UK,
    };
    let mut issues = RULES
        .iter()
        .chain(regional_rules.iter())
        .flat_map(|rule| {
            let needle = rule.incorrect.as_bytes();
            text.as_bytes()
                .windows(needle.len())
                .enumerate()
                .filter(move |(start, found)| {
                    found.eq_ignore_ascii_case(needle)
                        && has_word_boundaries(text, *start, *start + found.len())
                })
                .map(move |(start, found)| {
                    let incorrect = &text[start..start + found.len()];
                    let suggestion = if incorrect.chars().all(char::is_uppercase) {
                        rule.suggestion.to_uppercase()
                    } else if incorrect.chars().next().is_some_and(char::is_uppercase) {
                        let mut suggestion = rule.suggestion.to_owned();
                        if let Some(first) = suggestion.get_mut(..1) {
                            first.make_ascii_uppercase();
                        }
                        suggestion
                    } else {
                        rule.suggestion.to_owned()
                    };

                    Issue {
                        start,
                        end: start + found.len(),
                        incorrect: incorrect.to_owned(),
                        suggestion,
                        category: rule.category,
                        rule: rule.rule.to_owned(),
                        explanation: rule.explanation.to_owned(),
                    }
                })
        })
        .collect::<Vec<_>>();

    issues.sort_by_key(|issue| issue.start);
    let mut previous_end = 0;
    issues.retain(|issue| {
        if issue.start < previous_end {
            false
        } else {
            previous_end = issue.end;
            true
        }
    });
    issues
}

fn has_word_boundaries(text: &str, start: usize, end: usize) -> bool {
    let before_is_word = text[..start]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric);
    let after_is_word = text[end..]
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric);
    !before_is_word && !after_is_word
}

#[cfg(test)]
mod tests {
    use super::{analyze_with_variant, Category, EnglishVariant, Issue};

    fn analyze(text: &str) -> Vec<Issue> {
        analyze_with_variant(text, EnglishVariant::UnitedStates)
    }

    #[test]
    fn finds_grammar_and_spelling_issues_with_replacement_ranges() {
        let text = "Their is alot of work.";
        let issues = analyze(text);

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].incorrect, "Their is");
        assert_eq!(issues[0].suggestion, "There is");
        assert_eq!(issues[0].category, Category::Grammar);
        assert_eq!(&text[issues[0].start..issues[0].end], "Their is");
        assert_eq!(issues[1].incorrect, "alot");
        assert_eq!(issues[1].suggestion, "a lot");
        assert_eq!(issues[1].category, Category::Spelling);
    }

    #[test]
    fn corrected_text_has_no_matching_issues() {
        assert!(analyze("There is a lot of work. You're welcome.").is_empty());
    }

    #[test]
    fn catches_common_spelling_errors_without_matching_inside_other_words() {
        let issues =
            analyze("I definately recieve a sentance; the separate parts occured untill now.");
        let incorrect = issues
            .iter()
            .map(|issue| issue.incorrect.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            incorrect,
            vec!["definately", "recieve", "sentance", "occured", "untill"]
        );
        assert!(analyze("A separate group received it.").is_empty());
        assert!(analyze("The grammerian arrived.").is_empty());
    }

    #[test]
    fn applies_us_and_uk_spellings_without_flagging_the_selected_variant() {
        let us_issues = analyze_with_variant(
            "The colour was my favourite at the centre.",
            EnglishVariant::UnitedStates,
        );
        assert_eq!(
            us_issues
                .iter()
                .map(|issue| issue.suggestion.as_str())
                .collect::<Vec<_>>(),
            vec!["color", "favorite", "center"]
        );

        let uk_issues = analyze_with_variant(
            "The color was my favorite at the center.",
            EnglishVariant::UnitedKingdom,
        );
        assert_eq!(
            uk_issues
                .iter()
                .map(|issue| issue.suggestion.as_str())
                .collect::<Vec<_>>(),
            vec!["colour", "favourite", "centre"]
        );
        assert!(
            analyze_with_variant("The colour is bright.", EnglishVariant::UnitedKingdom).is_empty()
        );
        assert!(
            analyze_with_variant("The color is bright.", EnglishVariant::UnitedStates).is_empty()
        );
    }
}
