# INCOMPLETE

# TODO

# WORK IN PROGRESS

## TODO

* the spec _requires_ that a queue family not be empty, we don't have to check for 0
* no-alloc Submissions (file an issue about the Vec thing)
* fames in flight assert
* Drop Code doesn't close out the command pool / command buffers right, according to VK
* Drop code does some unsafe zero replacing, very dodgy.

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
[simple_logger](https://crates.io/crates/simple_logger) since it just spits
things to `stdout` and `stderr` instead of needing to setup log files. You also
don't need to remember to configure log levels with environment variables or
anything like that, which makes it easy to use for people who are forgetful or
who are using the always anemic `cmd.exe` command prompt. That's just fine for a
tutorial, so we'll do that. We just add a bit more to our `Cargo.toml`:

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

One the call to present returns we do the equivalent of something like
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

