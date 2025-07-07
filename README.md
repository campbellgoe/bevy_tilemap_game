# Bevy Tilemap Game

# getting started

Make sure you have [rust 2024](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

`cargo run`

wait for it to build once then it should open a window and show the bevy app running.

# notes
Is some code the AI wrote and I eventually got working. It procedurally generates (perlin noise) tiles.

## Problem (invalid surface) when trying to display a window

[invalid surface error](https://www.massless.ltd/blog/rust-bevy-invalid-surface-error-and-how-how-to-set-wgpu-backend)

For me, in order to get the window to render, I had to run this command:
`export WGPU_BACKEND=VULKAN`

before compiling then when ready to run the app:

`cargo run`

## WASM and rendering in a HTML5 canvas

I plan to make a version which runs on the web.

# Methodology

I'm currently learning Rust as someone who comes from JavaScript/TypeScript.

I'm using AI to help me on things I can't find answers for on Google.

This means it uses outdated packages and documentation for its generated code.

I have to then figure out how to make the code work.

I'm not a vibe coder I'm a real coder but ai is addictive.

---

This game is built in rust/bevy