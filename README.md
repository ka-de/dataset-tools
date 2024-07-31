# `dataset-tools`

## Features

---

### `check`

This versatile Rust program provides various checking and analysis functionalities for codebases and datasets to help maintain quality and consistency. It offers the following key features:

#### Attribute Scanning

Scans Rust files for built-in attributes, highlights them and gives you the whole list.

#### Multiline Detection

Find multiple lines in text files, this can cause issues with certain training tasks where multiple lines means that the captions are picked in a random order.

#### Optimization Verification

Analyzes `Cargo.toml` files for correct optimization settings.

#### Pedantic Warning Check

Ensure Rust files have the attribute set for pedantic warnings in `clippy`.

With more things to come, eventually!

## Release Build

---

```bash
cargo build --workspace -Z build-std --target x86_64-pc-windows-msvc --release
```

## Run `clippy` to Fix Warnings

---

```bash
cargo clippy --workspace --all-targets
```
