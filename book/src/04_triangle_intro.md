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
