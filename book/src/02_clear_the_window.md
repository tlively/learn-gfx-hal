# Clearing The Window

Once you have a window open, the _usual_ next step for a graphics tutorial is to
draw "your first triangle". You see, the fundamental primitive of 3d graphics is
the triangle. Yes, there are some systems such as the [Sega
Saturn](https://en.wikipedia.org/wiki/Sega_Saturn) that use quads instead, but
in all the modern systems you'll find it's going to be triangles. Even a quad is
just two triangles, when you think about it. With enough math and enough
parallel processing you can do anything you want with triangles.

* Skyrim? Triangles.
* Breath of The Wild? Has a few more triangles than Skyrim.
* Super Smash Bros? Just a whole lot of triangles.

We'll be covering triangles _quite_ a bit. However, in the context of `gfx-hal`,
which is like 97% "whatever Vulkan does", even if you're _not_ using the Vulkan
backend, there's a great _many_ steps of setup involved between "a window that
draws nothing" and "a window that draws one triangle".

In fact the [official gfx-hal docs](https://docs.rs/gfx-hal/0.1.0/gfx_hal/)
specifically give us a warning about this. The top level docs are so short I can
include all three sentences right here for dramatic effect:

> Low-level graphics abstraction for Rust. Mostly operates on data, not types.
> Designed for use by libraries and higher-level abstractions only.

There are basically no defaults provided. We have to list out every single
little step of the entire configuration process. I mean they convert C types
into Rust types for us, but it's still very "do it yourself". That's _cool_ if
you actually care about defining it all (which you will some day, I'm sure, or
you wouldn't be reading this right now), but it's also _long_ when you're
starting out and want to get something on the screen.

Since going all the way to "drawing a triangle" might end up feeling like too
much at once, we'll stop this lesson at an intermediate step. Remember how our
`winit` window from last lesson didn't refresh itself properly? We can fix just
that much and then stop there. That _alone_ will cover a surprising amount of
ground.

## Outline Our Target API

So, in the first lesson we had `WinitState` and it just had two public fields.
There's not much there, it's all safe code, and it's not the focus of our
lessons, so that's fine. I mean I guess you could pair up the wrong `EventLoop`
with the wrong `Window` or something, but two public fields is good enough.

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
design. Let's keep it short:

* Always, _Always_, **_Always_** [write the usage code
  first.](https://caseymuratori.com/blog_0025)

Even before we know _any details about how `gfx-hal` works_, we're going to just
write out how we _think_ we should be able to use it. How we think it's be
easiest to use. Once it's built we will be calling the methods a lot more than
we'll be implementing the methods, so unless we end up with some sort of
performance disaster or impossible requirement we'll keep the exterior simple
even if it means the interior might end up a little more complex.

So what's our _usage_ of the `HalState` type look like?

There's lots of answers you could have to that question. Really, there are.
Obviously since I'm writing this we're going to be using what I came up with,
but if you think you can get a better solution you should try it out. I'll try
to explain my thinking as best as I can, and hopefully you'll agree with me.

### Initialization

We already have `WinitState`, we're going to want `HalState` too. Clearly the
`WinitState` can be made before the `HalState` (since we did it last lesson).

We'll also want to have a `LocalState`, and that's the grab bag of everything
else in the program. If you're doing a game or a simulation or something that's
your `GameState` or `World` or whatever you wanna call the type.

So far the code outline looks like this:

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
~~probably~~ definitely depends on the `Window` it's going to draw within. It
can't be totally default with no inputs. We need a `HalState` initialization
method that takes a `Window` reference. The default name for any initialization
method in Rust is just `new`, and I can't think of a better name to use, so
we'll go with that.

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
local variables according to the input (eg: in a game you might move the player a tiny
bit, or whatever change), and then render the new state of the world into a frame that
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

This should look fairly familiar after what we did in the first lesson.

You may be wondering why the `do_the_render` function is taking a `&mut
HalState` as the first argument, instead of having it be a `&mut self` method on
the `HalState` type. Well, I'm not sure it's the perfect decision, but we're
going to _try_ and keep our `HalState` and `LocalState` as totally separate as
we can.

* If `HalState` doesn't know anything about the `LocalState` then it's a lot
  more likely to focus on reusable drawing operations, and we'll be a lot more
  likely to have something we can reuse in future situations (including
  "practical" situations beyond just this tutorial series).
* Similarly, if `LocalState` doesn't know about `HalState` then it's easier for
  it to focus on the "business logic" without worrying about anything else. We
  could even run the `LocalState` _without graphics at all_ (sometimes called a
  "headless" mode), which can be nice if you want to do CI tests, or hook it to
  a server people connect to, or any other unexpected use.

It can often be _tempting_ to make everything into a method on some type, but
that's an urge we need to resist in this situation.

### What Does `do_the_render` Actually Do?

I cheated a bit there, because I wrote down a call to `do_the_render` without
actually saying _what_ it's doing on the inside. That's the part we care about
the most! That's how we know what our `HalState` API needs to look like.

For this lesson, all we do is clear the screen. That sounds simple enough. Later
lessons will add more, but this is our starting point.

```rust
pub fn do_the_render(hal: &mut HalState, locals: &LocalState) {
  hal.draw_clear_frame(locals.color());
}
```

That looks okay at first, but there might be some sort of error that happens
during rendering. Nothing inside `do_the_render` particularly knows about how to
handle an error, so we'll just pass that back up the stack.

```rust
pub fn do_the_render(hal: &mut HalState, locals: &LocalState) -> Result<(), &str> {
  hal.draw_clear_frame(locals.color())
}
```

And then in `main` I guess we can just... log the error and quit? It's not ideal
for the program to shut itself down unexpectedly, but we don't really have a
backup strategy at the moment. In a more advanced situation the error might be
from the user trying to switch graphics settings or something, so you could
automatically switch back to the previous settings in that case. Depends on the
program, and the error.

Also, in a full program you'd want to use a proper error enum, but we don't know
what all our possible errors are, so we'll just use string literals for now.

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
cleanup work. `gfx-hal` is no different. Not only do we need to clean things up
to avoid leaks when we're done, we need to clean up in the exactly correct
order. The backend code can segfault your process just by you not shutting it
down properly.

How do we expose this in our API?

We _don't_.

I'm not saying that we _ignore_ the subject of cleanup, that would be foolish,
but I am saying that we should keep all of it entirely within the `HalState`
type. Things are smoothest for the user when they can just let a type drop away
without a care, and we're going to try and allow for such an easy use
experience. Mostly what this means is that we won't want to have any "getter"
methods that let an outside user move out anything that needs to be manually
destroyed later. If they want to check the value of a number or maybe even get a
`&mut` to some that's fine, but anything that needs to be explicitly cleaned up
we can't let out of our control.

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

# Implementing `draw_clear_frame`

You might think that we'd start by learning how to initialize things, but
actually our core goal is clearing the screen. Anything else that we do,
including the initialization, is _only in service to that goal_. So first we'll
focus on our core goal, then we'll see what we need for that, and then we'll see
what we need for _that_, until eventually we stop needing to have already done
something else.

## CommandQueue

The heart of it all is that we want to be able to safely call
[CommandQueue::submit](https://docs.rs/gfx-hal/0.1.0/gfx_hal/queue/struct.CommandQueue.html#method.submit),
which defines the "list" of what to do, and then we call
[Swapchain::present](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/trait.Swapchain.html#method.present),
which does the work and places the output into the "swapchain" of images that
the GPU uses from frame to frame.

### `submit`

The actual type of the `submit` method is _super generic_ which means that it
reads like a pile of space runes:

```rust
pub unsafe fn submit<'a, T, Ic, S, Iw, Is>(
    &mut self, 
    submission: Submission<Ic, Iw, Is>, 
    fence: Option<&B::Fence>
)
where
    T: 'a + Submittable<B, C, Primary>,
    Ic: IntoIterator<Item = &'a T>,
    S: 'a + Borrow<B::Semaphore>,
    Iw: IntoIterator<Item = (&'a S, PipelineStage)>,
    Is: IntoIterator<Item = &'a S>,
```

Gross, right? Let's cut out those generics and look again:

```rust
pub unsafe fn submit(&mut self, submission: Submission, fence: Option<&B::Fence>)
```

Okay that's _way_ easier to look at and understand. It's just a rustified
version of
[vkQueueSubmit](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkQueueSubmit.html).
Which doesn't mean anything to you right now because we're just starting, but
like I said at the top: if the `gfx-hal` docs aren't clear on their semantics,
you can usually assume that Vulkan semantics apply.

* We `submit` a `Submission` into the `CommandQueue`. Instead of giving a count
  ans a pointer to an array of "VkSubmitInto", we give a single `Submission`,
  which is itself composed of `IntoIterator` things that I assume get iterated
  over. I don't know why it's flipped around like that instead of just having us
  pass in a `&[SubmissionInfo]`, which would be the closest to Vulkan. Probably
  some other backend forced a divergence.
* We optionally give a "fence" which gets "signalled" once all of the submitted
  command buffers have completed execution.

### `present`

The `present` method looks like this

```rust
unsafe fn present<'a, C, S, Iw>(
    &'a self, 
    present_queue: &mut CommandQueue<B, C>, 
    image_index: SwapImageIndex, 
    wait_semaphores: Iw
) -> Result<(), ()>
where
    Self: 'a + Sized + Borrow<B::Swapchain>,
    C: Capability,
    S: 'a + Borrow<B::Semaphore>,
    Iw: IntoIterator<Item = &'a S>, 
```

And if we cut out the extra stuff:

```rust
unsafe fn present(
    &self, 
    present_queue: &mut CommandQueue<B, C>,
    image_index: SwapImageIndex,
    wait_semaphores: Iw) -> Result<(), ()>
```

So `present` takes a `&mut` to our `CommandQueue`, a target index within the
Swapchain to present to, and a semaphore to wait on before actually presenting
the image. This works like
[vkQueuePresentKHR](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkQueuePresentKHR.html).
When we call `submit`, one of the Submission elements is going to be a semaphore
to signal when the rendering is done. When we call `present` we give it that
same semaphore to wait on before presenting the image, so that the user only
sees complete images.

### Fences?

A [fence](https://en.wikipedia.org/wiki/Memory_barrier) (aka
[VkFence](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkFence.html))
is one of the two synchronization primitives we'll be dealing with. It's
basically just a bool, it can "signaled" or "unsignaled". You can share a fence
between threads and everyone always sees the current state, so it's _like_
having an
[AtomicBool](https://doc.rust-lang.org/core/sync/atomic/struct.AtomicBool.html).

Fences are for CPU to GPU synchronization. The CPU can wait on a fence, and the
GPU will signal the fence when it's done whatever it's supposed to have done.

### Semaphores?

A [semaphore](https://en.wikipedia.org/wiki/Semaphore_(programming)) (aka
[VkSemaphore](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSemaphore.html))
is the other synchronization primitive. In some contexts a semaphore can be any
integer value, but in a Vulkan / `gfx-hal` context they can only be "signaled"
or "unsignaled".

The big difference between a fence and a semaphore is that semaphores are for
GPU to GPU synchronization. When once part of a graphics pipeline (such as
presentation to the swapchain) depends on another part of the pipeline (such as
command buffer processing), then you describe that dependency to the GPU using a
semaphore.

## Submission

Supposing that we already have a `CommandQueue` from somewhere, we need to give
it a `Submission` of what to do.

```rust
pub struct Submission<Ic, Iw, Is> {
    pub command_buffers: Ic,
    pub wait_semaphores: Iw,
    pub signal_semaphores: Is,
}
```

Hmm, but the `submit` method had extra bounds in there:

```rust
  T: 'a + Submittable<B, C, Primary>,
  Ic: IntoIterator<Item = &'a T>,
  S: 'a + Borrow<B::Semaphore>,
  Iw: IntoIterator<Item = (&'a S, PipelineStage)>,
  Is: IntoIterator<Item = &'a S>,
```

So if we put that together, and allow ourselves to use some slightly fake Rust
syntax for just a moment, we need to build this:

```rust
pub struct Submission {
    pub command_buffers: IntoIterator<Item = &'a Submittable<B, C, Primary>>,
    pub wait_semaphores: IntoIterator<Item = (&'a Borrow<B::Semaphore>, PipelineStage)>,
    pub signal_semaphores: IntoIterator<Item = &'a Borrow<B::Semaphore>>,
}
```

* `command_buffers` is our
  [Submittable](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/trait.Submittable.html)
  things, which are `Borrow<B::CommandBuffer>`, so we can think of that as being
  "sorta like &CommandBuffer".
* `wait_semaphores` gives the semaphores that this submission has to _wait on
  before it starts_. Each semaphore is paired with a
  [PipelineStage](https://docs.rs/gfx-hal/0.1.0/gfx_hal/pso/struct.PipelineStage.html),
  allowing your submission to wait for a stage, do some work at that stage, wait
  for another stage, do some more work at the new stage, and so on.
* `signal_semaphores` gives a list of semaphores that this submission _will
  signal once it completes_. It doesn't say, but I'm guessing that all the
  semaphores just get signaled at once at the end of the Submission.

All of this is basically what you find in the
[VkSubmitInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSubmitInfo.html)
struct.

### ArrayVec Submissions

Remember when I said that we'd use the ArrayVec to avoid allocations per frame?
That's this part. It's simple really. Instead of writing something like:

```rust
let command_buffers = vec![the_command_buffer];
```

You write something like

```rust
let command_buffers: ArrayVec<[_; 1]> = [the_command_buffer].into();
```

## Submitting And Presenting

So far it sounds like we want something like this

```rust
  pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &str> {
    // SETUP FOR THIS FRAME
    // ...

    // RECORD SOME COMMANDS
    // ...

    // SUBMISSION
    let command_buffers: ArrayVec<[_; 1]> = [the_command_buffer].into();
    let wait_semaphores: ArrayVec<[_; 1]> = [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
    let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    let submission = Submission {
      command_buffers,
      wait_semaphores,
      signal_semaphores,
    };
    unsafe {
      the_command_queue.submit(submission, Some(flight_fence));
      the_swapchain.present(&mut the_command_queue, i_u32, present_wait_semaphores)
        .map_err(|_|"Failed to present into the swapchain!")
    }
  }
```

For all my fuss about things being so "manual and on your own", that seems
fairly reasonable so far.

## Recording Commands

So we need to fill up a
[CommandBuffer](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.CommandBuffer.html)
with the operations that we want to have happen during the draw process.

All we want to do is clear the screen, that's got to be easy enough.

Well, it turns out that a CommandBuffer isn't totally free to make, so we want
to make them ahead of time and then pick out and use a particular command buffer
each frame. Also, we want to grab a `&mut` to the particular command buffer when
recording it, and then we'll also need a `&` to it when we get it to the
submission, so that calls for the ever-lovable inner scope. Thankfully, in a
sense, recording to a CommandBuffer is all unsafe, so we can kill two birds with
one stone.

A CommandBuffer is actually wrapping around a
[RawCommandBuffer](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/trait.RawCommandBuffer.html)
with some metadata for bonus type safety. All the real documentation is given on
the RawCommandBuffer type. Unfortunately, the methods aren't _exactly_ the same
name. Hopefully that's fixed in 0.2.

We start by calling `begin`

```rust
pub unsafe fn begin(&mut self, allow_pending_resubmit: bool)
```

To begin the buffer overall. Then we start a particular render pass with

```rust
pub unsafe fn begin_render_pass_inline<T>(
    &mut self, 
    render_pass: &B::RenderPass, 
    frame_buffer: &B::Framebuffer, 
    render_area: Rect, 
    clear_values: T
) -> RenderPassInlineEncoder<B>
where
    T: IntoIterator,
    T::Item: Borrow<ClearValue>, 
```

Which records a render pass with no secondary command buffers.

Next we... immediately finish the render pass. The RenderPass struct will define
how to deal with the color buffer, including the clear effect, and the
ClearValue picks what color to clear to, so we're already done.

```rust
  pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &str> {
    // SETUP FOR THIS FRAME
    // ...

    // RECORD SOME COMMANDS
    {
      let buffer = &mut self.command_buffers[i_usize];
      let clear_values = [ClearValue::Color(ClearColor::Float(color))];
      buffer.begin(false);
      buffer.begin_render_pass_inline(
        &self.render_pass,
        &self.swapchain_framebuffers[i_usize],
        self.render_area,
        clear_values.iter(),
      );
      buffer.finish();
    }

    // SUBMISSION
    // ...
  }
```

## Frame Setup

What's left to so as setup? Well, the GPU can be doing more than one of these
buffer things at once. When you've got several images all going on it's called
having frames "in flight". At the start of each frame of work, we have to pick
the right fences and semaphores and all that for the current frame that we're
going to be working with. The simplest way is to just keep them in parallel
vectors and go through them like a ring buffer.

However, even once we've picked our current sync primitives, we have to
[acquire](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkAcquireNextImageKHR.html)
a particular image to work with out of the swapchain. We don't move the whole
image out of the swapchain, we just get an index to target later with the
`present` method.

## Final `draw_clear_frame` Code

So now we put it all together, with the signaling in big caps to help make it
clear.

* Get our sync primitives out of our ring buffers
* WAIT on the current `flight_fence` to know we're in the clear to use this
  position of our ring buffer.
* Reset that fence so we can pass it as part of our submission later.
* Grab an image index that will SIGNAL the `image_available` semaphore once it's
  fully ready.
* Record our command buffer while we're waiting for that.
* Submit a command buffer to WAIT on `image_available` and SIGNAL both
  `render_finished` and `flight_fence`.
* Present the results into the swapchain after a WAIT on `render_finished`

```rust
  /// Draw a frame that's just cleared to the color specified.
  pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &str> {
    // SETUP FOR THIS FRAME
    let flight_fence = &self.in_flight_fences[self.current_frame];
    let image_available = &self.image_available_semaphores[self.current_frame];
    let render_finished = &self.render_finished_semaphores[self.current_frame];
    // Advance the frame _before_ we start using the `?` operator
    self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

    let (i_u32, i_usize) = unsafe {
      self
        .device
        .wait_for_fence(flight_fence, core::u64::MAX)
        .map_err(|_| "Failed to wait on the fence!")?;
      self
        .device
        .reset_fence(flight_fence)
        .map_err(|_| "Couldn't reset the fence!")?;
      let image_index = self
        .swapchain
        .acquire_image(core::u64::MAX, FrameSync::Semaphore(image_available))
        .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
      (image_index, image_index as usize)
    };

    // RECORD COMMANDS
    unsafe {
      let buffer = &mut self.command_buffers[i_usize];
      let clear_values = [ClearValue::Color(ClearColor::Float(color))];
      buffer.begin(false);
      buffer.begin_render_pass_inline(
        &self.render_pass,
        &self.swapchain_framebuffers[i_usize],
        self.render_area,
        clear_values.iter(),
      );
      buffer.finish();
    }

    // SUBMISSION AND PRESENT
    let command_buffers = &self.command_buffers[i_usize..=i_usize];
    let wait_semaphores: ArrayVec<[_; 1]> = [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
    let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    // yes, you have to write it twice like this. yes, it's silly.
    let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    let submission = Submission {
      command_buffers,
      wait_semaphores,
      signal_semaphores,
    };
    let the_command_queue = &mut self.queue_group.queues[0];
    unsafe {
      the_command_queue.submit(submission, Some(flight_fence));
      self
        .swapchain
        .present(the_command_queue, i_u32, present_wait_semaphores)
        .map_err(|_| "Failed to present into the swapchain!")
    }
  }
```

# Initializing `HalState`

So for our `draw_clear_frame` method to work it expects that we have many things
on hand as part of `HalState`. Listing them in the order that they're used:

* fences (requires a Device)
* semaphores (requires a Device)
* current_frame (just starts at 0)
* frames_in_flight (comes from the Swapchain)
* device (requires an Adapter)
* swapchain (requires a Surface)
* command_buffers (requires a CommandPool)
* render_pass (requires a Device)
* swapchain_framebuffers (requires ImageView values)
* render_area (comes from the Swapchain)
* queue_group (requires an Adapter)

But, as you can probably guess, that's _not_ the order that they're initialized.
You should have noticed that there's some things on there we haven't even
discussed yet, which also have their requirements. In no particular order:

* image_views (requires a Backbuffer)
* backbuffer (requires a Surface)
* Command Pool (requires a Device)
* Surface (requires a Window)
* Adapter (requires an Instance)
* Instance

And so we'll have to sort out all of our initialization code so that we can
build it all up in the correct order.

TODO

# Cleaning Up `HalState`

.

# Input And LocalState

.
