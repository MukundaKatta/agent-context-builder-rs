//! Integration tests exercising the public API as an external consumer would.

use agent_context_builder::ContextBuilder;

#[test]
fn assembles_a_realistic_system_prompt() {
    let prompt = ContextBuilder::new()
        .section_with_priority("role", "You are a coding assistant.", 100)
        .section("style", "Be concise.")
        .section("rules", "Never invent APIs.")
        .section_with_priority("footer", "Respond in Markdown.", -10)
        .build();

    // Role floats to the top, footer sinks to the bottom, the rest keep
    // insertion order.
    assert_eq!(
        prompt,
        "You are a coding assistant.\n\nBe concise.\n\nNever invent APIs.\n\nRespond in Markdown."
    );
}

#[test]
fn toggling_sections_changes_output() {
    let builder = ContextBuilder::new()
        .section("base", "Base instructions.")
        .section("verbose", "Explain in detail.");

    let terse = builder.clone().disable("verbose").build();
    assert!(!terse.contains("Explain in detail."));
    assert_eq!(terse, "Base instructions.");

    let verbose = builder.build();
    assert!(verbose.contains("Explain in detail."));
}

#[test]
fn custom_separator_is_used_between_sections() {
    let prompt = ContextBuilder::new()
        .separator("\n---\n")
        .section("a", "alpha")
        .section("b", "beta")
        .build();
    assert_eq!(prompt, "alpha\n---\nbeta");
}

#[test]
fn updating_a_section_does_not_duplicate_it() {
    let builder = ContextBuilder::new()
        .section("role", "v1")
        .section("role", "v2")
        .section("role", "v3");
    assert_eq!(builder.len(), 1);
    assert_eq!(builder.get("role"), Some("v3"));
    assert_eq!(builder.build(), "v3");
}

#[test]
fn introspection_helpers_agree_with_build() {
    let builder = ContextBuilder::new()
        .section("a", "x")
        .section("b", "y")
        .disable("a");

    assert_eq!(builder.len(), 2);
    assert_eq!(builder.enabled_count(), 1);
    assert_eq!(builder.enabled_names(), vec!["b"]);
    assert!(builder.has("a"));
    assert!(!builder.is_enabled("a"));
    assert!(builder.is_enabled("b"));
}
