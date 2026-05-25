/*!
agent-context-builder: compose LLM system prompts from named sections.

Build a system prompt from independently managed named sections. Sections
can be added, removed, reordered, and conditionally included. The final
prompt is rendered in insertion or explicit priority order.

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .section("role", "You are a helpful assistant.")
    .section("rules", "Always cite sources.")
    .build();
assert!(prompt.contains("helpful assistant"));
```
*/

use std::collections::HashSet;

#[derive(Debug, Clone)]
struct Section {
    name: String,
    content: String,
    priority: i32,
    enabled: bool,
}

/// Builds a system prompt from named, ordered sections.
#[derive(Debug, Default)]
pub struct ContextBuilder {
    sections: Vec<Section>,
    separator: String,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self { sections: Vec::new(), separator: "\n\n".into() }
    }

    /// Set the separator between sections (default: double newline).
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Add a named section. If a section with this name exists, replace it.
    pub fn section(mut self, name: impl Into<String>, content: impl Into<String>) -> Self {
        let name = name.into();
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.content = content.into();
        } else {
            self.sections.push(Section { name, content: content.into(), priority: 0, enabled: true });
        }
        self
    }

    /// Add a section with explicit priority (higher = earlier in output).
    pub fn section_with_priority(mut self, name: impl Into<String>, content: impl Into<String>, priority: i32) -> Self {
        let name = name.into();
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.content = content.into();
            s.priority = priority;
        } else {
            self.sections.push(Section { name, content: content.into(), priority, enabled: true });
        }
        self
    }

    /// Disable a section by name (it will be omitted from build output).
    pub fn disable(mut self, name: &str) -> Self {
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.enabled = false;
        }
        self
    }

    /// Re-enable a previously disabled section.
    pub fn enable(mut self, name: &str) -> Self {
        if let Some(s) = self.sections.iter_mut().find(|s| s.name == name) {
            s.enabled = true;
        }
        self
    }

    /// Remove a section entirely.
    pub fn remove(mut self, name: &str) -> Self {
        self.sections.retain(|s| s.name != name);
        self
    }

    /// Get content of a section by name.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.sections.iter().find(|s| s.name == name).map(|s| s.content.as_str())
    }

    /// Names of all sections (in render order, enabled and disabled).
    pub fn section_names(&self) -> Vec<&str> {
        let mut sorted = self.sections.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted.iter().map(|s| s.name.as_str()).collect()
    }

    /// Count of enabled sections.
    pub fn enabled_count(&self) -> usize {
        self.sections.iter().filter(|s| s.enabled).count()
    }

    /// Render the prompt: enabled sections sorted by priority desc, then insertion order.
    pub fn build(&self) -> String {
        let mut sorted: Vec<&Section> = self.sections.iter().filter(|s| s.enabled).collect();
        // Stable sort: higher priority first; equal priority keeps insertion order.
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted.iter().map(|s| s.content.as_str()).collect::<Vec<_>>().join(&self.separator)
    }

    /// Check if a section with the given name exists.
    pub fn has(&self, name: &str) -> bool {
        self.sections.iter().any(|s| s.name == name)
    }

    /// All duplicate section names (should be empty in normal use).
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
        let p = ContextBuilder::new().section("role", "You are helpful.").build();
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
        assert!(p.contains("\n\nB") || p.contains("A\n\n"), "separator missing");
    }

    #[test]
    fn custom_separator() {
        let p = ContextBuilder::new()
            .separator(" | ")
            .section("a", "A")
            .section("b", "B")
            .build();
        assert!(p.contains(" | "));
    }

    #[test]
    fn replace_existing_section() {
        let builder = ContextBuilder::new()
            .section("role", "Old")
            .section("role", "New");
        assert_eq!(builder.get("role"), Some("New"));
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
    fn remove_deletes_section() {
        let b = ContextBuilder::new()
            .section("a", "keep")
            .section("b", "gone")
            .remove("b");
        assert!(!b.has("b"));
        assert!(b.has("a"));
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
    fn has_section() {
        let b = ContextBuilder::new().section("x", "v");
        assert!(b.has("x"));
        assert!(!b.has("y"));
    }

    #[test]
    fn section_names_listed() {
        let b = ContextBuilder::new().section("a", "").section("b", "");
        let names = b.section_names();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
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
    fn empty_builder_empty_string() {
        assert_eq!(ContextBuilder::new().build(), "");
    }

    #[test]
    fn no_duplicates_in_normal_use() {
        let b = ContextBuilder::new().section("a", "1").section("b", "2");
        assert!(b.duplicate_names().is_empty());
    }
}
