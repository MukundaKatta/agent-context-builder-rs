/*!
`agent-context-builder`: compose LLM system prompts from named sections.

Build a system prompt from independently managed named sections. Sections
can be added, removed, reordered, and conditionally included. The final
prompt is rendered in explicit priority order (higher priority first),
falling back to insertion order for sections that share the same priority.

# Quick start

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .section("role", "You are a helpful assistant.")
    .section("rules", "Always cite sources.")
    .build();
assert!(prompt.contains("helpful assistant"));
```

# Ordering

By default every section has priority `0` and is rendered in the order it
was inserted. Use [`ContextBuilder::section_with_priority`] (or
[`ContextBuilder::set_priority`]) to float important sections to the top:

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .section_with_priority("footer", "End of prompt.", -10)
    .section_with_priority("header", "Start of prompt.", 10)
    .build();
assert!(prompt.find("Start").unwrap() < prompt.find("End").unwrap());
```

# Conditional sections

Sections can be toggled on and off without losing their content, which makes
it easy to assemble a prompt from feature flags:

```rust
use agent_context_builder::ContextBuilder;

let debug = false;
let mut builder = ContextBuilder::new()
    .section("role", "You are a helpful assistant.")
    .section("debug", "Explain your reasoning step by step.");
if !debug {
    builder = builder.disable("debug");
}
assert!(!builder.build().contains("step by step"));
```
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashSet;

#[derive(Debug, Clone)]
struct Section {
    name: String,
    content: String,
    priority: i32,
    enabled: bool,
}

/// Builds a system prompt from named, ordered sections.
///
/// `ContextBuilder` uses a fluent, owned-`self` API so calls can be chained.
/// Every mutating method returns the builder, and read-only methods such as
/// [`ContextBuilder::get`] or [`ContextBuilder::build`] borrow it.
///
/// See the [crate-level documentation](crate) for a tour of the main concepts.
#[derive(Debug, Default, Clone)]
pub struct ContextBuilder {
    sections: Vec<Section>,
    separator: String,
}

impl ContextBuilder {
    /// Create an empty builder with the default separator (a blank line,
    /// i.e. `"\n\n"`) between sections.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            separator: "\n\n".into(),
        }
    }

    /// Set the separator placed between sections when rendering.
    ///
    /// The default is a double newline (`"\n\n"`), which renders sections as
    /// blank-line-separated paragraphs.
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Add a named section, or replace the content of an existing section with
    /// the same name.
    ///
    /// The section is created with priority `0` and enabled. If a section with
    /// `name` already exists, only its content is updated; its priority and
    /// enabled flag are preserved.
    pub fn section(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        let name = name.into();
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.content = content.into();
        } else {
            self.sections.push(Section {
                name,
                content: content.into(),
                priority: 0,
                enabled: true,
            });
        }
        self
    }

    /// Add a section with an explicit priority (higher renders earlier).
    ///
    /// If a section with `name` already exists, both its content and priority
    /// are updated.
    pub fn section_with_priority(
        mut self,
        name: impl Into<String>,
        content: impl Into<String>,
        priority: i32,
    ) -> Self {
        let name = name.into();
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.content = content.into();
            s.priority = priority;
        } else {
            self.sections.push(Section {
                name,
                content: content.into(),
                priority,
                enabled: true,
            });
        }
        self
    }

    /// Set the priority of an existing section. No-op if the section is absent.
    pub fn set_priority(mut self, name: &str, priority: i32) -> Self {
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.priority = priority;
        }
        self
    }

    /// Disable a section by name so it is omitted from [`build`](Self::build)
    /// output. Its content is retained and can be restored with
    /// [`enable`](Self::enable).
    pub fn disable(mut self, name: &str) -> Self {
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.enabled = false;
        }
        self
    }

    /// Re-enable a previously disabled section. No-op if the section is absent.
    pub fn enable(mut self, name: &str) -> Self {
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.enabled = true;
        }
        self
    }

    /// Remove a section entirely. No-op if the section is absent.
    pub fn remove(mut self, name: &str) -> Self {
        self.sections.retain(|s| s.name != name);
        self
    }

    /// Remove all sections, keeping the configured separator.
    pub fn clear(mut self) -> Self {
        self.sections.clear();
        self
    }

    /// Get the content of a section by name, if it exists.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.content.as_str())
    }

    /// Names of all sections (enabled and disabled) in render order.
    pub fn section_names(&self) -> Vec<&str> {
        let mut sorted = self.sections.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|s| std::cmp::Reverse(s.priority));
        sorted.iter().map(|s| s.name.as_str()).collect()
    }

    /// Names of only the enabled sections, in render order.
    pub fn enabled_names(&self) -> Vec<&str> {
        let mut sorted = self
            .sections
            .iter()
            .filter(|s| s.enabled)
            .collect::<Vec<_>>();
        sorted.sort_by_key(|s| std::cmp::Reverse(s.priority));
        sorted.iter().map(|s| s.name.as_str()).collect()
    }

    /// Total number of sections (enabled and disabled).
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Returns `true` if there are no sections at all.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Count of enabled sections.
    pub fn enabled_count(&self) -> usize {
        self.sections.iter().filter(|s| s.enabled).count()
    }

    /// Returns `true` if a section with `name` is present and enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.sections.iter().any(|s| s.name == name && s.enabled)
    }

    /// Render the prompt.
    ///
    /// Enabled sections are emitted in priority order (highest first), and
    /// sections that share a priority keep their relative insertion order
    /// (the sort is stable). Disabled sections are skipped, and an empty
    /// builder renders to an empty string.
    pub fn build(&self) -> String {
        let mut sorted: Vec<&Section> = self.sections.iter().filter(|s| s.enabled).collect();
        // Stable sort: higher priority first; equal priority keeps insertion order.
        sorted.sort_by_key(|s| std::cmp::Reverse(s.priority));
        sorted
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join(&self.separator)
    }

    /// Check whether a section with the given name exists (enabled or not).
    pub fn has(&self, name: &str) -> bool {
        self.sections.iter().any(|s| s.name == name)
    }

    /// Any duplicate section names.
    ///
    /// The fluent setters dedupe by name, so this returns an empty `Vec` in
    /// normal use. It exists as a defensive check for code that constructs a
    /// builder through other means.
    pub fn duplicate_names(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut dups = Vec::new();
        for s in &self.sections {
            if !seen.insert(&s.name) {
                dups.push(s.name.clone());
            }
        }
        dups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_section() {
        let p = ContextBuilder::new()
            .section("role", "You are helpful.")
            .build();
        assert_eq!(p, "You are helpful.");
    }

    #[test]
    fn two_sections_joined() {
        let p = ContextBuilder::new()
            .section("a", "First.")
            .section("b", "Second.")
            .build();
        assert!(p.contains("First."));
        assert!(p.contains("Second."));
    }

    #[test]
    fn default_separator_double_newline() {
        let p = ContextBuilder::new()
            .section("a", "A")
            .section("b", "B")
            .build();
        assert_eq!(p, "A\n\nB");
    }

    #[test]
    fn custom_separator() {
        let p = ContextBuilder::new()
            .separator(" | ")
            .section("a", "A")
            .section("b", "B")
            .build();
        assert_eq!(p, "A | B");
    }

    #[test]
    fn replace_existing_section() {
        let builder = ContextBuilder::new()
            .section("role", "Old")
            .section("role", "New");
        assert_eq!(builder.get("role"), Some("New"));
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn replace_preserves_priority() {
        // `section` must not reset a priority set earlier.
        let b = ContextBuilder::new()
            .section_with_priority("role", "Old", 5)
            .section("role", "New");
        let p = b.clone().section_with_priority("other", "Other", 0).build();
        // role has priority 5 so it renders before the priority-0 section.
        assert!(p.find("New").unwrap() < p.find("Other").unwrap());
        assert_eq!(b.get("role"), Some("New"));
    }

    #[test]
    fn disable_omits_section() {
        let p = ContextBuilder::new()
            .section("a", "Show")
            .section("b", "Hide")
            .disable("b")
            .build();
        assert!(p.contains("Show"));
        assert!(!p.contains("Hide"));
    }

    #[test]
    fn enable_restores_section() {
        let p = ContextBuilder::new()
            .section("a", "Content")
            .disable("a")
            .enable("a")
            .build();
        assert!(p.contains("Content"));
    }

    #[test]
    fn is_enabled_reports_state() {
        let b = ContextBuilder::new().section("a", "x").disable("a");
        assert!(!b.is_enabled("a"));
        assert!(!b.is_enabled("missing"));
        let b = b.enable("a");
        assert!(b.is_enabled("a"));
    }

    #[test]
    fn remove_deletes_section() {
        let b = ContextBuilder::new()
            .section("a", "keep")
            .section("b", "gone")
            .remove("b");
        assert!(!b.has("b"));
        assert!(b.has("a"));
    }

    #[test]
    fn clear_removes_everything() {
        let b = ContextBuilder::new()
            .section("a", "1")
            .section("b", "2")
            .clear();
        assert!(b.is_empty());
        assert_eq!(b.build(), "");
    }

    #[test]
    fn priority_ordering() {
        let p = ContextBuilder::new()
            .section_with_priority("low", "LOW", 1)
            .section_with_priority("high", "HIGH", 10)
            .build();
        let high_pos = p.find("HIGH").unwrap();
        let low_pos = p.find("LOW").unwrap();
        assert!(high_pos < low_pos);
    }

    #[test]
    fn set_priority_reorders() {
        let p = ContextBuilder::new()
            .section("a", "AAA")
            .section("b", "BBB")
            .set_priority("b", 100)
            .build();
        assert!(p.find("BBB").unwrap() < p.find("AAA").unwrap());
    }

    #[test]
    fn equal_priority_keeps_insertion_order() {
        let p = ContextBuilder::new()
            .section_with_priority("first", "1", 5)
            .section_with_priority("second", "2", 5)
            .section_with_priority("third", "3", 5)
            .build();
        assert_eq!(p, "1\n\n2\n\n3");
    }

    #[test]
    fn has_section() {
        let b = ContextBuilder::new().section("x", "v");
        assert!(b.has("x"));
        assert!(!b.has("y"));
    }

    #[test]
    fn section_names_listed_in_priority_order() {
        let b = ContextBuilder::new()
            .section_with_priority("a", "", 1)
            .section_with_priority("b", "", 9);
        assert_eq!(b.section_names(), vec!["b", "a"]);
    }

    #[test]
    fn enabled_names_excludes_disabled() {
        let b = ContextBuilder::new()
            .section("a", "")
            .section("b", "")
            .disable("a");
        assert_eq!(b.enabled_names(), vec!["b"]);
        assert_eq!(b.section_names().len(), 2);
    }

    #[test]
    fn enabled_count() {
        let b = ContextBuilder::new()
            .section("a", "")
            .section("b", "")
            .disable("b");
        assert_eq!(b.enabled_count(), 1);
    }

    #[test]
    fn len_and_is_empty() {
        let b = ContextBuilder::new();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
        let b = b.section("a", "x");
        assert!(!b.is_empty());
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn empty_builder_empty_string() {
        assert_eq!(ContextBuilder::new().build(), "");
    }

    #[test]
    fn all_disabled_builds_empty() {
        let p = ContextBuilder::new().section("a", "x").disable("a").build();
        assert_eq!(p, "");
    }

    #[test]
    fn no_duplicates_in_normal_use() {
        let b = ContextBuilder::new().section("a", "1").section("b", "2");
        assert!(b.duplicate_names().is_empty());
    }
}
