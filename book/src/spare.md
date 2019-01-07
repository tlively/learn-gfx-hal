# INCOMPLETE

# TODO

# WORK IN PROGRESS

# Drawing A Triangle

# TODO: kill all the stuff past here

# TODO: cut out as much vec as we can

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



## Create a Surface

Once our Instance is started, we want to make a
[Surface](https://docs.rs/gfx-hal/0.1.0/gfx_hal/window/trait.Surface.html). This
is the part where `winit` and `gfx-hal` touch each other just enough for them to
communicate.

```rust
  let surface = instance.create_surface(&winit_state.window);
```

The `create_surface` call is another of those methods that's part of the
Instance _types_ that each backend just happens to agree to have, rather than
being on the Instance _trait_ itself. You just pass in a `&Window` and it does
the right thing.

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
    .filter(|a| {
      a.queue_families
        .iter()
        .find(|qf| qf.supports_graphics() && qf.max_queues() > 0 && surface.supports_queue_family(qf))
        .is_some()
    })
    .next()
    .expect("Couldn't find a graphical Adapter!");
```

## Open up a Device

Okay so once we have an Adapter selected, we have to actually call
[open](https://docs.rs/gfx-hal/0.1.0/gfx_hal/adapter/trait.PhysicalDevice.html#tymethod.open)
on the associated PhysicalDevice to start using it. Think of this as the
difference between knowing the IP address you want to connect to and actually
opening the TCP socket that goes there.

Look, they even have a sample call to make. We need to specify a reference to a
slice of QueueFamily and QueuePriority tuple pairs. Well we know how to get a
QueueFamily we want, we just did that. A
[QueuePriority](https://docs.rs/gfx-hal/0.1.0/gfx_hal/adapter/type.QueuePriority.html)
is apparently just a 0.0 to 1.0 float for how high of priority we want. They use
1.0 in their example, so that seems fine to me.

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
  let (device, command_queues) = {
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
    (device, queue_group.queues)
  };
```

## Create a SwapChain

TODO!
