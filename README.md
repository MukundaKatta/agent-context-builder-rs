# agent-context-builder

Compose LLM system prompts from named sections with ordering and conditional inclusion.

`agent-context-builder` is a small, dependency-free Rust library for assembling
the system prompt of an LLM agent out of independently managed, named sections.
Each section can be added, replaced, reordered by priority, disabled, re-enabled,
or removed. The final prompt is rendered by joining the enabled sections — sorted
by priority (highest first) and otherwise in insertion order — with a configurable
separator.

## Why

Agent system prompts tend to grow into one large, hard-to-edit string. This crate
lets you treat the prompt as a set of addressable pieces — `role`, `rules`,
`tools`, `context`, and so on — so you can toggle or reorder them at runtime
without rewriting the whole thing.

## Features

- Named sections with replace-on-duplicate semantics.
- Explicit priority ordering (higher priority renders earlier).
- Enable / disable sections without removing them.
- Configurable separator between sections (defaults to a blank line).
- Introspection helpers: `get`, `has`, `section_names`, `enabled_count`,
  `duplicate_names`.
- No external dependencies.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
agent-context-builder = "0.1"
```

## Usage

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .section("role", "You are a helpful assistant.")
    .section("rules", "Always cite sources.")
    .build();

assert!(prompt.contains("helpful assistant"));
```

### Priority ordering

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .section_with_priority("low", "Background notes.", 1)
    .section_with_priority("high", "Critical instructions.", 10)
    .build();

// The higher-priority section is rendered first.
assert!(prompt.find("Critical").unwrap() < prompt.find("Background").unwrap());
```

### Conditional inclusion

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .section("role", "You are a helpful assistant.")
    .section("debug", "Verbose debugging guidance.")
    .disable("debug") // omitted from build() output
    .build();

assert!(!prompt.contains("Verbose"));
```

### Custom separator

```rust
use agent_context_builder::ContextBuilder;

let prompt = ContextBuilder::new()
    .separator(" | ")
    .section("a", "A")
    .section("b", "B")
    .build();

assert!(prompt.contains(" | "));
```

## API overview

| Method | Description |
| --- | --- |
| `new()` | Create an empty builder (separator defaults to `"\n\n"`). |
| `separator(sep)` | Set the separator placed between sections. |
| `section(name, content)` | Add a section, or replace its content if the name exists. |
| `section_with_priority(name, content, priority)` | Add or update a section with an explicit priority. |
| `disable(name)` / `enable(name)` | Toggle whether a section is rendered. |
| `remove(name)` | Delete a section entirely. |
| `get(name)` | Get a section's content, if present. |
| `has(name)` | Check whether a section exists. |
| `section_names()` | List section names in render order. |
| `enabled_count()` | Number of currently enabled sections. |
| `duplicate_names()` | Any duplicate section names (empty in normal use). |
| `build()` | Render the final prompt string. |

## Building and testing

```sh
cargo build
cargo test
```

## Tech stack

- Language: Rust (edition 2021)
- Dependencies: none (standard library only)

## License

Licensed under the MIT License. See the `license` field in `Cargo.toml`.
