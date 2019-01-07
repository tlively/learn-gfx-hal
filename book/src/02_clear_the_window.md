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
This is how we get that magical vsync thing that we wanted. There's a list of possible modes that we get, and we want to pick the first one according to the following list of "best to worst":

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

## Create The Images

We also need to create some
[Image](https://docs.rs/gfx-hal/0.1.0/gfx_hal/trait.Backend.html#associatedtype.Image)
and
[ImageView](https://docs.rs/gfx-hal/0.1.0/gfx_hal/trait.Backend.html#associatedtype.ImageView)
pairs. These are defined by the backend, so no docs really.

```rust
    let frame_images: Vec<(<back::Backend as Backend>::Image, <back::Backend as Backend>::ImageView)> = match backbuffer {
      Backbuffer::Images(images) => images
        .into_iter()
        .map(|image| {
          let image_view = unsafe {
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
          };
          (image, image_view)
        })
        .collect(),
      Backbuffer::Framebuffer(_) => unimplemented!("Can't handle framebuffer backbuffer!"),
    };
```

## Create Our FrameBuffers

```rust
    let swapchain_framebuffers: Vec<<back::Backend as Backend>::Framebuffer> = {
      frame_images
        .iter()
        .map(|(_, image_view)| unsafe {
          device
            .create_framebuffer(
              &render_pass,
              vec![image_view],
              Extent {
                width: extent.width as _,
                height: extent.height as _,
                depth: 1,
              },
            )
            .expect("Failed to create a framebuffer!")
        })
        .collect()
    };
```

## Create Our CommandPool

```rust
    let mut command_pool = unsafe {
      let raw_command_pool = device
        .create_command_pool(qf_id, CommandPoolCreateFlags::empty())
        .expect("Could not create the raw command pool!");
      assert!(Graphics::supported_by(queue_type));
      CommandPool::<back::Backend, Graphics>::new(raw_command_pool)
    };
```

## Create Our CommandBuffers

```rust
    let submission_command_buffers: Vec<_> = swapchain_framebuffers.iter().map(|_| command_pool.acquire_command_buffer()).collect();
```

## Create Our Sync Primitives

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

# Drawing A Clear Frame

# Not Using Expect

## Select Our Sync Primitives

```rust
      let fence = &self.in_flight_fences[self.current_frame];
      let image_available = &self.image_available_semaphores[self.current_frame];
      let render_finished = &self.render_finished_semaphores[self.current_frame];
```

## Select An Image

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

# Wrapping It Up

//.