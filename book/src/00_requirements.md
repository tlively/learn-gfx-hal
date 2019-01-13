# Requirements

I assume that you have basic familiarity with Rust. So go read [The Rust
Book](https://doc.rust-lang.org/book/) if you haven't ever done that.

We will also be touching upon elements of `unsafe` Rust, and so you should also
read [The Rustonomicon](https://doc.rust-lang.org/nomicon/) if you have not.
Actually, that's a mild lie, most of `gfx-hal-0.1.0` doesn't define any of its
safety limits anyway (not beyond "whatever Vulkan say is okay"), so it's all a
shot in the dark no matter what you do. Even if you're using a backend that
isn't Vulkan.

I don't assume you have any prior graphics programming skills. I sure don't have
much myself. I drew a quad once in OpenGL, but that's it. We'll be learning and
reviewing all that stuff together.

The code all assumes that you're using **Rust 2018**.

I set rustfmt to have 2 space indents and a line limit of 130. I hope you're
fine with that.
