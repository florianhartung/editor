# Todos

- How important is async really? I'd prefer to not pull in the entire tokio ecosystem dependencies.
  - Read through https://rust-lang.github.io/async-book/
  - Need async in core for LSP
  - Some frontends may rely on async. e.g. Iced

# Comments on Swiboe

- RPC should have used names. To say that names make renaming RPCs harder is an excuse IMO. That only hides the problem of inter-plugin dependence and worsens the developer experience (DX) of plugin authors.
- The communication format should not be serialized by default due to the obvious performance penalty. Instead plugins can still all be implemented in Rust and linked into the same executable (which makes them less plugin-ny). For example to share buffer data between plugins, a Arc<Rope> may be used. However, this still should allow for external (Wasm) plugins to hook into this mechanism. Also the Rust source code for integrated plugins must always be compilable to Wasm as well (so that they could run as external plugins).
