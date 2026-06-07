//! Compose a small agent system prompt and print it.
//!
//! Run with: `cargo run --example system_prompt`

use agent_context_builder::ContextBuilder;

fn main() {
    let include_debug = std::env::args().any(|a| a == "--debug");

    let mut builder = ContextBuilder::new()
        // The role should always come first, so give it a high priority.
        .section_with_priority("role", "You are a meticulous coding assistant.", 100)
        .section("style", "Be concise and prefer code over prose.")
        .section("rules", "Never invent APIs. Cite the file you changed.")
        // A trailing instruction that should always render last.
        .section_with_priority("footer", "Respond in GitHub-flavored Markdown.", -100)
        // This section is only included when debugging.
        .section("debug", "Explain your reasoning step by step.");

    if !include_debug {
        builder = builder.disable("debug");
    }

    println!("{}", builder.build());

    eprintln!(
        "\n[{} of {} sections rendered]",
        builder.enabled_count(),
        builder.len()
    );
}
