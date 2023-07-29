# Juodas Calc

State machine based calculator without usage of any expression parsers. It's written in Rust using [iced](https://github.com/iced-rs/iced) GUI library.

![](https://github.com/osennij-morok/juodas-calc/blob/master/for-readme/juodas-calc-demo1.gif)

# Requirements

[Cargo](https://github.com/rust-lang/cargo) package manager. I recommend you to install it via [rustup](https://rustup.rs).

# Compilation

```bash
cargo build --release
```

The application executable will be stored in `./target/release/` directory.

# Motivation

Why did I choose to write such a calculator without usage of any parsers? Cause it's much more challenging! Any dummy can write a calculator on top of parsers. Also Rust itself makes the task more complicated and interesting.
