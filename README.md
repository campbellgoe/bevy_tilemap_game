# Bevy Tilemap Game

Is some code the AI wrote and I eventually got working. It procedurally generates (perlin noise) tiles.

# Problem (invalid surface) when trying to display a window

[invalid surface error](https://www.massless.ltd/blog/rust-bevy-invalid-surface-error-and-how-how-to-set-wgpu-backend)

For me, in order to get the window to render, I had to run this command:
`export WGPU_BACKEND=VULKAN`

before compiling

`cargo run`

# Methodology

I'm currently learning Rust as someone who comes from JavaScript/TypeScript.

I'm using AI to help me on things I can't find answers for on Google.

This means it uses outdated packages and documentation for its generated code.

I have to then figure out how to make the code work.

That's my methodology.
