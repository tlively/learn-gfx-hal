# Triangle Intro

Hey, you're back.

This lesson builds upon the last one. Before we could draw a clear frame, now
we'll add the ability to draw a frame with a single triangle in it.

## Usage Code

Once again, even baby steps in functionality will demand pages and pages of work
to get arranged properly.

What we're going to write in this lesson is a single public method so that we
can draw a single triangle as a displayed frame. For now we'll stick to just
_one_ triangle (three points), and even then, only a 2D triangle of `(x,y)`
points.

```rust
pub struct Triangle {
  points: [[f32; 2]; 3]
}
```

Why only 2D? Unfortunately, without the help of camera perspective, lightning,
shading, and other effects like that, 3D things just don't show up very well on
a 2D screen. Instead of looking like a normal triangle at an angle, it just
looks like a slightly differently shaped triangle, but still totally flat. So
when we finally transmit the triangle to the GPU we'll simply give all three
points an identical `z` coordinate for now.

To have some sort of confirmation of input and output like before we'll have one
of the triangle points follow the user's mouse movements. Nothing fancy, just a
way to see that we're continually  drawing a new thing each time. Actually
passing in the triangle to draw is basically identical to the clear color
function:

```rust
impl HalState {
  pub fn draw_triangle_frame(&mut self, triangle: Triangle) -> Result<(), &'static str> {
    unimplemented!()
  }
}
```

The ability to draw exactly one triangle isn't very useful on its own. The
`draw_clear_frame` we could potentially use in the future (during a brief
loading screen or something), but a method to draw one triangle doesn't have
much long term use. In fact we will probably remove the method entirely in later
lessons as our program evolves, rather than try to keep such a useless method
updated through the changes we add from lesson to lesson.

Why add a thing only to take it away? Because demanding of ourselves to draw a
single triangle, of any quality, forces us to put in to place many more parts of
our overall "render pipeline". The rendering pipeline is what's here to stay. A
**complete** rendering pipeline is _even more complex_ than a complete
Swapchain. It's a many lesson long process. We've seen only a hint of it in the
last lesson. We'll add some more in this lesson. We'll expand it along in future
lessons.

One might argue that the entire field of 3D programming is just an unending
process of learning more and more about how you can twist the rendering pipeline
to do exactly what you want, when you want, as fast as possible.

### Terminology Sidebar: Immediate vs Retained

As we go further I should probably define two terms you might see come up here
or in other graphics tutorials: Immediate API and Retained API.

* An immediate API is any API where you call a function with an argument and it
  does all the work with that argument right then, without storing the argument
  data for later.
* A retained API is any API where your function calls cause data to be
  _retained_ by the system. Usually you make some calls to set up the situation,
  and then you make a separate call to compute things using the requested setup.

In general, an immediate API is often easier to use, but a retained API is often
more efficient if the input format and usage format differ (so you don't have to
convert more than once) or if the system needs special resources (heap
allocation, open file handles, things like that).

## Quick Bug Fixes

There's two things we have to change about last lesson's code before we proceed
to mostly work on new code.

### That Swapchain Is Too Big!

On the Metal backend (mac os) the extent that's reported in the swapchain
capabilities isn't clamped to the window size, so you get a reported maximum
size of 4096 x 4096. Obviously that's far too big! It doesn't matter for just
clearing the screen, but it matters now that we'll be drawing something.

We just have to edit how we define the extent as we create our Swapchain:

```rust
let extent = {
  let window_client_area = window.get_inner_size().ok_or("Window doesn't exist!")?;
  Extent2D {
    width: caps.extents.end.width.min(window_client_area.width as u32),
    height: caps.extents.end.height.min(window_client_area.height as u32),
  }
};
```

### The Swapchain Doesn't Resize!

The window can resize, but the backing swapchain doesn't resize. Again, this
isn't apparent when you're drawing nothing, but once you draw something it'll be
drawing at the starting resolution and then scaling up or down to the window's
real size.

Now, you _could_ try to carefully destroy anything that came from the Swapchain
and then the Swapchain itself and then re-create each element at the new size.
You could, it'd work.

Why bother being so fiddly though? We've gone to all the work of making our
`HalState` type very cleanly close itself down. Let's take advantage of that and
just throw out the _entire_ old `HalState` and build a new one. We don't have to
think about what the ordering of anything is, we don't have to remember to
update the change_resolution code every time we touch some other part of the
code. It's really so much less error prone. "Just restart the whole thing" is
how you get that magical [Nine Nines
Stability](https://en.wikipedia.org/wiki/Erlang_(programming_language)), after
all ;3

```rust
if inputs.new_frame_size.is_some() {
  drop(hal_state);
  hal_state = match HalState::new(&winit_state.window) {
    Ok(state) => state,
    Err(e) => panic!(e),
  };
}
```

# Drawing A Triangle

To draw a triangle, we will use the same sort of setup before, with the frame
based drawing and the "ring buffer" vectors of all our tools. Literally just
copy and paste all of `draw_clear_frame` to a new spot and name it
`draw_triangle_frame`, the bulk of it is that similar. The argument is a single
triangle instead of a single color though.

```rust
pub fn draw_triangle_frame(&mut self, triangle: Triangle) -> Result<(), &'static str> {
```

Now you'd think "hey can't we abstract the commonalities here? Well, maybe but
you can't really do it with a function and a closure because lifetimes and
function boarders don't play particularly nice in Rust. Our draw code
unfortunately really relies on having a lot of "split borrows" (where the borrow
is just on one field at a time) instead of struct-wide borrows (eg: `&self` or
`&mut self`). Or you could do it as a macro maybe? Either way it'd be probably
quite a bit of work for not too much gained. We don't want to over abstract
until we see how the code is growing.

All that changes is that instead of starting a CommandBuffer and then recording
nothing at all, we'll actually record something this time.

```rust
    // RECORD COMMANDS
    unsafe {
      let buffer = &mut self.command_buffers[i_usize];
      const TRIANGLE_CLEAR: [ClearValue; 1] = [ClearValue::Color(ClearColor::Float([0.1, 0.2, 0.3, 1.0]))];
      buffer.begin(false);
      {
        let mut encoder = buffer.begin_render_pass_inline(
          &self.render_pass,
          &self.framebuffers[i_usize],
          self.render_area,
          TRIANGLE_CLEAR.iter(),
        );
        encoder.bind_graphics_pipeline(&self.graphics_pipeline);
        // Here we must force the Deref impl of ManuallyDrop to play nice.
        let buffer_ref: &<back::Backend as Backend>::Buffer = &self.buffer;
        let buffers: ArrayVec<[_; 1]> = [(buffer_ref, 0)].into();
        encoder.bind_vertex_buffers(0, buffers);
        encoder.draw(0..3, 0..1);
      }
      buffer.finish();
    }
```

This time out the mouse will control one of the triangle points instead of the
color, so we'll pick a fixed color for the clear color. Once we start the
"render pass inline" we're actually going to bind what we get back from that.
It's a
[RenderPassInlineEncoder](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.RenderPassInlineEncoder.html),
which is also
Deref<Target=[RenderSubpassCommon](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.RenderSubpassCommon.html)>,
and it gives us access to the operations of a particular render pass.

* [RenderSubpassCommon::bind_graphics_pipeline](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.RenderSubpassCommon.html#method.bind_graphics_pipeline)
  picks a particular graphics pipeline for the rendering of this subpass. You
  _can_ have more than one graphics pipeline, each with its own settings, if you
  want, though while we're starting out we only need one per program.
* [RenderSubpassCommon::bind_vertex_buffers](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.RenderSubpassCommon.html#method.bind_vertex_buffers)
  picks the vertex buffers to use for this subpass. The magical looking `0` here
  has to match up with the
  [VertexBufferDesc](https://docs.rs/gfx-hal/0.1.0/gfx_hal/pso/struct.VertexBufferDesc.html)
  that's specified as part of the graphics pipeline that you're using. We'll
  talk about the full graphics pipeline definition in a moment, but the thing to
  pay attention to right now is that you can have many buffers and you don't
  need to specify them all in a single bind call. You could give 3 starting at
  0, give 3 more starting at 3, etc. We only have one buffer, so we just need
  one bind call and we place it at the 0th index.
* [RenderSubpassCommon::draw](https://docs.rs/gfx-hal/0.1.0/gfx_hal/command/struct.RenderSubpassCommon.html#method.draw)
  uses Range properly, so those really are exclusive endings. This uses our
  three vertices (indexed 0, 1, 2) and a single instance (indexed 0). The
  instance thing has to do with a more advanced technique called "instanced
  drawing" where you can draw a particular setup many times as a single draw
  call, specifying parameters per instance. That'd be for something like drawing
  ten copies of the same tree model, each in their own position and orientation
  within the scene. There's a small price per draw call that you make, so if
  you're drawing "the same" thing many times with small variation it pays off to
  setup instanced drawing and make a single draw call with many instances
  specified. We'll cover all that more in a future lesson. For now we've just
  got a single triangle as part of a single instance.

That's all we gotta do!

# Define A Graphics Pipeline

TODO

# Define A Buffer For Vertex Data

TODO
