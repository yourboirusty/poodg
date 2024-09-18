# Poodg Game
This is a Rust based game using `embedded-graphics` in combination with `embedded-graphics-simulator` and `embedded-graphics-simulator-web` to provide a binary for Linux X86, WASM and an embedded RP2040 device.

WASM build is available to play [on Itch.io](https://the-rusto.itch.io/poodg)

# Native 
Depends on SDL2. Doesn't work on WSL, doesn't compile on my Windows.

1. `cargo run --target x86_64-unknown-linux-gnu`


# WASM
1. You'll need [Trunk](https://trunkrs.dev/)
- If you don't mind waiting a bit, `cargo install trunk`
- If you want it to be speedier, use `binstall`
- `cargo install cargo-binstall`
- `cargo binstall trunk`
2. `trunk serve --open`
