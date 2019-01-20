# Shaders

I know you've been itching for a short lesson, so let's do a short lesson.

This time we're talking more about Shaders? [What are
shaders?](https://www.youtube.com/watch?v=Kh0Y2hVe_bw) We just don't know.

# GLSL and SPIRV

Technically you can use any number of shader languages with `gfx-hal` as long as
they can compile to SPIRV, but for now we're using a slightly special variant of
GLSL. The `shaderc-rs` crate that we're using compiles the textual GLSL format
into the binary SPIRV format. The exact process is that the `shaderc-rs` crate's
`build.rs` file downloads the source of the `shaderc` C++ program and builds
that, then puts those binaries in your `target/` directory, then when you call
the crate it invokes that program to do the actual compilation. If you think
that's crazy and silly you're right. People are working on better solutions,
[top men](https://www.youtube.com/watch?v=yoy4_h7Pb3M), I assure you, but until
then this is the best system we've got.

Now there's a whole lot that can be said about GLSL. You can seriously write
[books](https://thebookofshaders.com/) and [blogs](http://www.iquilezles.org/)
and [demo site](https://www.shadertoy.com/) after [demo
site](https://www.vertexshaderart.com/) after [demo
site](http://glslsandbox.com/) after [demo
site](https://www.interactiveshaderformat.com/) for just GLSL. It's well out of
scope of this lesson or even this entire tutorial to try and cover it all.
Seriously you should read that book stuff and blog stuff and anything else you
can about GLSL if you really want to know it all.

What we're doing here is an _introduction_ to the GLSL that we'll be using with
`gfx-hal` (which is basically normal GLSL, but with a few things you have to be
more clear about).

## Format

The first line of all your shaders is just going to be `#version 450`, and that's it.

There's technically a whole lot of GLSL versions, because each release of OpenGL
has a GLSL that goes with it, and you can try to be compatible with lots of
OpenGL versions by having different shaders and stuff, but since we're not
_really_ using OpenGL, we just treat it like we're using version 450.

## Inputs and Outputs

Next you generally want to specify your inputs and outputs.

In normal GLSL you don't have to specify a layout value for each input and
output, but for the GLSL that we want to compile to SPIRV you are _required_ to
give a layout location for each. The general format is

```glsl
layout (location = INDEX) DIRECTION TYPE NAME;
```

* The `INDEX` values are just integer values. They're basically arbitrary, but
  your Rust code and GLSL code must **all** agree on whatever you pick.
  * With a Vertex shader, the `AttributeDesc` determines the locations for
    passing CPU side data into into GLSL data at the start of the process.
  * With a Fragment shader, the `SubpassDesc` determines the locations for
    fragment outputs becoming framebuffer data at the end of the process.
  * _Between_ shaders the locations and variable outputs from one stage need to
    match the locations and names of the next stage any time you want to pass
    data between rendering stages.
* The `DIRECTION` is a keyword: one of `in`, `out`, or `uniform`. Technically
  there are other things you could put here but they're aliases for one of those
  three so let's stick to the basics. An `in` value comes from a previous stage
  (or the vertex buffer data, for the Vertex Shader), and an `out` value goes to
  the next stage (or the framebuffer, for the Fragment Shader). A `uniform` is a
  special kind of read-only value that we'll get to a little farther down the page.
* The `TYPE` is a variable type, using C style names, so it's stuff like
  `float`, `int`, and `uint`, not `f32`, `i32`, and `u32`. There's also `vecN`
  where N is 2, 3, or 4 if you want a float vector, and you can have integer
  vectors and such as well. You can even declare structs using the C style where
  it's `StructName { type1 field1; type2 field2; ... }`. We'll use the struct
  style before this lesson is over.
* The `NAME` is just a name for the variable.

In GLSL you'd normally have access to a few magical values that you can read and
write from, but not all of that translates cleanly with the `shaderc` compiler.
In our case, the thing that we need to be the most aware of is that instead of
writing to a magical `gl_Position` value during the vertex shader, we need to
declare and use a special output like this:

```glsl
layout (location = 0) out gl_PerVertex {
  vec4 gl_Position;
};
```

## Functions

Your GLSL can have any number of functions that you like, declared in the C
style where the output is on the left, then the function names and arguments.

At minimum each shader needs an "entry point", as you may recall from the
pipeline declaration we did. By tradition it's just called `main`, and that's
probably good enough so we'll go with that in our shaders.

# Adding A Vertex Attribute For Color

So what's any of that mean for us? Well first of all we can add a color
attribute to our vertex data.

## Update Shader Code

First we update our shaders to use the new color attribute. A full color output
is of course RGBA (vec4), but as input we'll just give RGB and assume that
A=1.0 within the fragment shader.

So our _input_ locations are 0 (position) and 1 (color), out _output_ locations
are also 0 (the magical gl_Position) and 1 (frag_color for the fragment shader).
The fact that the position information is location 0 for both inputs and outputs
isn't special, you could swap it around if you wanted. Like with the previous
lesson, our fragment shader just promotes the 2D input into a basic 3D output.
It feel a little silly to write `frag_color = color;`, but we just have to
include that line to make the data pass on to the next stage properly.

```rust
pub const VERTEX_SOURCE: &str = "#version 450
layout (location = 0) in vec2 position;
layout (location = 1) in vec3 color;

layout (location = 0) out gl_PerVertex {
  vec4 gl_Position;
};
layout (location = 1) out vec3 frag_color;

void main()
{
  gl_Position = vec4(position, 0.0, 1.0);
  frag_color = color;
}";
```

For our fragment shader, we accept a single input, with the _exact same name and
location_ as the output from the vertex shader. This means that we don't have a
location 0 input for our fragment shader, and that's fine. Our output is a
location 0 vec4, so that'll become the color buffer output, so we'll sensibly
call it `color`. We just promote the RGB color into an RGBA color by giving it
an alpha of 1.0 ("fully opaque").

```rust
pub const FRAGMENT_SOURCE: &str = "#version 450
layout (location = 1) in vec3 frag_color;

layout (location = 0) out vec4 color;

void main()
{
  color = vec4(frag_color,1.0);
}";
```

## Make Triangle Give New Data

Next we'll add a method to our `Triangle` type so that it gives positions
interleaved with color data at the right time.

```rust
  pub fn vertex_attributes(self) -> [f32; 3 * (2 + 3)] {
    let [[a, b], [c, d], [e, f]] = self.points;
    [
      a, b, 1.0, 0.0, 0.0, // red
      c, d, 0.0, 1.0, 0.0, // green
      e, f, 0.0, 0.0, 1.0, // blue
    ]
  }
```

## Create A Pipeline With More Attributes

Now we want to support the color attribute in our `create_pipeline` function.
That's pretty easy, we just change how the the `vertex_buffer` and `attributes`
values are defined.

```rust
      let vertex_buffers: Vec<VertexBufferDesc> = vec![VertexBufferDesc {
        binding: 0,
        stride: (size_of::<f32>() * 5) as ElemStride,
        rate: 0,
      }];
      let position_attribute = AttributeDesc {
        location: 0,
        binding: 0,
        element: Element {
          format: Format::Rg32Float,
          offset: 0,
        },
      };
      let color_attribute = AttributeDesc {
        location: 1,
        binding: 0,
        element: Element {
          format: Format::Rgb32Float,
          offset: (size_of::<f32>() * 2) as ElemOffset,
        },
      };
      let attributes: Vec<AttributeDesc> = vec![position_attribute, color_attribute];
```

## Colors!

That's pretty much it. Didn't I promise that _eventually_ it'd get easier to
enhance the program? That was pretty easy.

So easy... that we'll keep going and add a little bit more to the lesson.

# Push Constants

Last lesson I said that most of the time you don't re-upload vertex data every
frame. That's because usually you'd have a single model (an "iconic triangle")
and then you'd tell the shader what animation frame, or position, or global
light level, or whatever else without touching the model data directly. It
doesn't seem like a big difference right now when there's only 3 vertex entries
in one triangle, but if there's _thousands_ of vertex entries, and there's
_tens_ of copies of that model that have to show up in the scene, well you'd
rather be doing all that math on your GPU (with dozens of ALUs) than on your CPU
(with only a handful of ALUs). That's like, the whole _point_ of the GPU after
all.

So how do we know about these special global values during a draw call? They get
placed into things called _uniforms_, that's what the `uniform` keyword is for
in GLSL. When GLSL was used for OpenGL there were just "uniforms", but with the
introduction of Vulkan now we've got both "Uniform Buffers" (like the old
uniforms) and also "Push Constants" (a fancy new thing). Uniforms get set before
a draw call and then they're a fixed, read-only value for that entire draw call.
No changes per-vertex or per-fragment or anything else. Any shader can access
that uniform, if it's been correctly configured in your pipeline setup.

As I said, push constants are newer, so older 3D books might not mention them if
you pick one up for some light technical reading, but they work the same way as
a uniform buffer. The main difference is that push constants are somewhat easier
to use, since there's no extra buffer memory to fiddle with, but you only get a
_very_ limited amount of push constant space. With gfx-hal you can only use 128
**bytes** of push constant space. The Vulkan spec assures you that you have at
least that much, and many cards offer more these days, but currently gfx-hal has
no way to ask the graphics card exactly how much it supports. Here's hoping for
a fix in 0.2.

As a demo of how to use push constants, we'll record a `std::time::Instant` at
the creation of our `HalState` and then use the time since that instant to shift
our triangle towards black.

## Add It To Our `HalState`

We first add an
[Instant](https://doc.rust-lang.org/std/time/struct.Instant.html) to our
`HalState`.

Before we record the command buffer, we'll decide the current time value as an `f32`.

```rust
    // DETERMINE THE TIME DATA
    let duration = Instant::now().duration_since(self.creation_instant);
    let time_f32 = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;
```

And just after we bind the vertex buffer, we also push the graphics constant:

```rust
        encoder.bind_vertex_buffers(0, buffers);
        encoder.push_graphics_constants(&self.pipeline_layout, ShaderStageFlags::FRAGMENT, 0, &[time_f32.to_bits()]);
```

The important part is that we have to remember to adjust our pipeline definition as well.

```rust
      let push_constants = vec![(ShaderStageFlags::FRAGMENT, 0..1)];
      let layout = unsafe {
        device
          .create_pipeline_layout(&descriptor_set_layouts, push_constants)
          .map_err(|_| "Couldn't create a pipeline layout")?
      };
```

## Add It To The Shader Code

Adding the push constants to a shader is pretty easy, but there's a few rules.
All of your push constants appear in a single block with the special layout
value of `push_constant`. This block isn't `in` or `out`, instead it's
`uniform`. After that you give a name for the block type, the block itself, and then
the name that we're going to access it under. If you haven't programmed in C
before this might seem weird, but they think it's normal. We just have to go
with it.

Since we want a single `f32` to be the time, we define it as a block that holds
a single `float` value (the GLSL equivalent) which we'll call `time`. Then we
take `push.time` (think of it like they're all stored within a global `push`
struct), do some funny math on that so that it ends up as a 0.0 to 1.0 value
(since color channels are supposed to be in that range), and make a vec4. We can
multiply the vec4 from our time with the vec4 for our `frag_color`, and it does
a component-wise multiplication (a.x * b.x, a.y * b.y, etc). Then we shove the
result out the door. Bam, now our triangle shifts between black and rainbow.

```rust
pub const FRAGMENT_SOURCE: &str = "#version 450
layout (push_constant) uniform PushConsts {
  float time;
} push;

layout (location = 1) in vec3 frag_color;

layout (location = 0) out vec4 color;

void main()
{
  float time01 = -0.9 * abs(sin(push.time * 0.9)) + 0.9;
  color = vec4(frag_color,1.0) * vec4(time01,time01,time01,1.0);
}";
```

# Uniform Buffers

Like I said, there's a harsh limit on your push constant space. If you want more
global data than you can fit into your push constants you need to setup a
Uniform Buffer.

However, I promised to keep this lesson short, and we'll be using uniform,
buffers for Textures during the next lesson, so we _won't_ go into them right
now. Knowing about push constants already teaches you about the general idea of
uniform data, so we've covered enough to stop here.
