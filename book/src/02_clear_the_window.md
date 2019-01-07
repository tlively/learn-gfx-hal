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

## Be Clear About Our Goal

[Always write the usage code first.](https://caseymuratori.com/blog_0024) Always.

Before we do anything new we're going to just write an outline for how we think
our code _should_ work. We don't know the limits of our tools right now, but that
actually doesn't matter too much. If we need to change, we can change later.

We want to start with an idea that's _easy to call_, because in the long term
we'll be calling any bit of code a lot more than we'll be writing it.

We already wrapped up the `winit` stuff into a state blob, so we'll assume that
soon enough we'll also want to wrap `gfx-hal` into a state blob. Also, let's
just assume that having a logging system active is a good idea ahead of time.
There's already been a local variable apart from our two main state blobs, and
there might be more. If we have enough local vars that aren't part of `winit` or
`gfx-hal` then that will eventually become a state blob too.

Once everything is in place, we go to a primary loop:

1) Gathers user input for this frame
2) Processes the effects of the input
3) Draws the new frame
4) Waits for Vertical Sync before looping

Once the main loop ends for whatever reason we can try to shut down things as
gracefully as possible. Actually, it's in some sense pointless to do shutdown
code, since the OS will clean up any resources for you when the process exits.
However, it's important to know how to do the shut down in case a scenario comes
up where you want to close down `gfx-hal` entirely without leaving the process.
Maybe you want to re-initialize on another GPU or something like that. We want
to go over the how at least once, even if we let the OS do all the cleanup for
us in future lessons.

Everything so far sounds simple enough, let's look at that in code form:

```rust
fn main() {
  // START LOGGING
  // ???

  // START WINIT
  let mut winit_state = WinitState::default();

  // START GFX HAL
  let mut hal_state = HalState::new(???);

  // CREATE LOCAL VARIABLES
  let mut running = true;
  // ???

  'main_loop: loop {
    winit_state.events_loop.poll_events(|event| match event {
      // HANDLE EVENTS HERE
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => running = false,
      // ???
    });
    if !running {
      break 'main_loop;
    }

    // PROCESS THE EVENT CHANGES
    // ???

    // DRAW THE FRAME
    if let Err(e) = hal_state.draw_clear_frame(color) {
      error!("Error while drawing a clear frame: {}", e);
      break 'main_loop;
    }
  }

  // SHUT DOWN AS GRACEFULLY AS WE CAN
  if let Err(e) = hal_state.shutdown_gracefully() {
    error!("Error while shutting down: {}", e);
  }
}
```

## Allow For Logging

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

We could import each macro individually with a `use` statement, but
`#[macro_use]` just grabs out all the macros from the `log` crate without any
extra fuss. That's all we need to emit basic logging messages.

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
log = "0.4.0"
env_logger = "0.5.12"
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

## Create A HalState Struct

Okay, okay, we've got all out initial dependencies in place, time to get back to
code.

First we declare that struct that's going to hold all of our `gfx-hal` state.

```rust
#[derive(Debug)]
pub struct HalState {
  // TODO
}
```

We definitely want to have a construction method that will hide away _as much_ of
the initialization as we can, because there's going to be piles of it.

```rust
impl HalState {
  pub fn new() -> Self {
    unimplemented!()
  }
}
```

## TODO
---
---
Figure out how to 