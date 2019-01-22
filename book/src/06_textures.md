# Textures

You can draw a lot with just triangles and colors. Do a search for "low poly
art" and you'll find a bunch of stuff that's just lots and lots of color shaded
triangles. Like the digital version of stained glass art. It's really cool.

But you can't make Skyrim or Smash Bros with just colored triangles. At some
point you want to stick a picture of a thing on those triangles. A picture that
you place onto a model is called a "texture", even though really it's just a
normal image. In fact, you can use `gfx-hal` to render into an image, then keep
that image around and use it to texture your models.

A picture has "pixels", and sometimes you'll hear about a texture having
"texels". Just a way that some people distinguish between images intended for
final use and images intended for placement onto a model. The thing that's the
most special about textures is that since X, Y, and Z are already being used for
3D spatial positioning of a vertex, the position within a texture that it maps
to is called U and V. This is called [UV
Mapping](https://en.wikipedia.org/wiki/UV_mapping) and it can get very
complicated if you have a single texture being wrapped around a 3D model.

As always, each stage of this is hard enough already, so we'll keep it simple.
This time out we're going to place a texture onto a "Quad" (two triangles
oriented to make a quadrilateral). Like before, part of the quad will follow the
mouse so that we can see it stretch around and even flip backwards when the
mouse moves "behind" the start of the quad.

What picture? Well I've drawn a pic of a friendly water pal in MS Paint, just
for this occasion. Here's a quarter-size sample:

![creature-smol](images/creature-smol.png)

# Making A Quad

So instead of having a `Triangle` type, we're going to have a `Quad` type. What
makes up a quad? Of course it's four points instead of three.

```rust
#[derive(Debug, Clone, Copy)]
pub struct Quad {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
}
```

So if we have four "real" points, and we want to make two triangles... well we
need 3 points per triangle... We could just list out some of the points more
than once (scrub mode) or we could get fancy in how we tell the GPU to do it and
kick it up to a technique called "Indexed Drawing" (cool mode). The details of
that will be covered in a moment, right now we need to have a method to turn a
quad into some vertex data.

```rust
impl Quad {
  pub fn vertex_attributes(self) -> [f32; 4 * (2 + 3 + 2)] {
    let x = self.x;
    let y = self.y;
    let w = self.w;
    let h = self.h;
    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
    // X    Y    R    G    B                  U    V
      x  , y+h, 1.0, 0.0, 0.0, /* red     */ 0.0, 1.0, /* bottom left */
      x  , y  , 0.0, 1.0, 0.0, /* green   */ 0.0, 0.0, /* top left */
      x+w, y  , 0.0, 0.0, 1.0, /* blue    */ 1.0, 0.0, /* bottom right */
      x+w, y+h, 1.0, 0.0, 1.0, /* magenta */ 1.0, 1.0, /* top right */
    ]
  }
}
```

As you can see, we're approaching the limit of being able to specify it all as a
flat array. In future lessons we'll talk about having a proper Vertex type and
giving it fields so that it's easier to tell what parts are what and such. For a
single quad it's probably okay to do it like this.

So each vertex will have an XY position like before, and an RGB color like
before, and now we're adding a UV texture coordinate as well. We'll also have to
change around our pipeline setup to allow for the new vertex attribute.

Texture positions are always stored as 0.0 to 1.0 within the texture, U goes
horizontal (like X) and V is vertical (like Y). Within `gfx-hal`, the (0.0, 0.0)
position for UV coordinates is the **top left corner** of the image. Even if the
backend would normally use some other system, `gfx-hal` does the translations
necessary so that (0.0, 0.0) is the top left.

Note that some other graphics systems (mostly OpenGL) put the texture origin at
the _bottom_ left instead! If you're trying out some shader code samples from
some other place and your images come out unexpectedly upside down, that's why.
You can compensate by flipping the image data before you upload it (I'll mention
that in a moment), or you can flip the computed coordinate before looking up the
data in the texture by using `1.0-V` instead of using `V` directly.

# Indexed Drawing

Indexed drawing is a way to save on vertex space by specifying the minimum
number of vertices in just any order within an array, and then also specifying
indexes into that array to describe the triangles themselves.

That might sound silly, at first. We save a little space on the vertex data that
we didn't specify twice, but then we have to give all the indexes, so are we
really saving much? Let's check.

Say we have 28 bytes per vertex (7 floats * 4 bytes each), and also that indexes
are given as `u16` values:

* If there's a Quad:
  * We reduce the vertex data from 6 to 4 (56 bytes saved)
  * We need to spend 6 indexes to describe the triangles (12 bytes used)
  * Net savings of 44 bytes per quad (56-12)
* If there's a Cube:
  * We reduce the vertex data from 36 to 8 (784 bytes saved)
  * We need to spend 36 indexes to describe the triangles (72 bytes used)
  * Net savings of 712 bytes per cube (784-72)
* As the model shape gets more complex, causing more triangles to share the same
  vertex, the overall savings _improve_.

So, yeah, that's totally sweet.

## Making A `BufferBundle` Type

First of all, now that we're having more than one buffer, we want to take that
buffer creation (declare buffer, check requirements, get memory, bind memory)
and pack it into its own thing. We'll call it a `BufferBundle`, because that
seems like a good enough name for a really generic sort of thing that we don't
even fully know how we'll use in the future.

The struct for it is very simple. We can even make it generic over the `Backend`
trait for maximum angle brackets in our code. (Rust is always better with more
angle brackets in the types, right?)

```rust
pub struct BufferBundle<B: Backend, D: Device<B>> {
  pub buffer: ManuallyDrop<B::Buffer>,
  pub requirements: Requirements,
  pub memory: ManuallyDrop<B::Memory>,
  pub phantom: PhantomData<D>,
}
```

We'll make all the fields be `pub`, because (hot take) that's honestly the
better default for fields, unless you're trying to maintain some invariants with
the type. The `BufferBundle` isn't smart enough to have any invariants.

So we've got it generic over `Backend`, and then our methods will be using a
particular `Device`, and it'd be slightly insane to try and use a buffer between
two different device implementations, so we'll throw in a 👻
[PhantomData](https://doc.rust-lang.org/core/marker/struct.PhantomData.html) 👻
so that things know we had a particular device in mind when we made the buffer.
Is there anything that PhantomData can't solve? I sure hope not. 👻

Do we want this type to have any methods? Yeah, obviously, we want to be able to
make new ones. We'll just cut that code for making the vertex buffer and then
make it a little more buffer agnostic and reusable.

```rust
impl<B: Backend, D: Device<B>> BufferBundle<B, D> {
  pub fn new(adapter: &Adapter<B>, device: &D, size: usize, usage: BufferUsage) -> Result<Self, &'static str> {
    unsafe {
      let mut buffer = device
        .create_buffer(size as u64, usage)
        .map_err(|_| "Couldn't create a buffer!")?;
      let requirements = device.get_buffer_requirements(&buffer);
      let memory_type_id = adapter
        .physical_device
        .memory_properties()
        .memory_types
        .iter()
        .enumerate()
        .find(|&(id, memory_type)| {
          requirements.type_mask & (1 << id) != 0 && memory_type.properties.contains(Properties::CPU_VISIBLE)
        })
        .map(|(id, _)| MemoryTypeId(id))
        .ok_or("Couldn't find a memory type to support the buffer!")?;
      let memory = device
        .allocate_memory(memory_type_id, requirements.size)
        .map_err(|_| "Couldn't allocate buffer memory!")?;
      device
        .bind_buffer_memory(&memory, 0, &mut buffer)
        .map_err(|_| "Couldn't bind the buffer memory!")?;
      Ok(Self {
        buffer: ManuallyDrop::new(buffer),
        requirements,
        memory: ManuallyDrop::new(memory),
        phantom: PhantomData,
      })
    }
  }
```

Also we want to be able to throw them away when we're done. Question: Do we want
it to be `Drop`? Mmmm, no. But `HalState` is `Drop`, why not this too? Well,
`HalState` gets to be `Drop` because it's holding the `Device` field that's
needed to destroy all the other stuff it has. A `BufferBundle` has a PhantomData
for a device thing, but it isn't holding an _actual_ `Device`, so it can't
perform a `Drop` on its own. _Should_ it hold an actual device reference? I
think not. That'd make it really hard to store in our `HalState` alongside the
device field. The lifetimes would go crazy. So we'll just make a method to
`manually_drop` the type, and then it'll do the thing.

```rust
  pub unsafe fn manually_drop(&self, device: &D) {
    use core::ptr::read;
    device.destroy_buffer(ManuallyDrop::into_inner(read(&self.buffer)));
    device.free_memory(ManuallyDrop::into_inner(read(&self.memory)));
  }
}
```

## Adding `BufferBundle` To `HalState`

So now `HalState` wants two fields like this:

```rust
  vertices: BufferBundle<back::Backend, back::Device>,
  indexes: BufferBundle<back::Backend, back::Device>,
```

Lokathor, why did we make BufferBundle be all generic and not have HalState be
all generic?

Because I tried that at first and doing the whole `HalState` generic gave me
some trouble at the time, so I just gave up on it. Obviously.

Creating these buffers is pretty easy:

```rust
    const F32_XY_RGB_UV_QUAD: usize = size_of::<f32>() * (2 + 3 + 2) * 4;
    let vertices = BufferBundle::new(&adapter, &device, F32_XY_RGB_UV_QUAD, BufferUsage::VERTEX)?;

    const U16_QUAD_INDICES: usize = size_of::<u16>() * 2 * 3;
    let indexes = BufferBundle::new(&adapter, &device, U16_QUAD_INDICES, BufferUsage::INDEX)?;
```

And once we have an index buffer we can fill it up just once as part of our
`HalState` startup. Even if our quad changes from frame to frame, the indexes
don't, so we won't have to re-upload them each frame (the savings don't stop!)

```rust
    // Write the index data just once.
    unsafe {
      let mut data_target = device
        .acquire_mapping_writer(&indexes.memory, 0..indexes.requirements.size)
        .map_err(|_| "Failed to acquire an index buffer mapping writer!")?;
      const INDEX_DATA: &[u16] = &[0, 1, 2, 2, 3, 0];
      data_target[..INDEX_DATA.len()].copy_from_slice(&INDEX_DATA);
      device
        .release_mapping_writer(data_target)
        .map_err(|_| "Couldn't release the index buffer mapping writer!")?;
    }
```

This is the exact same idea as writing to the vertex buffer, so it should look
very familiar. Do we want to make a `write_stuff` method on the `BufferBundle`
type and capture this pattern? Hmmmmmm, maybe later. I don't think it'd be hard,
but it's not really our goal right now.

## Performing Indexed Drawing

When we're doing our command buffer encoding we do it just a little different.

Now we have to bind an index buffer (there's _just one_ index buffer per draw
call, even if there's more than one vertex buffer being combined)

```rust
        encoder.bind_index_buffer(IndexBufferView {
          buffer: &self.indexes.buffer,
          offset: 0,
          index_type: IndexType::U16,
        });
```

And then instead of calling `draw` with a vertex range, offset, and instance
range, we call `draw_indexed` with an index range, offset, and instance range.

```rust
        encoder.draw_indexed(0..6, 0, 0..1);
```

Like I said, it's only _slightly_ different.

# Adding A Vertex Attribute For Texture Positions

# Loading An Image

# Shading The Image Onto The Quad
