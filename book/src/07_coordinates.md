# Coordinates

So, when we did the Quad in the textures lesson, you may recall that it was
(x,y) for the base corner, and then width and height for the size, and then we
used the base corner plus the width and height to find the other three corners.
One of the non-base corners added the width to x, one added the height to y, and
one did both of the adds at the same time. How do we know that `x` and `width`
go together? Why doesn't `x` go with `height`? That's just kinda the convention
that you learn in school. `x` is horizontal with bigger values going to the
right, and `y` is vertical with bigger values going up. This is part of the
[Cartesian coordinate
system](https://en.wikipedia.org/wiki/Cartesian_coordinate_system). You might
not remember the name, but you probably remember the whole thing with `x` going
horizontally and `y` going vertically.

Here's the deal though. We're not aiming for just 2D drawing. We're aiming for
3D drawing and having 2D be just an occasional specialization of the whole 3D
process. So where does the `z` direction go? We probably drew that in our little
notebooks. It's hard to draw. On top of that we've got those `u` and `v` values
for texture lookups, and those don't have the origin in the middle of the space,
they have it in the top left corner of the texture. What's going on with any of
this nonsense?

## Coordinate Systems

Turns out there's not just one "coordinate system", there's many coordinate
system**s**, that we have to deal with. Textures have a coordinate system, the
screen has a coordinate system, the scene has a coordinate system, and even each
individual model has a coordinate system. What?

Yeah, because floating point numbers have limited accuracy across big ranges,
the convention is to have all the points of a single model be in "model space"
with relatively small numbers and high accuracy. Then you have a transform
function that converts model space positions into world space positions based on
the overall position of the model in the world. Then the whole scene is being
viewed from some particular place and with a particular perspective, so there's
a transformation function to turn world space coordinates into screen space
coordinates. Any coordinate out of bounds of the screen space is off-screen, so
we don't even draw it.

## Transformations

So what's this "function" that shifts points between coordinate spaces? Well, I
kinda made it sound like a code function, but it's actually a math function
(`math`, _singular_, this is not a plural "maths" tutorial that's silly).

To translate a 3D point, we make the point into a `vec4` with the final position
as `1.0`, then we multiply it by a specially prepared 4x4 matrix (the
"transformation matrix"). That gives us a `vec4` output, and then to turn it
back into a `vec3` we just divide the `x`, `y`, and `z` axis by the `w` axis.

Vectors? Matrix? `w`-axis? What?

Ho boy are you in for some fun!

Yes, this (and more) is among the powers that math can grant you, but first you
must learn that math.

## Learning The Math

I'm not a math teacher, and math is universal enough that I can just tell you to
go learn from someone else's math course and you'll be able to use those skills
here, so that's exactly what I'll do.

* If you want _just_ the fundamentals right now you can read the
  [Transformations](https://learnopengl.com/Getting-started/Transformations)
  lesson on `LearnOpenGL.com`. Everything before the `In practice` section is
  totally code free, just a math lesson, so you don't need to have any previous
  OpenGL or C++ experience.

* When you have the time you should really sit down and learn the subject a
  little more properly there is a [Khan Academy Linear
  Algebra](https://www.khanacademy.org/math/linear-algebra) course, it's a totally
  free video series style

## Applying The Math

So now that we know about how vectors and matrices work, how do we do this in
our code? Well, there's three main options here:

* [nalgebra-glm](https://docs.rs/nalgebra-glm) is a crate that provides a
  GLM-like interface to the [nalgebra](https://docs.rs/nalgebra) crate via type
  aliases and such. `nalgebra` is a serious math crate for serious math people.
  I generally wouldn't suggest that you use this crate _unless_ you need to also
  use the `ncollide` crate (which is a collision system based on nalgebra). The
  extreme amount of generics makes error messages far worse when there's a type
  error, and it also makes your compiles take longer.
* [vek](https://docs.rs/vek) is the up and coming swiss army math lib that plans
  to have an emphasis on SIMD support and is `#![no_std]`. I _always_ approve of
  a lib going for the `no_std` treatment. There's also a whole lot of features
  you can enable to get extra benefits.
* [cgmath](https://docs.rs/cgmath) is the "tried and true beginner's crate" for
  graphics math. It's specifically their mandate to keep the focus on computer
  graphics. Back in the day this set them apart from `nalgebra` all on its own,
  but now that `vek` is coming up fast I'm not sure that `cgmath` has enough to
  set itself apart.

What should we use? Honestly, if it were just me I'd probably use `vek` to get
off the ground and then write my own vector math lib with no generics at all
when I wanted to take it easy during some programming day. Seriously, there's
only so much code involved in a vec math lib, you can totally write your own
from scratch.

However, this project isn't really about me, it's about you, the reader.
Accordingly, we're going to be using the `nalgebra-glm` crate. Out of all the
options, it's definitely got the worst error messages when things go wrong, so
any other crate that you decide to use instead will seem like a breeze in
comparison if you switch to another crate.

```toml
[dependencies]
...
nalgebra-glm = "0.2"
```

# The Primary Coordinate Systems

There's actually an unlimited number of possible coordinate systems, but let's
focus on a few of them that you're most likely to encounter.

## Spatial Coordinates

A lot of the time we're concerned with 3D spatial positioning.

### Model Space

Each individual model exists in its own "model space". A model can be anything
that's got all of its vertex positions specified in the same space. We'll be
using some basic shapes to start, and later on we'll learn how to load model
data out of a file.

The important thing about model space is that it's totally arbitrary and unique
to each model. You need to decide for yourself what your units are.

### World Space

By convention, each model within the scene has a transformation that converts
its model space points into world space points. This lets all of the models
exist in a single, unified coordinate space that's easier to think about.

As with model space, it's actually fairly arbitrary as to what your scale is.
The benefit of a world space is that you're usually doing not only the graphics,
but also any physics and such within the world space scale. It unifies the
whole simulation to get things into a space with a single origin.

### View Space

Graphics only happens from a particular point of observation. Transforming World
Space coordinates into how they should appear relative to the observer puts them
in "View Space".

Once things are in View Space we can apply a Projection to the view. There's two
main projections to pick from:

* Orthographic Projection makes parallel lines stay parallel as they move far
  away from you. Things are more angular, and even a little unreal looking
  because of it. You probably want this projection for "artificial" sorts of
  scenes, like if the user is designing something, or if the user is looking
  over the scene in an "all knowing" sort of way and the scene is more like a
  game board, like The Sims or Civilization.
* Perspective Projection makes parallel lines appear to meet the farther away
  they go from you. Like when looking far down a highway stretching out ahead.
  This is basically how graphics work in the "Real Life" game, and that's a
  fairly popular one that people have really become used to. You probably want
  this projection if the scene is something that is being observed from some
  sort of "real" perspective (either 1st person or 3rd person).

The important thing here is that _the output is no longer arbitrary_. Once
you've run your projection matrix has transformed the vertex and the vertex
shader spits that value out, it has to be in what's called "Normalized Device
Coordinates". For `gfx-hal` this means:

* X: -1.0 to +1.0 range, with +X going to the right
* Y: -1.0 to +1.0 range, with +Y going to the down
* Z: 0.0 to 1.0 range, with +Z going deeper into the screen

## Texture Coordinates

In addition to "physical" locations, there's also texture lookups.

With textures the convention is to call the directions `u` and `v`, with `u`
being horizontal and `v` being vertical.

* U: 0.0 to 1.0, +U goes right
* V: 0.0 to 1.0, +V goes down

# NOTES

perspective is your cameras's lense

view is the position of your camera

model is where you set the object in front of your camera
