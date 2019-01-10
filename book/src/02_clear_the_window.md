
# Clearing The Window

Once you have a window open, the _usual_ next step for a graphics tutorial is to
get you to draw "your first triangle". You see, the fundamental primitive of 3d
graphics is basically always the triangle. Yes, there are some systems such as
the [Sega Saturn](https://en.wikipedia.org/wiki/Sega_Saturn), that use quads
instead, but in all the modern systems you'll find it's going to be triangles. A
triangle is the most flexible primitive to have (even a quad is just two
triangles, when you think about it), and with enough math and enough parallel
processing you can do anything you want with triangles.

Skyrim? Triangles.

Breath of The Wild? Has a few more Triangles than Skyrim.

Super Smash Bros? Just a whole lot of triangles.

We'll be covering triangles _quite_ a bit. However, in the context of `gfx-hal`,
which is like 97% just "whatever Vulkan does", even if you're _not_ using the
Vulkan backend, there's a great _many_ steps of setup involved between "a window
that draws nothing" and "a window that draws one triangle".

In fact the [official gfx-hal docs](https://docs.rs/gfx-hal/0.1.0/gfx_hal/)
specifically give us a warning about this. They're so short I can include all
three sentences right here for dramatic effect:

> Low-level graphics abstraction for Rust. Mostly operates on data, not types.
> Designed for use by libraries and higher-level abstractions only.

There are basically no defaults provided. We have to list out every single
little step of the entire configuration process. I mean they convert C types
into Rust types for us, but it's still very "do it yourself". That's _cool_ if
you actually care about defining it all (which you will some day, I'm sure, or
you wouldn't be reading this right now), but it's also _sad_ when you're
starting out and want to get something on the screen.

Let's get to it! Since going all the way from nothing "drawing a triangle" will
probably be too much for a single "lesson" sized unit, we'll stop this lesson at
an intermediate step. Remember how our `winit` window from last lesson didn't
refresh itself properly? We can fix just that much and stop there. That _alone_
will cover a surprising amount of ground.

## Outline Our Target API

So, in the first lesson we had `WinitState` and it just had two public fields.
There's not much there that can be screwed up, so that's fine. I mean I guess
you could pair up the wrong `EventLoop` with the wrong `Window` or something,
but really there's so little there it's not worth worrying about the design on
that front. Just two public fields, and people can figure out the rest on their
own.

With `gfx-hal` it is _wildly_ the opposite situation. We're going to be juggling
a dozen or more things at once, and most of them are **very** unsafe things that
must be handled with extreme care. `gfx-hal`, at its core, is about directing a
pile of DMA units and a hyper-SIMD co-processor with all safety checks left in
"up to you" mode. That's _about_ as unsafe as it gets. Not only do we want a
`HalState` type, we want to expose _nothing_ that's inside of it, because it's
all a giant pile of sharp and dangerous things. We want to wrap all that up,
then offer a very small, well curated, semantically meaningful set of operations
that the outside world can access.

Sure sounds like API Design. There's so much that could be said about API
design.

* Let's keep it short: Always, _Always_, **_Always_** [write the usage code
  first.](https://caseymuratori.com/blog_0025)

Even before we know _anything about how `gfx-hal` works_, we're going to just
write how we _think_ we should be able to use it. How we think it's be easiest
to use. We'll be calling the methods a lot more than we'll be implementing the
methods, so unless we end up with some sort of performance disaster or
impossible requirement we'll keep the exterior simple even if it means the
interior might end up a little more complex.

So what's our _usage_ of the `HalState` type look like?

There's lots of answers you could have to that question. Really, there are.
Obviously since I'm writing this we're going to be using what I came up with,
but if you think you can get a better solution you should try it out. I'll try
to explain my thinking as best as I can.

### Initialization

We have `WinitState`, we're going to want `HalState` too. Clearly the
`WinitState` can be made before the `HalState` (since we did last lesson). We'll
also have a `LocalState`, and that's the grab bag of everything else, and maybe
we make it before or after the other two things. If you're doing a game or a
simulation or something that's your `GameState` or `World` or whatever you wanna
call the type. So far the code looks like this:

```rust
fn main(){
  let mut winit_state = WinitState::default();
  let mut hal_state = HalState::default();
  let mut local_state = LocalState::default();
  // MAIN LOOP
  // CLEANUP
}
```

Except, when you think about it, the way that `gfx-hal` initializes itself
~~probably~~ definitely depends on the `Window` it's going to draw within. So we
need a `HalState` initialization method that takes a `Window` reference. The
default name for any initialization method in Rust is just `new`, and I can't
think of a better name to use, so we'll go with that.

```rust
fn main(){
  let mut winit_state = WinitState::default();
  let mut hal_state = HalState::new(&winit_state.window);
  let mut local_state = LocalState::default();
  // MAIN LOOP
  // CLEANUP
}
```

Also, of course, our local variables might depend on all sorts of things in some
sort of application specific way. That part is up to you.

### Main Loop

Once things are all initialized and ready we go into the "main loop" part of the
program.

**Digression:** Video is really just a series of still pictures. You show one after
the other, very quickly, and a human brain interprets the existence of movement
where none "really" exists. Each picture is a "frame", and how quickly you go
from one frame to the next is the "frames per second" (fps). The minimum fps for
apparent movement is actually quite modest, you only need [about
12](https://en.wikipedia.org/wiki/Frame_rate#Human_vision). More is better of
course, the movement appears smoother the more fps you have. People have been
animating for a long time and there's all sorts of standards by now, but on a
computer you're usually expected to be drawing at about 60fps for "good" quality
animation and 30fps for "I guess that's okay for something made in Unity"
quality animation.

**Back to code:** The implication here is that each pass through our main loop
will be one frame of display. We gather up the input for that frame, adjust our
local variables according to the input (eg: in a game you move the player a tiny
bit, or whatever), and then render the new state of the world into a frame that
gets shown to the user. Something like this:

```rust
fn main(){
  let winit_state = WinitState::default();
  let hal_state = HalState::new(&winit_state.window);
  let mut local_state = LocalState::default();
  loop {
    let inputs = UserInput::poll_events_loop(&mut winit_state.event_loop);
    if inputs.end_requested {
      break;
    }
    local_state.update_from_input(inputs);
    do_the_render(&mut hal_state, &local_state);
  }
  // CLEANUP
}
```

Most of this is fairly obvious based on how the code of the last worked out with
the event loop and all.

You may be wondering why the `do_the_render` function is taking a `&mut
HalState` as the first argument, instead of having it be a `&mut self` method on
the `HalState` type. Well, I'm not sure it's the perfect decision, but we're
going to _try_ and keep our `HalState` and `LocalState` as totally separate as
we can.

* If `HalState` doesn't know anything about the `LocalState` then it's a lot
  more likely to focus on reusable drawing operations, and we'll be a lot more
  likely to have something we can reuse in future situations.
* Similarly, if `LocalState` doesn't know about `HalState` then it's easier to
  focus on the "business logic" of the program going through its state changes
  without worrying about anything else. We could even run the `LocalState`
  _without graphics at all_ (sometimes called a "headless" mode), which can be
  very nice if you need to "fast forward" a simulation, or run tests on your CI
  server.

It can often be _tempting_ to make everything into a method on some type, but
that's an urge we need to resist in this situation.

### What Does `do_the_render` Do?

I cheated a bit there, because I wrote down a call to `do_the_render` without
actually saying _what_ it's doing on the inside. That's the part we care about
the most! That's how we know what our `HalState` API needs to look like.

For this lesson, all we do is clear the screen. That sounds simple enough. Later
lessons will add more, of course, but this is our starting point.

```rust
pub fn do_the_render(hal: &mut HalState, locals: &LocalState) {
  hal.draw_clear_frame(locals.color());
}
```

Hmm, but there might be some sort of error that happens during rendering. Well,
nothing in `do_the_render` particularly knows about how to handle an error, and
that should probably be reported to the person that called `do_the_render` in
case they want to stop if there's an error, so we'll just pass that back up the
stack.

```rust
pub fn do_the_render(hal: &mut HalState, locals: &LocalState) -> Result<(), RenderError> {
  hal.draw_clear_frame(locals.color())
}
```

And then in `main` I guess we can just... log the error and quit. It's not ideal
for the program to shut itself down unexpectedly, but we don't really have a
backup strategy at the moment. In a more advanced situation the error might be
from the user trying to switch graphics settings or something, so you could
automatically switch back to the previous settings maybe. Depends on the
program, and the error.

Anyway, now things look more like this:

```rust
fn main(){
  let winit_state = WinitState::default();
  let hal_state = HalState::new(&winit_state.window);
  let mut local_state = LocalState::default();
  loop {
    let inputs = UserInput::poll_events_loop(&mut winit_state.event_loop);
    if inputs.end_requested {
      break;
    }
    local_state.update_from_input(inputs);
    if let Err(e) = do_the_render(&mut hal_state, &local_state) {
      error!("Rendering Error: {:?}", e);
      break;
    }
  }
  // CLEANUP
}
```

### Cleanup

Usually when working with "foreign" data, anything that comes from outside of
Rust, you have to consider the possibility that you'll have to manually do some
cleanup work. `gfx-hal` is not only normal in this regard, but it leans _hard_
into manual destruction. We'll have to clean up all of our components in an
extremely controlled manner or it'll cause segfaults on some backends.

How do we expose this in our API?

We _don't_.

I'm not saying that we _ignore_ the subject of cleanup, that would be foolish,
but I am saying that we should keep all of it entirely within the `HalState`
type. Things are smoothest for the user when they can just let a type drop away
without a care, and we're going to try and allow for such an easy use
experience. Mostly what this means is that we won't want to have any "getter"
methods that let an outside user move out anything that needs to be manually
destroyed later. If they want to check the value of a number or something that's
totally fine, but anything that needs to be explicitly cleaned up we can't let
out of our control.

Now we can see our final outline:

```rust
fn main(){
  let winit_state = WinitState::default();
  let hal_state = HalState::new(&winit_state.window);
  let mut local_state = LocalState::default();
  loop {
    let inputs = UserInput::poll_events_loop(&mut winit_state.event_loop);
    if inputs.end_requested {
      break;
    }
    local_state.update_from_input(inputs);
    if let Err(e) = do_the_render(&mut hal_state, &local_state) {
      error!("Rendering Error: {:?}", e);
      break;
    }
  }
}
```

Will we achieve this? Hard to say without trying.

## Activate Logging Powers

As you write for `gfx-hal`, you'll definitely write stuff that's wrong. That's
just how it goes, no shame in it. There's so many rules and details that even
the `gfx-rs` team members don't know all of it all the time. They look at the
Vulkan spec to verify the rules just like anyone else has to. Thankfully, we can
avoid having too many bugs quietly creep into things by logging what's going on
inside the program and hopefully something will show up in the logs to explain
the problem when there is a problem.

### The `log` Crate

If you've ever done logging before you know that usually there's a "logging
facade" which defines a way to write log messages that libraries use, and then
there's an actual logging implementation that a binary will activate at the
start of a process to receive logging messages and deal with them. Rust is no
different.

You use the [log](https://docs.rs/log) crate to write a logging message. You use
[a logging implementation of
choice](https://docs.rs/log/0.4.6/log/#available-logging-implementations) to
actually process those logging messages. The actual macros for logging are just
like how `println!` works, but instead of being called `println!` there's one
macro for each "level" of logging. From most important to least important it
goes: `error!`, `warn!`, `info!`, `debug!` and `trace!`. Different logging
implementations let you limit the levels that actually get logged, and the
logging crate has features to restrict what logging messages even get compiled
in (so you can compile out all logging in release mode or whatever). It's a
whole huge thing you can really dig through if you want.

I don't want to. I want to not have any fuss. So we'll use
[simple_logger](https://crates.io/crates/simple_logger) which is exactly as easy
as it sounds. You write one line, once, and then logging messages just go to
`stdout` or `stderr`.

First we add things to our `Cargo.toml` file.

```toml
[dependencies]
log = "0.4.0"
simple_logger = "1.0"
winit = "0.18"
```

And then we turn on the `simple_logger` in main before we do anything else:

```rust
fn main() {
  simple_logger::init().unwrap();
  // ...
```

And now we'll see anything that someone wanted to log. If we want to do our own
logging that's easy too:

```rust
#[allow(unused_imports)]
use log::{error, warn, info, debug, trace};
```

### LunarG Vulkan SDK

Next you'll also want some tools that _aren't_ strictly Rust related (shocking,
I know).

The [LunarG Vulkan SDK](https://vulkan.lunarg.com/sdk/home) is a free set of
tools for all major operating systems. Once you install the SDK, if you're using
the `gfx-backend-vulkan` crate as your `gfx-hal` backend it'll log any
validation errors when `debug_assertions` are on. You don't need to do any
special setup, it just conveniently happens for you.

Unfortunately, when testing with other backends you're much more "on your own",
but some help is still better than zero help.

## Adding In `gfx-hal` And A Backend

Adding `gfx-hal` to our `Cargo.toml` file comes in two parts. There's `gfx-hal`,
and also we need an actual "backend" that provides a specific implementation of
the types and operations that `gfx-hal` defines.

### Configuring Cargo

We want to keep the backend selection as easy to swap as possible. Normally this
is done at compile time, since there's only about one good backend per OS
anyway, and it keeps things simpler than trying to select a backend at startup.
The standard idiom for how to do this looks something like:

```toml
[features]
default = []
metal = ["gfx-backend-metal"]
dx12 = ["gfx-backend-dx12"]
vulkan = ["gfx-backend-vulkan"]

[dependencies]
log = "0.4.0"
simple_logger = "1.0"
winit = "0.18"
gfx-hal = "0.1"
arrayvec = "0.4"

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

If you want the Rust Language Server (RLS) to play nice with the various
optional features you must tell it which one to use for its compilations. You
could specify a default feature, but that's not quite elegant. If you're using
VS Code with the RLS plugin you can instead make a `.vscode/settings.json` file
in your project folder, and then in there place a setting for the feature you
want it to use for RLS runs. Something like this:

```json
{
  "rust.features": [
    "dx12"
  ]
}
```

If you're using RLS with some editor besides VS Code I'm afraid I don't know the
details of how you tell RLS to use a particular feature, but you probably can.
Consult your plugin docs, and such.

### Configuring The Code

Over inside our main file we won't actually be importing too much from the
backends, but we'll place some conditional `use` statements so that they're
always aliased to the same name, regardless of what one we're using.

```rust
#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;
```

### Any Other Backend Options?

There _are_ other backend options that we haven't considered:

* [gfx-backend-empty](https://crates.io/crates/gfx-backend-empty) does nothing
  but provide the required implementations as empty structs and do-nothing
  methods and similar. It's mostly used in the rustdoc examples for `gfx-hal`,
  so that they can check that doctests compile properly. You might also use this
  with RLS I guess, but since you'll also need a real backend compiled to run
  any code, you might as well make RLS use your real backend.
* [gfx-backend-gl](https://crates.io/crates/gfx-backend-gl) lets you target
  OpenGL 2.1+ and OpenGL ES2+. You'd probably use this if you wanted to run
  inside a webpage, or perhaps on a Raspberry Pi (which has OpenGL ES2 drivers,
  but not Vulkan), or anything else where you can't pick one of the "main"
  options. Unfortunately, the GL backend is actually a little busted at the
  moment. The biggest snag is that webpages and desktop apps have rather
  different control flow, so it's hard to come up with a unified API. Work is
  being done, and hopefully soon I'll be able to recommend the GL backend.

### Also `arrayvec`

As you might have noticed, we're going to be using
[arrayvec](https://docs.rs/arrayvec) later on for the `ArrayVec` type. I don't
want to come back to `Cargo.toml` later, so we can just mention it now.

`ArrayVec` works basically just like `Vec` but it's backed by an array on the
stack, not a data blob on the heap, so it can't resize, but it also doesn't need
a heap allocation to construct. We'll be using it during our draw code so that
we can call a few critical functions without doing a heap allocation each frame.
The functions in question have some weird generic bounds that work out for `Vec`
and `ArrayVec` and similar, but not for arrays themselves. Generics just be like
that sometimes.

# Making `gfx-hal` Clear The Screen

You might think that we'd start by learning how to initialize things, but
actually our core goal is clearing the screen. Anything else that we do,
including the initialization, is _only in service to that goal_. So first we'll
focus on our core goal, then we'll see what we need for that, and then we'll see
what we need for _that_, until eventually we stop needing to have already done
something else.

# Initializing `HalState`

.

# Cleaning Up `HalState`

.

# Input And LocalState

.

# TODO
# WIP
# OLD NOTES PAST HERE
* the spec _requires_ that a queue family not be empty, we don't have to check
  for 0, explain this

# Create A HalState Struct

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

This pattern is really obvious, but it'll get us pretty far. Note that not all
the values that end up in the base scope of our `new` method will go into the
`HalState` struct. Some of them just are just needed when going between each of
the major initialization steps, but we don't store them long term.

Unfortunately, we can no longer `derive(Debug)` on our struct, since the
`Instance` type doesn't have `Debug`. That's a little sad, but we'll live
through it.

## Create a Surface

Once our Instance is started, we want to make a
[Surface](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/trait.Surface.html). This
is the part where `winit` and `gfx-hal` touch each other just enough for them to
communicate.

```rust
    let mut surface = instance.create_surface(window);
```

The `create_surface` call is another of those methods that's part of the
Instance _types_ that each backend just happens to agree to have, rather than
being on the Instance _trait_ itself. You just pass in a `&Window` and it does
the right thing.

This means that our `new` method will need to accept a `&Window`. Spoilers:
that's all it'll need even once we're all done. So we can fill in that question
mark with a real argument list:

```rust
impl HalState {
  pub fn new(window: &Window) -> Self {
    // STUFF
  }
}
```

## Create an Adapter

Next we need an
[Adapter](https://docs.rs/gfx-hal/0.1.0/gfx_hal/adapter/struct.Adapter.html),
which represents the graphics card you'll be using. A given Instance might have
more than one available, so we call
[enumerate_adapters](https://docs.rs/gfx-hal/0.1.0/gfx_hal/trait.Instance.html#tymethod.enumerate_adapters)
on our Instance to get the list of what's available. How do we decide what to
use? Well, you might come up with any criteria you like. The biggest thing you
probably care about is if the Adapter can do graphics work and/or computation
work. For now we just want one that can do graphics work.

Each Adapter has a `Vec<B::QueueFamily>`, and a
[QueueFamily](https://docs.rs/gfx-hal/0.1.0/gfx_hal/queue/family/trait.QueueFamily.html)
has methods to check if that QueueFamily supports graphics, compute, and/or
transfer. If a QueueFamily supports graphics or compute it will always also
support transfer (otherwise you wouldn't be able to send it things to draw and
compute), but some QueueFamily could theoretically support _just_ transfer and
nothing else. Also, each QueueFamily has a maximum number of queues that's
available, and we obviously need to have more than 0 queues available for it to
be acceptable. Finally, we obviously need to make sure that our Surface supports
the QueueFamily we're selecting.

So we have a `Vec<Adapter<Self::Backend>>` and each of those holds a
`Vec<B::QueueFamily>`, sounds like it's time for some Iterator magic.

```rust
    let adapter = instance
      .enumerate_adapters()
      .into_iter()
      .find(|a| {
        a.queue_families
          .iter()
          .any(|qf| qf.supports_graphics() && qf.max_queues() > 0 && surface.supports_queue_family(qf))
      })
      .expect("Couldn't find a graphical Adapter!");
```

## Open up a Device

Okay so once we have an Adapter selected, we have to actually call
[open](https://docs.rs/gfx-hal/0.1.0/gfx_hal/adapter/trait.PhysicalDevice.html#tymethod.open)
on the associated PhysicalDevice to start using it. Think of this as the
difference between knowing the IP address you want to connect to and actually
opening the TCP socket that goes there. Look, they even have an example in the
docs.

We need to specify a reference to a slice of QueueFamily and QueuePriority tuple
pairs. Well we know how to get a QueueFamily we want, we just did that. A
[QueuePriority](https://docs.rs/gfx-hal/0.1.0/gfx_hal/adapter/type.QueuePriority.html)
is apparently just a 0.0 to 1.0 float for how high of priority we want. They use
1.0 in their example, and that seems fine to me.

Calling `open` gives us a Result, but we don't really know what to do if there's
a failure, so we'll just `expect` on that with a message like we have with other
things so far. This gives us a
[Gpu](https://docs.rs/gfx-hal/0.1.0/gfx_hal/struct.Gpu.html), which just bundles
up a Device and some Queues. The Queues value lets us call
[take](https://docs.rs/gfx-hal/0.1.0/gfx_hal/queue/family/struct.Queues.html#method.take)
to try and get out a particular QueueGroup by a specified id value. A QueueGroup
is just a vector of CommandQueue values with some metadata. We call `take` with
the id value of the QueueFamily we've been working with and hopefully get a
QueueGroup out. There's technically another Option layer we have to `expect`
away, but we're used to that by now I think. Once we have a QueueGroup, we can
get that vector of CommandQueue values and call it a day. Doesn't hurt much to
throw in a `debug_assert!` that we've really got at least one `CommandQueue`
available. We always _should_, because of the `filter` on the queue_families
that we did, but re-checking things you think are probably already true is the
whole point of a debug_assert after all.

TODO: max_queues can never be 0

```rust
    let (device, command_queues, queue_type, qf_id) = {
      let queue_family = adapter
        .queue_families
        .iter()
        .find(|qf| qf.supports_graphics() && qf.max_queues() > 0 && surface.supports_queue_family(qf))
        .expect("Couldn't find a QueueFamily with graphics!");
      let Gpu { device, mut queues } = unsafe {
        adapter
          .physical_device
          .open(&[(&queue_family, &[1.0; 1])])
          .expect("Couldn't open the PhysicalDevice!")
      };
      let queue_group = queues
        .take::<Graphics>(queue_family.id())
        .expect("Couldn't take ownership of the QueueGroup!");
      debug_assert!(queue_group.queues.len() > 0);
      (device, queue_group.queues, queue_family.queue_type(), queue_family.id())
    };
```

The `queue_type` and `qf_id` values will be used in a later step, but don't need
to be saved in our `HalState` struct.

## Create A Swapchain

The next thing to make is a
[Swapchain](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/trait.Swapchain.html).
It's uh, basically a chain of images that get swapped into place, like the name
says. As you might know, animation and video is a series of still images. When
you show them one after the other very quickly the viewer's brain registers it
as movement. The minimum speed you need for animation is actually very low, like
15fps or lower. Older TV shows run at around 24fps. With a computer these days
you're basically expected to use 30fps as the "low" framerate and 60fps as the
"standard" framerate.

Of course, since these images will be presented on the surface by the physical
device, we start with the
[compatibility](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/trait.Surface.html#tymethod.compatibility)
method to make sure that they can agree on something. This gets us a whole bunch
of information, only some of which we care about right now. We're trying to call
[Device::create_swapchain](https://docs.rs/gfx-hal/0.1.0/gfx_hal/device/trait.Device.html#tymethod.create_swapchain).
We have a surface, and we can specify `None` as our old swapchain since we don't
have a previous swapchain. In between those we need to have a
[SwapchainConfig](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/struct.SwapchainConfig.html).
We use
[from_caps](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/struct.SwapchainConfig.html#method.from_caps)
to just pass along our current capabilities, but we need to pick a specific
[Format](https://docs.rs/gfx-hal/0.1.0/gfx_hal/format/enum.Format.html) (the
layout of the image data) and a default extent (how many pixels wide and tall
the images are). The capabilities has an `extents` field which can give us our
default extent, so we just need a Format.

There's like, a million Formats available. You better believe that all of them
are used by someone, somewhere. It might be the case that you have image data
that's already in some format, and you try to pick that to cut down on data
conversion. We don't have any data already though, so we don't super care about
the format. Turns out the surface might not care about the format either. It
gives back an `Option<Vec<Format>>`, and `None` means "I don't care". If it
doesn't care, we'll pick `Rgba8Srgb`, which is the most commonly used form of
pixel data. `Rgba8` means "red, green, blue, 8 bits each", and `Srgb` literally
just stands for "standard red green blue". If there _is_ a preferred list we'll
take whatever `Srgb` offering they have. It's _possible_ (in terms of the type
system) that there's a preferred list given but it ends up being empty, which is
weird enough we'll throw a panic at that point.

Finally, we also want to specify a
[PresentMode](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/enum.PresentMode.html).
This is how we get that magical vsync thing that we wanted. There's a list of
possible modes that we get, and we want to pick the first one according to the
following list of "best to worst":

* Mailbox: Always VSync, and if you have **3 or more images** it'll keep
  rendering frames _faster_ than 60fps. One will always be "the frame being
  shown", one is tagged as the "most recent frame", and all the rest of the
  slots are cycled between as render targets (so _more_ than 3 isn't actually
  very helpful). This is the "triple buffering" style that you might have heard
  about. The user never sees graphical tears, and your user input to user output
  time always stays as low as possible. If you use this mode with only 2 images
  in your swapchain it basically ends up working like Fifo mode.
* Fifo: Always VSync, and always show frames in the exact order that they're
  created. This works best if you've got **2** images in your swapchain: one
  "being shown" and one either "being worked on" or "finished" (aka "double
  buffering"). _More_ than 2 images in your chain when using this mode will
  cause a longer delay between user input and image on screen.
* Relaxed: VSync as much as possible, but if we miss the timing by only a little
  bit just go anyway. At this point we're not even being precise about our
  results, but you might want to accept this as a possible fallback mode if the
  GPU doesn't support one of the first two, since at least there's probably a
  vsync _most_ of the time.
* Immediate: No VSync at all. Maybe someone wants it, but I don't.

```rust
    let (swapchain, extent, backbuffer, format) = {
      let (caps, opt_formats, present_modes, _composite_alphas) = surface.compatibility(&adapter.physical_device);
      let format = opt_formats.map_or(Format::Rgba8Srgb, |formats| {
        formats
          .iter()
          .find(|format| format.base_format().1 == ChannelType::Srgb)
          .cloned()
          .unwrap_or(*formats.get(0).expect("Given an empty preferred format list!"))
      });
      let present_mode = if present_modes.contains(&PresentMode::Mailbox) {
        PresentMode::Mailbox
      } else if present_modes.contains(&PresentMode::Fifo) {
        PresentMode::Fifo
      } else if present_modes.contains(&PresentMode::Relaxed) {
        PresentMode::Relaxed
      } else if present_modes.contains(&PresentMode::Immediate) {
        PresentMode::Immediate
      } else {
        panic!("Couldn't select a Swapchain presentation mode!")
      };
      let swap_config = SwapchainConfig::from_caps(&caps, format, caps.extents.end).with_mode(present_mode);
      let extent = swap_config.extent;
      let (swapchain, backbuffer) = unsafe {
        device
          .create_swapchain(&mut surface, swap_config, None)
          .expect("Failed to create the swapchain!")
      };
      (swapchain, extent, backbuffer, format)
    };
```

The `backbuffer` and `format` values will be used in a later step, but don't need
to be saved in our `HalState` struct.

## Define A RenderPass

Now that we've decided on a Format, we can define our
[RenderPass](https://docs.rs/gfx-hal/0.1.0/gfx_hal/trait.Backend.html#associatedtype.RenderPass).
This is unfortunately one of those things where the details are defined by each
particular backend. To make one, we call
[Device::create_render_pass](https://docs.rs/gfx-hal/0.1.0/gfx_hal/device/trait.Device.html#tymethod.create_render_pass).
Basically, [rendering can get really
complicated](https://stackoverflow.com/a/48304000/455232). Right now we just
want to clear the screen, so we're going to define an
[Attachment](https://docs.rs/gfx-hal/0.1.0/gfx_hal/pass/struct.Attachment.html)
for the color change and then a single
[SubpassDesc](https://docs.rs/gfx-hal/0.1.0/gfx_hal/pass/struct.SubpassDesc.html).
All we want to do is clear the screen, so we define an Attachment for the color
buffer (there's other types of buffer too, but we don't care at the moment). It
clears the color when the buffer is loaded, and then stores the buffer when the
subpass is done. You might have more than one subpass, and in that case _only
one_ of your passes would clear out the old image, and the rest would just keep
adding new edits to your work so far. Or, however you wanna do it. As you can
see if you check the docs, there's piles of options to dig in to here. You can
be sure that we'll get into more of this later.

```rust
    let render_pass = {
      let color_attachment = Attachment {
        format: Some(format),
        samples: 1,
        ops: AttachmentOps {
          load: AttachmentLoadOp::Clear,
          store: AttachmentStoreOp::Store,
        },
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::Present,
      };
      let subpass = SubpassDesc {
        colors: &[(0, Layout::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
      };
      unsafe {
        device
          .create_render_pass(&[color_attachment], &[subpass], &[])
          .expect("Couldn't create a render pass!")
      }
    };
```

## Create The ImageViews

Next we're going to take the Image values that our Backbuffer has and make one
ImageView each. This actually doesn't use anything from the `render_pass` step,
not all of the steps have a strict dependency.

```rust
    let image_views: Vec<(<back::Backend as Backend>::ImageView)> = match backbuffer {
      Backbuffer::Images(images) => images
        .into_iter()
        .map(|image| unsafe {
          device
            .create_image_view(
              &image,
              ViewKind::D2,
              format,
              Swizzle::NO,
              SubresourceRange {
                aspects: Aspects::COLOR,
                levels: 0..1,
                layers: 0..1,
              },
            )
            .expect("Couldn't create the image_view for the image!")
        })
        .collect(),
      Backbuffer::Framebuffer(_) => unimplemented!("Can't handle framebuffer backbuffer!"),
    };
```

We won't be using the ImageViews _directly_, but we'll need them in the next
step, and we'll also need to keep them around to manually clean up at the end.

## Create Our FrameBuffers

Once our ImageView values are all set we'll use them to make our FrameBuffer
values, which is what you _actually_ tell the GPU to draw with.

```rust
    let swapchain_framebuffers: Vec<<back::Backend as Backend>::Framebuffer> = {
      image_views
        .iter()
        .map(|image_view| unsafe {
          device
            .create_framebuffer(
              &render_pass,
              vec![image_view],
              Extent {
                width: extent.width as u32,
                height: extent.height as u32,
                depth: 1,
              },
            )
            .expect("Failed to create a framebuffer!")
        })
        .collect()
    };
```

## Create Our CommandPool

Next, we make a
[CommandPool](https://docs.rs/gfx-hal/0.1.0/gfx_hal/pool/struct.CommandPool.html).
As with the framebuffer part of things, this doesn't strictly depend on the
previous step. The idea here is that the GPU does _as little checking as
possible_, so internally it's using a thing that `gfx-hal` calls a
`RawCommandPool`. We want a little type safety on top, so we wrap that up in a
CommandPool which carries some PhantomData about what types of commands are
appropriate for that pool. Our Device gives us the RawCommandPool, we check that
it supports Graphics like we want, and then we wrap it into a CommandPool.

```rust
    let mut command_pool = unsafe {
      let raw_command_pool = device
        .create_command_pool(qf_id, CommandPoolCreateFlags::empty())
        .expect("Could not create the raw command pool!");
      assert!(Graphics::supported_by(queue_type));
      CommandPool::<back::Backend, Graphics>::new(raw_command_pool)
    };
```

We need the CommandPool in the next step, but after that we don't use it
directly, so we'll store it within HalState as an Option<CommandPool> value,
because that makes the cleanup easier later on.

## Create Our CommandBuffers

Once our CommandPool is ready, we can get some
[CommandBuffer](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.CommandBuffer.html)
values, which is what we _actually_ write out our drawing commands with. You
write into a CommandBuffer and then "submit" the buffer to the GPU and it does
the rest. Since we'll be doing the writing and submitting later on a frame by
frame basis, we can just get some now without writing anything to them.

```rust
    let submission_command_buffers: Vec<_> = swapchain_framebuffers.iter().map(|_| command_pool.acquire_command_buffer()).collect();
```

## Create Our Sync Primitives

I told you that the GPU avoids as many checks as possible by default, but that
even includes memory synchronization checks. Instead, we have to make a pile of
sync primitives and then use them by hand every frame.

For now, we just make a pile of them, and they'll go into use once we start
doing the drawing. We want to have one of each primitive per frame "in flight".
This is related to that `PresentMode` concept above. For our example we want to
have 3, so we'll define it as a const in the HalState struct.

```rust
impl HalState {
  const MAX_FRAMES_IN_FLIGHT: usize = 3;
}
```

In your own program you might want to have it depend on how much video memory is
available or some other factor that you check for at runtime.

The actual creation process is unexciting, we make semaphores for images being
available, semaphores for rendering of images being finished, and fences for an
image being in flight.

```rust
    let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
      let mut image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore> = vec![];
      let mut render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore> = vec![];
      let mut in_flight_fences: Vec<<back::Backend as Backend>::Fence> = vec![];
      for _ in 0..Self::MAX_FRAMES_IN_FLIGHT {
        image_available_semaphores.push(device.create_semaphore().expect("Could not create a semaphore!"));
        render_finished_semaphores.push(device.create_semaphore().expect("Could not create a semaphore!"));
        in_flight_fences.push(device.create_fence(true).expect("Could not create a fence!"));
      }
      (image_available_semaphores, render_finished_semaphores, in_flight_fences)
    };
```

## Wrap Up HalState

Now we're all done, except that we also want to add a `current_frame` value so
that we know which of those sync primitive slots we're on within the vectors we
just set up.

The only thing left to do is return our new struct.

```rust
    Self {
      _instance: instance,
      _surface: surface,
      _adapter: adapter,
      device,
      command_queues,
      swapchain,
      extent,
      render_pass,
      image_views,
      swapchain_framebuffers,
      command_pool: Some(command_pool),
      submission_command_buffers,
      image_available_semaphores,
      render_finished_semaphores,
      in_flight_fences,
      current_frame: 0,
    }
```

# Drawing A Clear Frame

Now that we've got all of our HalState declared, let's make it able to draw a
frame to a designated clear color.

This whole method is just... just one big unsafe block. Or, well, we could make
the method be unsafe, but basically we have to blindly mask away the unsafety at
some point, so _oh well_.

## Not Using Expect Anymore

In some sense, it's _maybe_ okay for an example to have a bunch of uses of
`expect` despite the fact that there might reasonably be errors. Don't get me
wrong, you should always avoid a panic you don't need in your long term code,
but since we don't really know what we're doing yet, and since we don't really
know how to recover from problems yet, we've kinda got no choice except to
panic.

It's still not at all elegant.

So, moving forward we'll avoid panics as much as we can. To start with instead
of having some method on HalState like

```rust
pub fn draw_clear_frame(&mut self, color: [f32; 4])
```

We're going to make it be

```rust
pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &'static str>
```

This is a little better. Not the best. In the long term we'd want to sort out
all the possible error cases and classify them into a big enum or something.
That'd be neat, and easy for the caller to match on when an error does happen.
Except we don't really _know_ all possible errors yet, so we'll just start with
string literals. Just to keep ourselves in the habit of returning a Result
instead of triggering a panic.

## Select Our Sync Primitives

First we'll get out the sync primitives for the frame count we're on.

```rust
      let fence = &self.in_flight_fences[self.current_frame];
      let image_available = &self.image_available_semaphores[self.current_frame];
      let render_finished = &self.render_finished_semaphores[self.current_frame];
```

## Select An Image

Now we have to pick what image we're working on. We wait for the fence of this
index so that we don't overwrite an image while it's actually in flight. Then we
reset it, and try to acquire an image. The image isn't necessarily immediately
ready, so we pass in a semaphore here that will be signalled in a moment.

```rust
      self
        .device
        .wait_for_fence(fence, core::u64::MAX)
        .map_err(|_| "Failed to wait on the fence!")?;
      self.device.reset_fence(fence).map_err(|_| "Couldn't reset the fence!")?;
      let image_index = self
        .swapchain
        .acquire_image(core::u64::MAX, FrameSync::Semaphore(image_available))
        .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
      let i = image_index as usize;
```

## Write The Command Buffer

With an index in hand, we record a command buffer. We pick out the one for the
index we got and tell it to clear the whole render area to the color that was
given in the method argument. That's it, that's all we're gonna do.

The whole thing goes inside an dummy scope so that the `&mut` on the submission
command buffers goes away before the next step.

```rust
      {
        let command_buffer = &mut self.submission_command_buffers[i];
        command_buffer.begin(true);
        let render_area = Rect {
          x: 0,
          y: 0,
          w: self.extent.width as i16,
          h: self.extent.height as i16,
        };
        let clear_values = [ClearValue::Color(ClearColor::Float(color))];
        command_buffer.begin_render_pass_inline(&self.render_pass, &self.swapchain_framebuffers[i], render_area, clear_values.iter());
        command_buffer.finish();
      }
```

## Submit The Buffer, Present The Image

Once we've got our command buffer all written we have to submit it. Once it's
been properly submitted, we have to "present" the image as a separate step. Of
course the submission might be finished recording before the image we're doing
it for is ready, and we might try to present it before the commands have
actually been completed. This is where all those sync primitives play their
part.

Once the call to present returns we do the equivalent of something like
`self.current_frame += 1`, except that we need it to also roll around when we
hit our MAX_FRAMES_IN_FLIGHT value, so there's a mod too. Divisions are slow and
all, but we can pretty much trust the compiler to turn this mod into a mul and
shift operation, since it's mod by a constant
([godbolt](https://rust.godbolt.org/z/Drwbg0)).

```rust
      let submission = Submission {
        command_buffers: &self.submission_command_buffers[i..=i],
        wait_semaphores: vec![(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)],
        signal_semaphores: vec![render_finished],
      };
      self.command_queues[0].submit(submission, Some(fence));
      self
        .swapchain
        .present(&mut self.command_queues[0], image_index, vec![render_finished])
        .map_err(|_| "Couldn't present the image!")?;
      self.current_frame = (self.current_frame + 1) % Self::MAX_FRAMES_IN_FLIGHT;
      Ok(())
```

# Cleanup Code

At the end of it all we want to clean up our stuff. Sadly, `gfx-hal` doesn't
clean up much automatically.

We just call `self.device.destroy_thing` for each thing that needs to be
destroyed. That's not complicated. What makes it complex is that a few things
are difficult to move out of a borrowed context.

See, Drop is defined to have a method `drop` that takes `&mut self`, like this

```rust
impl core::ops::Drop for HalState {
  fn drop(&mut self) {
    // STUFF
  }
}
```

And then all the vectors we can drain out:

```rust
      for fence in self.in_flight_fences.drain(..) {
        self.device.destroy_fence(fence)
      }
      for semaphore in self.render_finished_semaphores.drain(..) {
        self.device.destroy_semaphore(semaphore)
      }
      for semaphore in self.image_available_semaphores.drain(..) {
        self.device.destroy_semaphore(semaphore)
      }
      for image_view in self.image_views.drain(..) {
        self.device.destroy_image_view(image_view);
      }
      for framebuffer in self.swapchain_framebuffers.drain(..) {
        self.device.destroy_framebuffer(framebuffer);
      }
```

And the CommandPool is in an Option, so we can use `take` and then `map` to cleanly handle that

```rust
      self.command_pool.take().map(|command_pool| {
        self.device.destroy_command_pool(command_pool.into_raw());
      });
```

But the RenderPass and the Swapchain... we want to destroy them, but we can't
move them out of the borrowed context. And we want to not have them be wrapped
in Option since we'll be using them every single frame and it'll be a lot of
code noise to be dealing with that Option layer.

So...

We'll just do the _hyper unsafe_ thing, and use `replace`. What will go in the
old position? Just a `zeroed` value. Is that legal? I really don't think so. But
you can't witness the struct after it's been dropped so you can't actually use
the zeroed data so... It's probably fine? We'll hope it's fine.

```rust
      // BIG DANGER HERE, DO NOT DO THIS OUTSIDE OF A DROP
      use core::mem::{replace, zeroed};
      self.device.destroy_render_pass(replace(&mut self.render_pass, zeroed()));
      self.device.destroy_swapchain(replace(&mut self.swapchain, zeroed()));
```

And now our HalState works with Drop just like anything else.

# Wrapping Up The Example

We're almost home! I promise!

## Picking A Clear Color

To pick a color each frame, first we add a few variables to our pile of locals.

```rust
  let mut running = true;
  let (mut frame_width, mut frame_height) = winit_state.window.get_inner_size().map(|logical| logical.into()).unwrap_or((0.0, 0.0));
  let (mut mouse_x, mut mouse_y) = (0.0, 0.0);
```

And we make the input gathering a little more interesting

```rust
  'main_loop: loop {
    winit_state.events_loop.poll_events(|event| match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => running = false,
      Event::WindowEvent {
        event: WindowEvent::Resized(logical),
        ..
      } => {
        frame_width = logical.width;
        frame_height = logical.height;
      }
      Event::WindowEvent {
        event: WindowEvent::CursorMoved { position, .. },
        ..
      } => {
        mouse_x = position.x;
        mouse_y = position.y;
      }
      _ => (),
    });
    if !running {
      break 'main_loop;
    }
```

And then with this data we do _some_ arbitrary thing to pick us a color each
frame and call the method. We'll use the mouse's X and Y position to generate
Red and Green channel values for our color.

```rust
    // This makes a color that changes as the mouse moves, just so that there's
    // some feedback that we're really drawing a new thing each frame.
    let r = (mouse_x / frame_width) as f32;
    let g = (mouse_y / frame_height) as f32;
    let b = (r + g) * 0.3;
    let a = 1.0;

    if let Err(e) = hal_state.draw_clear_frame([r, g, b, a]) {
      error!("Error while drawing a clear frame: {}", e);
      break 'main_loop;
    }
```

## Closing Up Shop

One thing is that before we let the HalState struct drop we have to try and wait
for all active queues to finish out. Otherwise we'll get all sorts of bad use
after free stuff.

After whe loop we put

```rust
  // If we leave the main loop for any reason, we want to shut down as
  // gracefully as we can.
  if let Err(e) = hal_state.wait_until_idle() {
    error!("Error while waiting for the queues to idle: {}", e);
  }
```

And inside HalState we make a small helper:

```rust
  /// Waits until the device goes idle.
  pub fn wait_until_idle(&self) -> Result<(), HostExecutionError> {
    self.device.wait_idle()
  }
```

And we're done.

## Turn It On

If you turn on the program you should get a black screen that shifts around with
shades of green and magenta when you move the mouse around.

