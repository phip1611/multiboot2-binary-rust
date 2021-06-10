A Rust/Cargo workspace doesn't work with Rustc/cargo 1.54 nightly, because
each member needs a custom target and this is not yet officially supported.

The name for the unstable feature is "per-package-target".

- https://github.com/rust-lang/cargo/pull/9030/
- https://github.com/rust-lang/cargo/issues/9451


Therefore two standalone Rust projects!
