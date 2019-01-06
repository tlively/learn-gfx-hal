# INCOMPLETE

# TODO

# WORK IN PROGRESS

# Clearing The Window

Once you have a window open, the usual next step for a graphics tutorial is to
get you to draw "your first triangle". You see, the fundamental primitive of 3d
graphics is basically always the triangle. There are some systems, such as the
[Sega Saturn](https://en.wikipedia.org/wiki/Sega_Saturn), that use "quads"
instead, but in all the modern systems you'll find it's going to be triangles. A
triangle is the most flexible primitive to have, and with enough math and enough
parallel processing you can do anything you want. Skyrim? Triangles. Breath of
The Wild? Triangles. Super Smash Bros? Just a whole lot of triangles.

However, there's a great many steps of setup involved between "draws nothing"
and "draws one triangle". Our `hello_winit` example, with proper spacing and a
few comments and all that, comes out to about 45 lines. A `hello_triangle`
example, without even anything in the way of comments or much whitespace, is
about 470 lines. That's a fairly big pile of new material. Possibly too big. So,
yes, we will go over all of that. Yes, we'll draw a triangle soon enough.
However we're going to at least break it into two lessons.

Remember how our `winit` window didn't refresh itself properly? We can add
enough code to make it refresh itself to a designated clear color that we
specify. That's a good halfway point between where we are and where we're trying
to get to farther on.

## Adding In `gfx-hal` And A Backend

First, we have to add the `gfx-hal` crate to our `Cargo.toml` file. We also need
to pick a backend crate. Remember that the "hal" in `gfx-hal` is for "Hardware
Abstraction Layer". So `gfx-hal` just provides the general types and operations,
then each backend actually implements the details according to the hardware API
it's abstracting over.

Since we want it to be something you can pick per-compile, we're going to use a
big pile of features and optional dependencies:

```toml
[features]
default = []
metal = ["gfx-backend-metal"]
dx12 = ["gfx-backend-dx12"]
vulkan = ["gfx-backend-vulkan"]

[dependencies]
winit = "0.18"
gfx-hal = "0.1"

[dependencies.gfx-backend-vulkan]
version = "0.1"
optional = true

[target.'cfg(target_os = "macos")'.dependencies.gfx-backend-metal]
version = "0.1"
optional = true

[target.'cfg(windows)'.dependencies.gfx-backend-dx12]
version = "0.1"
optional = true
```

If you want RLS to play nice with the various optional features you must tell it
which one to use for its compilations. If you're using VS Code with the RLS
plugin, instead of messing up your `Cargo.toml` by specifying a default feature
you can instead make a `.vscode/settings.json` file in your project folder and
then place a setting for the feature you want it to use for RLS runs. Something
like this:

```json
{
  "rust.features": [
    "dx12"
  ]
}
```

If you're using RLS with some editor besides VS Code I'm afraid I don't know the
details of how you tell it to use a particular feature, but you probably can.
Consult your plugin docs, and such.

Over inside our main file we put some conditional stuff at the top:

```rust
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
```

Yes, in the 2018 edition it's not _strictly necessary_ to have `extern crate`
any more, but this way we can alias whichever backend we pick to just be `back`.

Finally, before we go on, I'll mention that there _are_ other backend options
that we haven't considered:

* [gfx-backend-empty](https://crates.io/crates/gfx-backend-empty) does nothing
  but provide the required implementations as empty structs and do-nothing
  methods and so on. It's mostly used in the rustdoc examples for `gfx-hal`, and
  you might also use this with RLS or something, but you can't actually draw a
  picture or compute anything with it.
* [gfx-backend-gl](https://crates.io/crates/gfx-backend-gl) lets you target
  OpenGL 2.1+ and OpenGL ES2+. You'd probably use this if you wanted to run in a
  webpage, or perhaps on a Raspberry Pi (which has OpenGL ES2 drivers, but not
  Vulkan), or something like that where you couldn't pick one of the main
  options. Unfortunately, the GL backend is actually a little busted at the
  moment. The biggest snag is that webpages and desktop apps have rather
  different control flow, so it's hard to come up with a unified API. Work is
  being done, and hopefully soon I'll be able to recommend the GL backend.

## Allow For Logging

Since we're already mucking about with extra dependencies and stuff we'll also
take the time to add logging ability to our program.

In Rust you use the [log](https://docs.rs/log) crate as the generic logging
facade. It provides macros for each log level and you call them just like you'd
call `println!`. Then a particular logging backend (some other crate) picks up
those logging calls and does the actual logging into a file or over the network
or however. The simplest logging backend to use is probably
[env_logger](https://docs.rs/env_logger) since it just spits things to `stdout`
and `stderr` instead of needing to setup log files. That's fine for a tutorial,
so we'll do that. We just add a bit more to our `Cargo.toml`:

```toml
[dependencies]
log = "0.4.0"
env_logger = "0.5.12"
winit = "0.18"
gfx-hal = "0.1"
```

And then we turn on the `env_logger` in main before we do anything else:

```rust
fn main() {
  env_logger::init();
  // ...
```

And we'll see anything that someone wanted to log. If we want to do our own
logging that's easy too:

```rust
#[macro_use]
extern crate log;
```

Again, we could set it up with `use` statements, but `#[macro_use]` just grabs
out all the macros from the `log` crate without any extra fuss. That's all we
need to emit basic logging messages.

## Create A HalState Struct

Okay, okay, we've got all out initial dependencies in place, time to get back to
code. What do we do first?

Well, the `winit` crate gave us just two things to keep together and that
already called for a struct to help organization. Spoilers: `gfx-hal` is gonna
give us _way_ more than two things to keep track of.

```rust
#[derive(Debug)]
pub struct HalState {
  // TODO
}
```

We definitely want to have a construction method that will hide away _as much_ of
the initialization as we can.

```rust
impl HalState {
  pub fn new() -> Self {
    unimplemented!()
  }
}
```

## Create An Instance

The very first thing we do in our `HalState::new` method is create an
[Instance](https://docs.rs/gfx-hal/0.1.0/gfx_hal/trait.Instance.html). This does
whatever minimal things are required to activate your selected backend API. It's
quite simple. Every backend provides a _type_ called `Instance` that also
implements the `Instance` _trait_. The types, by convention, have a method
called `create` which you pass a `&str` (the name for your instance) and `u32`
(the version for your instance). Don't forget that `create` isn't part of the
Instance trait, it's just a convention for now. In future versions of `gfx-hal`
it might become more formalized.

```rust
pub struct HalState {
  instance: back::Instance,
}
impl HalState {
  pub fn new() -> Self {
    let instance = back::Instance::create(WINDOW_NAME, 1);

    Self { instance }
  }
}
```

As you can see, we add a field in the struct definition, and then in `new` we
create that value. At the end of `new` we pack up all the stuff we've created.
Right now it's just one thing but we'll have about 20 things by the end of this.
After this first one I won't show the whole struct and new method each time,
we'll just follow the same pattern over and over:

* Add a field to the struct
* Generate a value of that type
* Put that value into the struct we return at the bottom of `new`

This pattern is really obvious, but it'll get us pretty far.

Unfortunately, we can no longer `derive(Debug)` on our struct, since the
`Instance` type doesn't have `Debug`. That's a little sad, but we'll live
through it.

