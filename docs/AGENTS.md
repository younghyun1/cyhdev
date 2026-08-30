For Rust projects:

Use of `unwrap()` and `expect()` is not tolerable. All Option<T> and Result<T, E> types must be dealt explicitly through match statements. Generated code will not include any pathways to explicit panics; all errors will be logged and a graceful exit from the current context will be executed.

In projects using `tracing`, info, warning, and error messages must be logged in a structured manner in this way:             
```rust
error!(env_key = key, error = %e, "Missing required environment variable");
```

Performance is key to Rust. When using `diesel`, exclude N+1 errors.

Run `cargo fmt`, `cargo check`, and `cargo clippy` after finishing large refactorings and fix generated warnings and errors.

Use of Linux intrinsics using `libc` and such are encouraged when more performant. We develop for Linux and macOS only.

For SolidJS/TypeScript projects:
No usage of 'any'. Make the most stringest use of TypeScript's type safety system as well as composable data structures (DTOs).