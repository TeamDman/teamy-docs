# Explore Rust interactively

Use [Evcxr](https://github.com/evcxr/evcxr/blob/6ac1c87dd37603cce2e054025bbbf416c33c41e2/README.md) to try Rust expressions in an interactive evaluation context. Its components include a terminal REPL (`evcxr_repl`), a Jupyter kernel (`evcxr_jupyter`) and their shared evaluation library. This is evaluation prior art, not a dependency adopted into the [Teamy CLI template](rust-cli-template.md). Review the matching release and its prerequisites before installing; this review did not install or run Evcxr.

For example, in an already installed REPL, enter the following as separate submissions:

```text
let values = [2, 4, 6];
values.iter().sum::<i32>()
```

The [common usage guide at `6ac1c87`](https://github.com/evcxr/evcxr/blob/6ac1c87dd37603cce2e054025bbbf416c33c41e2/COMMON.md) describes persisted variables, crate dependencies, local crate paths, async support and limitations. Persistent variables cannot reference other persistent variables, and a crashing or exiting evaluation process loses its variables. Dependency compilation can take time. The [REPL instructions](https://github.com/evcxr/evcxr/blob/6ac1c87dd37603cce2e054025bbbf416c33c41e2/evcxr_repl/README.md) list `rust-src` and a recent Rust toolchain among setup considerations. These examples and platform requirements were read from upstream documentation; they have not been tested here.

Evcxr evaluates Rust code with the process's authority: do not treat a REPL or notebook as a sandbox for untrusted snippets. Adding a crate may also execute build-time code. Preserve useful experiments as project code or tests when they need reproducibility.

## Embed a scripting language with Rhai

[Rhai](https://github.com/rhaiscript/rhai/blob/f5bd5b9e2743dbf1df077e48f6059fabbae4135c/README.md) is a separate, dynamically typed scripting language and engine implemented in Rust. A Rust host can register functions and types, supply values through a scope, compile a script to an AST for repeated evaluations, and choose which features to expose. This is an evaluation reference, not an adopted dependency or a replacement for the Teamy Rust CLI baseline.

For a small host-side experiment, a pure function can be exposed explicitly:

```rust
let mut engine = rhai::Engine::new();
engine.set_max_operations(1_000);
engine.register_fn("double", |value: i64| value * 2);
let result: i64 = engine.eval("double(21)")?;
assert_eq!(result, 42);
```

This is an illustrative fragment for a Rust function that propagates Rhai errors; it has not been run here. The host must declare the `rhai` dependency and select an appropriate pinned version and features. For an application, start with [teamy-rust-cli](rust-cli-template.md), then record the added engine and its policy.

Rhai's [operation limit](https://rhai.rs/book/safety/max-operations.html) is unlimited by default. A finite limit can stop scripts that exceed it, but an operation is not a wall-clock deadline: one registered Rust function may do substantial work. The `unchecked` feature disables safety checks including that limit. Audit exposed functions, module loading, data size, cancellation and side effects independently, and test denied actions and runaway scripts under the selected configuration. A restricted scripting API does not itself authorize an action; see [delegated authority](delegated-authority.md).

For rendering text from controlled data rather than evaluating interactive Rust or embedding an action language, see [Liquid templates](liquid-templates.md).
