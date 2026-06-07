# agent-context-builder

[![CI](https://github.com/MukundaKatta/agent-context-builder-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/agent-context-builder-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

Compose LLM system prompts from named sections, with ordering and conditional
inclusion.

`agent-context-builder` is a tiny, dependency-free Rust library for assembling
the system prompt of an LLM agent out of independently managed, **named**
sections. Sections can be added, replaced, reordered, toggled on/off, and
removed — and the final prompt is rendered deterministically in priority order.

It is handy when a prompt is built up from several concerns (role, rules,
style guide, tool descriptions, feature-flagged behavior, footer) that you
want to manage by name instead of by string concatenation.

## Why

Building a system prompt by hand usually means manually concatenating strings
and re-shuffling them whenever a section needs to move or be toggled. This
crate replaces that with a small fluent builder:

- **Named sections** — refer to a piece of the prompt by name, replace or
  remove it later without touching the rest.
- **Deterministic ordering** — give a section a priority to float it to the
  top or sink it to the bottom; equal-priority sections keep insertion order.
- **Conditional inclusion** — `disable`/`enable` a section to feature-flag
  parts of the prompt without losing their content.
- **Zero dependencies** — pure `std`, `#![forbid(unsafe_code)]`.

## Install

Add it to your `Cargo.toml`:

```toml
[dependencies]
agent-context-builder = "0.1"
```

Or with cargo:

```sh
cargo add agent-context-builder
```

## Usage

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    // High priority keeps the role at the very top.
    .section_with_priority("role", "You are a meticulous coding assistant.", 100)
    .section("style", "Be concise and prefer code over prose.")
    .section("rules", "Never invent APIs. Cite the file you changed.")
    // Negative priority sinks the footer to the bottom.
    .section_with_priority("footer", "Respond in GitHub-flavored Markdown.", -100)
    .build();

assert!(prompt.starts_with("You are a meticulous coding assistant."));
assert!(prompt.ends_with("Respond in GitHub-flavored Markdown."));
```

### Conditional (feature-flagged) sections

```rust
use agent_context_builder::ContextBuilder;

let debug = false;

let mut builder = ContextBuilder::new()
    .section("role", "You are a helpful assistant.")
    .section("debug", "Explain your reasoning step by step.");

if !debug {
    builder = builder.disable("debug");
}

let prompt = builder.build();
assert!(!prompt.contains("step by step"));
```

### Custom separator

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .separator("\n---\n")
    .section("a", "alpha")
    .section("b", "beta")
    .build();

assert_eq!(prompt, "alpha\n---\nbeta");
```

A runnable version of these snippets lives in
[`examples/system_prompt.rs`](examples/system_prompt.rs):

```sh
cargo run --example system_prompt          # prompt without the debug section
cargo run --example system_prompt -- --debug   # include the debug section
```

## API

All builder methods take and return `self` so calls can be chained; read-only
methods borrow `&self`.

### Construction & configuration

| Method | Description |
| --- | --- |
| `ContextBuilder::new()` | Empty builder with the default `"\n\n"` separator. |
| `separator(sep)` | Set the string placed between sections when rendering. |

### Adding & modifying sections

| Method | Description |
| --- | --- |
| `section(name, content)` | Add a section, or replace the content of an existing one with that name (priority preserved). |
| `section_with_priority(name, content, priority)` | Add or update a section with an explicit priority (higher renders earlier). |
| `set_priority(name, priority)` | Change the priority of an existing section. |
| `disable(name)` | Omit a section from `build` output while keeping its content. |
| `enable(name)` | Re-enable a previously disabled section. |
| `remove(name)` | Delete a section entirely. |
| `clear()` | Remove all sections (keeps the separator). |

### Inspecting

| Method | Returns | Description |
| --- | --- | --- |
| `get(name)` | `Option<&str>` | Content of a section, if present. |
| `has(name)` | `bool` | Whether a section exists (enabled or not). |
| `is_enabled(name)` | `bool` | Whether a section exists and is enabled. |
| `section_names()` | `Vec<&str>` | All section names in render order. |
| `enabled_names()` | `Vec<&str>` | Enabled section names in render order. |
| `len()` | `usize` | Total number of sections. |
| `is_empty()` | `bool` | Whether there are no sections. |
| `enabled_count()` | `usize` | Number of enabled sections. |
| `duplicate_names()` | `Vec<String>` | Any duplicate names (empty in normal use). |

### Rendering

| Method | Returns | Description |
| --- | --- | --- |
| `build()` | `String` | The rendered prompt: enabled sections in priority order (highest first), joined by the separator. Equal priorities keep insertion order. |

## Ordering rules

1. Disabled sections are skipped.
2. Remaining sections are sorted by priority, highest first.
3. Sections sharing a priority retain their insertion order (the sort is
   stable).

The default priority is `0`, so without any `*_priority` calls sections render
in the order you added them.

## Development

```sh
cargo build
cargo test            # unit + integration + doc tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## License

Licensed under the [MIT License](https://opensource.org/licenses/MIT).
