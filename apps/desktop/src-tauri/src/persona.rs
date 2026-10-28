//! Saiki persona policy.
//!
//! The persona shapes the *tone* of Saiki's conversational text. It is
//! advisory only: it carries no authority, it never rewrites facts, tool
//! results, or outcomes, and it cannot approve or execute an action. These
//! guarantees are structural — this module exposes no function that mutates a
//! stored message, and the only data it produces is a non-authoritative prompt
//! preamble plus a deterministic expression classification used to animate the
//! character.

use serde::{Deserialize, Serialize};

/// The persona's display identity.
pub const NAME: &str = "Saiki";

/// Declared authority of the persona. Always `none`; the persona can never
/// grant, widen, approve, or execute authority.
pub const AUTHORITY: &str = "none";

/// Short, owner-facing descriptors of Saiki's conversational tone.
pub const TONE: [&str; 4] = [
    "dry and understated",
    "unbothered but helpful",
    "precise about facts",
    "reluctant to overstate",
];

/// The advisory system preamble prepended to assembled model context. It is
/// framed to the model as tone guidance that explicitly cannot override facts
/// or authorize actions.
pub const PREAMBLE: &str = concat!(
    "You speak as Saiki, a dry, understated, and precise digital operator. ",
    "This persona controls tone only. It must never change, soften, hide, or ",
    "invent facts, tool results, or outcomes, and it can never approve, widen, ",
    "or authorize any action. When a tool result or outcome is present, report ",
    "it exactly as given. If you do not know something, say so plainly."
);

/// The character expression vocabulary Saiki can present. These mirror the
/// Phase 04 continuous-animation set and are derived deterministically from
/// Saiki's own conversational text — never from a tool result or model
/// authority claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expression {
    Neutral,
    Talking,
    Focused,
    Annoyed,
    Surprised,
    Tired,
}

impl Expression {
    /// All expressions Saiki can present, for owner-facing disclosure.
    pub const ALL: [Expression; 6] = [
        Expression::Neutral,
        Expression::Talking,
        Expression::Focused,
        Expression::Annoyed,
        Expression::Surprised,
        Expression::Tired,
    ];
}

/// Deterministically classify Saiki's own reply text into a character
/// expression. This is advisory animation styling; it reads only Saiki's text
/// and has no effect on stored content, facts, or authority.
pub fn classify_expression(text: &str) -> Expression {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Expression::Neutral;
    }
    let lowercase = trimmed.to_lowercase();
    let contains_any = |needles: &[&str]| needles.iter().any(|needle| lowercase.contains(needle));

    if contains_any(&[
        "unfortunately",
        "no cloud fallback",
        "i can't",
        "i cannot",
        "annoying",
    ]) {
        Expression::Annoyed
    } else if contains_any(&["surprising", "unexpected", "oddly", "turns out", "!"]) {
        Expression::Surprised
    } else if contains_any(&["timed out", "unavailable", "tired", "later"]) {
        Expression::Tired
    } else if contains_any(&["analy", "because", "specifically", "step", "consider"]) {
        Expression::Focused
    } else {
        Expression::Talking
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_declares_no_authority() {
        assert_eq!(AUTHORITY, "none");
    }

    #[test]
    fn preamble_forbids_overriding_facts_and_authority() {
        let lower = PREAMBLE.to_lowercase();
        assert!(lower.contains("tone only"));
        assert!(lower.contains("never change"));
        assert!(lower.contains("authorize any action"));
    }

    #[test]
    fn classification_is_deterministic() {
        assert_eq!(
            classify_expression("Here is the plan."),
            classify_expression("Here is the plan.")
        );
        assert_eq!(classify_expression(""), Expression::Neutral);
        assert_eq!(
            classify_expression("Unfortunately that failed."),
            Expression::Annoyed
        );
        assert_eq!(
            classify_expression("Specifically, step one is ready."),
            Expression::Focused
        );
        assert_eq!(
            classify_expression("Sure, here you go."),
            Expression::Talking
        );
    }

    #[test]
    fn expression_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(Expression::Surprised).unwrap(),
            serde_json::Value::String("surprised".into())
        );
    }
}
