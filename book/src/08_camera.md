# Camera

Now that we've got everything in the right coordinate system, let's play around
a little bit with that.

As a review:

* The Model Matrix places the model into the world
* The View Matrix defines _where the camera is_
* The Projection Matrix defines _the lens of the camera_

So this lesson we'll play around with the View matrix and the Projection matrix
a tiny bit.

## Quick Patch

I've forgotten until now, but there's an important little bit you'll want to
know about on Windows. There's a special attribute that you can set on your
program to make it not have an attached console. This makes it so that if you
run the program from outside of a terminal it won't open up a dummy terminal in
the background.

`windows_subsystem = "windows"`

However, this also makes it so that the program can't do terminal output _even
if_ it was run from a terminal. Since we want to see terminal debug output stuff
in debug builds, we want to only activate this attribute in builds _without_
`debug_assertions`.

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

# Orthographic Projection

I mentioned it in passing before, but there's a second major category of
projection that you might sometimes use. We're currently using Perspective,
which makes things look "real". There's also Orthographic, which makes things
look more "tactical" looking. Like how SimCity, or Civilization, or whatever
your favorite example is.

Since our scene is a bunch of cubes floating around in space, the orthographic
projection is going to look kinda weird, but we'll slot it in there as an
option. When the users presses the Tab key it'll flip a bool to swap between the
two projections.

## Update `draw_cubes_frame`

First, we'll want to control the view and projection as part of our LocalState
now. That makes sense, in a game the camera position is more a part of the game
state than a part of the graphics driver.

We just adjust the function to accept a `view_projection` matrix that it's given
for the scene, and we'll just decide what the view and projection are before we
call here.

```rust
  pub fn draw_cubes_frame(
    &mut self, view_projection: &glm::TMat4<f32>, models: &[glm::TMat4<f32>],
  ) -> Result<(), &'static str> {
```

## Update `UserInput`

Now we have to track if the user wants us to swap the projection. First we add
another field to the inputs.

```rust
#[derive(Debug, Clone, Default)]
pub struct UserInput {
  pub end_requested: bool,
  pub new_frame_size: Option<(f64, f64)>,
  pub new_mouse_position: Option<(f64, f64)>,
  pub swap_projection: bool,
  pub seconds: f32,
}
```

Then we add another match case to our event polling:

```rust
      Event::WindowEvent {
        event:
          WindowEvent::KeyboardInput {
            input:
              KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Tab),
                ..
              },
            ..
          },
        ..
      } => {
        // Each time we see TAB we flip if a projection swap has been requested.
        // This will probably only happen once per frame anyway.
        output.swap_projection = !output.swap_projection;
      }
```

Ya get all that? It's pretty wordy, but that's just the `winit` way to say "Tab
was pressed": KeyboardInput + EventState::Pressed + VirtualKeyCode::Tab.

## Update `LocalState`

Now the `LocalState` will hold two different projection matrices: one for
perspective and one for orthographic. We'll flip which one we use with a bool.

```rust
#[derive(Debug, Clone)]
pub struct LocalState {
  pub frame_width: f64,
  pub frame_height: f64,
  pub mouse_x: f64,
  pub mouse_y: f64,
  pub cubes: Vec<glm::TMat4<f32>>,
  pub view: glm::TMat4<f32>,
  pub perspective_projection: glm::TMat4<f32>,
  pub orthographic_projection: glm::TMat4<f32>,
  pub is_orthographic: bool,
  pub spare_time: f32,
}
```

Which means we add a bit to our "update from user input" method:

```rust
    if input.swap_projection {
      self.is_orthographic = !self.is_orthographic;
    }
```

and now we need to initialize all the new data when we first make the LocalState
value. Once again, `nalgebra-glm` has many different `orthographic` projections
to pick from, and we want `_lh_zo`. This time instead of picking an aspect ratio
and view angle (plus near plane and far plane) we pick the left, right, bottom,
and top bounds of the view (plus near plane and far plane). The bounds are in
world coordinates, and I picked +/- 5.0 since our cubes are in that general
area. For your own code you'd need to decide on a comfortable value based on
your world scale and such.

```rust
    LocalState {
      frame_width,
      frame_height,
      mouse_x: 0.0,
      mouse_y: 0.0,
      cubes: vec![
        glm::identity(),
        glm::translate(&glm::identity(), &glm::make_vec3(&[1.5, 0.1, 0.0])),
        glm::translate(&glm::identity(), &glm::make_vec3(&[-3.0, 2.0, 3.0])),
        glm::translate(&glm::identity(), &glm::make_vec3(&[0.5, -4.0, 4.0])),
        glm::translate(&glm::identity(), &glm::make_vec3(&[-3.4, -2.3, 1.0])),
        glm::translate(&glm::identity(), &glm::make_vec3(&[-2.8, -0.7, 5.0])),
      ],
      spare_time: 0.0,
      view: glm::look_at_lh(
        &glm::make_vec3(&[0.0, 0.0, -5.0]),
        &glm::make_vec3(&[0.0, 0.0, 0.0]),
        &glm::make_vec3(&[0.0, 1.0, 0.0]).normalize(),
      ),
      perspective_projection: {
        let mut temp = glm::perspective_lh_zo(800.0 / 600.0, f32::to_radians(50.0), 0.1, 100.0);
        temp[(1, 1)] *= -1.0;
        temp
      },
      orthographic_projection: {
        let mut temp = glm::ortho_lh_zo(-5.0, 5.0, -5.0, 5.0, 0.1, 100.0);
        temp[(1, 1)] *= -1.0;
        temp
      },
      is_orthographic: false,
    }
```

And then in do_the_render we pick the right projection, combine it with our
view, and call `draw_cubes_frame`:

```rust
fn do_the_render(hal_state: &mut HalState, local_state: &LocalState) -> Result<(), &'static str> {
  let projection = if local_state.is_orthographic {
    local_state.orthographic_projection
  } else {
    local_state.perspective_projection
  };
  let view_projection = projection * local_state.view;
  hal_state.draw_cubes_frame(&view_projection, &local_state.cubes)
}
```

Now we can see how ugly an orthographic projection is!

Well, it's not always ugly, but we'd really need to have a different sort of
scene of stuff to look at if we wanted it to look good. Unfortunately that's a
little out of scope at the moment, so I'll leave trying that out up to you now
that you know what to do. Right now I just wanted you to know that it's
_possible_, and let you have a sense of why the `view` and `projection` matrix
data isn't always just a single matrix based on the camera position.

# Euler Angle Camera

Ultimately a camera is just about picking a **location** and **orientation** of
where you're looking at things from. However, there's actually two major types
of camera similar to how there's two major types of projection.

First we'll go over a camera that uses "[Euler
Angles](https://en.wikipedia.org/wiki/Euler_angles)", because it's a little
easier to think about. Euler angles means `pitch`, `roll`, and `yaw`. Like a
plane. This is often called an "FPS Camera" because it works like in a first
person game.

* `pitch`: angle up and down
* `roll`: angle rocking side to side
* `yaw`: angle left and right

Actually, I lied just now, we **won't** be handling `roll`. If you allow the
user to adjust their `roll` value as much as they want they can trigger a
[Gimbal Lock](https://en.wikipedia.org/wiki/Gimbal_lock). For most first person
experiences you don't need roll at all, so we'll block the user from
accidentally giving themselves problems.

Also, we'll limit the maximum `pitch` value to +/- 89 degrees. You remember that
`up` vector thing from the `look_at` projection? If the `pitch` is allowed to
hit 90 degrees then the `up` vector and the `front` vector line up and that's a
problem too. Again, this users are actually very used to the idea that they
can't just look up more and more until it's flipped over.

Can we avoid those limits? Yes, that's what the second camera style is for.
However, we'll do the simple one first because most of the time it's all you
need and it's easier to understand.

## EulerCamera Struct

So we need a _location_ and _orientation_. Our struct can hold exactly that:

```rust
#[derive(Debug, Clone, Copy)]
pub struct EulerCamera {
  pub position: glm::TVec3<f32>,
  pitch_deg: f32,
  yaw_deg: f32,
}
```

We've got a little extra note there that the pitch and yaw will be in degrees,
because degrees are usually easier for a human to think about, but the `sin` and
`cos` functions are for `radians`, so when we eventually call those we'll need a
conversion first.

Now we declare the "up" vector, which is always the same for this camera style.
Unfortunately, I'm not seeing a `const` function for making a TVec3 value, so
we'll declare the array for the data and then wrap it when we need to I guess.

```rust
impl EulerCamera {
  const UP: [f32; 3] = [0.0, 1.0, 0.0];
```

Next we want a "front" vector. This is a vector that points forward out of the
camera into the world. We're actually tracking our pitch and yaw as angles, but
we'll need the front vector for doing movement and computing the `look_at`
matrix. This involves some `sin` and `cos` calls, so we have to convert our
degree values into radian values.

(If we wanted we could cache this vector along side our angle values, but that's
not really necessary so we'll keep it simple.)

```rust
  fn make_front(&self) -> glm::TVec3<f32> {
    let pitch_rad = f32::to_radians(self.pitch_deg);
    let yaw_rad = f32::to_radians(self.yaw_deg);
    glm::make_vec3(&[
      yaw_rad.cos() * pitch_rad.cos(),
      pitch_rad.sin(),
      yaw_rad.sin() * pitch_rad.cos(),
    ])
  }
```

Orientation updates are pretty simple, but we have to be mindful of our limits.
We'll cap `pitch` at +/- 89 degrees, and we'll make sure that the `yaw` value
gets wrapped to being within +/- 360.0 degrees to avoid any potential weird
accuracy problems (remember that floats are more accurate the closer they are to
zero).

```rust
  pub fn update_orientation(&mut self, d_pitch_deg: f32, d_yaw_deg: f32) {
    self.pitch_deg = (self.pitch_deg + d_pitch_deg).max(-89.0).min(89.0);
    self.yaw_deg = (self.yaw_deg + d_yaw_deg) % 360.0;
  }
```

This is the part where, if you _were_ caching your front vector value, you'd
update your angles and then rebuild your front vector after each
`update_orientation` call.

Now we need a way to update the _position_ of the camera. We accept some keys
and then how far the camera was able to move (if it moved). The distance moved
is camera_speed * time_elapsed, but whoever calls `update_position` can just
compute that on their side before they call us.

The way that this works is that we gather up all the deltas that the keys are
trying to get us to do, normalize that total if it's non-zero, and then adjust
our position by that normalized vector times the distance.

```rust
  pub fn update_position(&mut self, keys: &HashSet<VirtualKeyCode>, distance: f32) {
    let up = glm::make_vec3(&Self::UP);
    let forward = self.make_front();
    let cross_normalized = glm::cross::<f32, glm::U3>(&forward, &up).normalize();
    let mut move_vector =
      keys
        .iter()
        .fold(glm::make_vec3(&[0.0, 0.0, 0.0]), |vec, key| match *key {
          VirtualKeyCode::W => vec + forward,
          VirtualKeyCode::S => vec - forward,
          VirtualKeyCode::A => vec + cross_normalized,
          VirtualKeyCode::D => vec - cross_normalized,
          VirtualKeyCode::E => vec + up,
          VirtualKeyCode::Q => vec - up,
          _ => vec,
        });
    if move_vector != glm::zero() {
      move_vector = move_vector.normalize();
      self.position += move_vector * distance;
    }
  }
```

I've implemented it as a "flying" camera. It uses the front vector for movement,
so if you look up while going forward then you also move up (depending on
pitch). I've also set `Q` and `E` to shift the camera directly up and down. If
that's not appropriate for your own program then you'd want to compute a forward
vector with just X and Z changes based on `yaw` alone. What you'd probably
actually want is to directly place the camera within the location given to you
by some physics object as it moves through the simulation, and just let the
physics system handle all the position updates. Just assigning to the position
field directly is fine, that's why it's `pub`.

(Note, the `A` and `D` math is sensitive to the fact that the projection matrix
is flipping `Y` values _after_ they pass through the View matrix. In other
words, if you port this code to OpenGL where `Y` is up naturally then you'll
need to flip which one is `+` and which one is `-`, otherwise you'll move
left/right flipped).

Finally, now that we can adjust the details on our camera, we just need to ask
it to please give us the correct view matrix.

```rust
  pub fn make_view_matrix(&self) -> glm::TMat4<f32> {
    glm::look_at_lh(
      &self.position,
      &(self.position + self.make_front()),
      &glm::make_vec3(&Self::UP),
    )
  }
```

Oh, and it needs at least one constructor because of those private fields. Let's
give it a const constructor for being at a particular position. Always nice to
have a const constructor if you can manage it.

```rust
  pub const fn at_position(position: glm::TVec3<f32>) -> Self {
    Self {
      position,
      pitch_deg: 0.0,
      yaw_deg: 0.0,
    }
  }
```

## Update `LocalState`

Now that we've got this nice camera we can replace out view matrix field with a
camera field.

```rust
#[derive(Debug, Clone)]
pub struct LocalState {
  pub frame_width: f64,
  pub frame_height: f64,
  pub mouse_x: f64,
  pub mouse_y: f64,
  pub cubes: Vec<glm::TMat4<f32>>,
  pub camera: EulerCamera,
  pub perspective_projection: glm::TMat4<f32>,
  pub orthographic_projection: glm::TMat4<f32>,
  pub is_orthographic: bool,
  pub spare_time: f32,
}
```

And in the LocalState initializer we need to place the camera at the same
position as before.

```rust
camera: EulerCamera::at_position(glm::make_vec3(&[0.0, 0.0, -5.0])),
```

which means that `do_the_render` needs a minor update to get a view matrix in
the new way.

```rust
fn do_the_render(hal_state: &mut HalState, local_state: &LocalState) -> Result<(), &'static str> {
  let projection = if local_state.is_orthographic {
    local_state.orthographic_projection
  } else {
    local_state.perspective_projection
  };
  let view_projection = projection * local_state.camera.make_view_matrix();
  hal_state.draw_cubes_frame(&view_projection, &local_state.cubes)
}
```

That finally brings us to `LocalState::update_from_input`.

This part is a little bit of a pickle. We're updating our "physics" by 1/60th of
a second every 1/60th of a second. However, we're accepting input faster than
that in some cases (if Mailbox mode is selected). Does the camera count as part
of our physics? Can we do some updates to it faster than 60fps and then others
at only 60fps? Should we buffer _all_ updates until the next physics frame and
only do them exactly when the rest of the physics happens? Well, unfortunately
that's an answer you'll need to sort out for yourself.

Our camera isn't _really_ connected to anything, but if your camera **is**
connected to an actual physics entity (like a player entity) then you'd probably
need to buffer up the inputs that come in faster than 60fps, do your physics at
the right time, and then update your camera only in response to the physics
simulation result. Or you could not even use Mailbox mode if you don't want to
worry about it possibly being there and possibly not being there.

For our example, I'll have the camera code be disjoint from the physics code
just to see how it would be done if you wanted to do it that way.

```rust
    // do camera updates distinctly from physics, based on this frame's time
    const MOUSE_SENSITIVITY: f32 = 0.05;
    let d_pitch_deg = input.orientation_change.1 * MOUSE_SENSITIVITY;
    let d_yaw_deg = input.orientation_change.0 * MOUSE_SENSITIVITY;
    self
      .camera
      .update_orientation(d_pitch_deg, d_yaw_deg);
    self
      .camera
      .update_position(&input.keys_held, 5.0 * input.seconds); // 5 meters / second
```

## Update `UserInput`

So obviously our user input is storing a few more things than before, let's look at that.

```rust
#[derive(Debug, Clone, Default)]
pub struct UserInput {
  pub end_requested: bool,
  pub new_frame_size: Option<(f64, f64)>,
  pub new_mouse_position: Option<(f64, f64)>,
  pub swap_projection: bool,
  pub keys_held: HashSet<VirtualKeyCode>,
  pub orientation_change: (f32, f32),
  pub seconds: f32,
}
```

Okay, and we're actually going to be tracking quite a bit more now, so our
polling method has a few more arguments.

```rust
  pub fn poll_events_loop(
    winit_state: &mut WinitState, last_timestamp: &mut SystemTime,
    keys_held: &mut HashSet<VirtualKeyCode>, focused: &mut bool, grabbed: &mut bool,
  ) -> Self {
```

Actually, all that stuff has to do with `winit` really, so it should be in the
`WinitState`, don't you think? We're already taking a `&mut WinitState`.

```rust
#[derive(Debug)]
pub struct WinitState {
  pub events_loop: EventsLoop,
  pub window: Window,
  pub keys_held: HashSet<VirtualKeyCode>,
  pub grabbed: bool,
}
```

Alright, and now our match statement is totally different, so we'll take it
again from the top. First though, we have to do an annoying manual split of the
borrow.

```rust
impl UserInput {
  pub fn poll_events_loop(winit_state: &mut WinitState, last_timestamp: &mut SystemTime) -> Self {
    let mut output = UserInput::default();
    // We have to manually split the borrow here. rustc, why you so dumb sometimes?
    let events_loop = &mut winit_state.events_loop;
    let window = &mut winit_state.window;
    let keys_held = &mut winit_state.keys_held;
    let grabbed = &mut winit_state.grabbed;
```

Now we can start the events poll. First up is CloseRequested, which we just mark
down in our output.

```rust
    // now we actually poll those events
    events_loop.poll_events(|event| match event {
      // Close when asked
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => output.end_requested = true,
```

Next we need to track what the state of all keys is. This is a little annoying
at the edge cases, because of [key
rollover](https://en.wikipedia.org/wiki/Rollover_(key)), and also because if the
uses presses and holds a key _before the window opens_ we won't get the key
press event for that. Most of the time though we can fairly reliably get key
info. Now there's two ways to do this: one is through the
[WindowEvent](https://docs.rs/winit/0.18.1/winit/enum.WindowEvent.html) type and
the other is through the
[DeviceEvent](https://docs.rs/winit/0.18.1/winit/enum.DeviceEvent.html) type. We
want to use DeviceEvent. The difference is that you only get window events for
keys when your window is active, but you get device events at all times. If the
user presses or releases a key when the window is out of focus we want to track
that. If they press a key and then click in the window, we want to respond to
that _right away_ without them having to release and press the key again.
Similarly, if they have a key held and then switch to another window, we want to
know if it got released while we didn't have focus.

```rust
      // Track all keys, all the time. Note that because of key rollover details
      // it's possible to get key released events for keys we don't think are
      // pressed. This is a hardware limit, not something you can evade.
      Event::DeviceEvent {
        event:
          DeviceEvent::Key(KeyboardInput {
            virtual_keycode: Some(code),
            state,
            ..
          }),
        ..
      } => drop(match state {
        ElementState::Pressed => keys_held.insert(code),
        ElementState::Released => keys_held.remove(&code),
      }),
```

That would be the end of it, but MacOS is dumb and doesn't provide keys as
device events. So we need to handle keys as window events too. Also, even on
non-mac there's a few window event keys that we want to respond do. We're
keeping "tab swaps the projection", and also we're adding "escape undoes the
grab".

```rust
// We want to respond to some of the keys specially when they're also
      // window events too (meaning that the window was focused when the event
      // happened).
      Event::WindowEvent {
        event:
          WindowEvent::KeyboardInput {
            input:
              KeyboardInput {
                state,
                virtual_keycode: Some(code),
                ..
              },
            ..
          },
        ..
      } => {
        #[cfg(feature = "metal")]
        {
          match state {
            ElementState::Pressed => keys_held.insert(code),
            ElementState::Released => keys_held.remove(&code),
          }
        };
        if state == ElementState::Pressed {
          match code {
            VirtualKeyCode::Tab => output.swap_projection = !output.swap_projection,
            VirtualKeyCode::Escape => {
              if *grabbed {
                debug!("Escape pressed while grabbed, releasing the mouse!");
                window
                  .grab_cursor(false)
                  .expect("Failed to release the mouse grab!");
                window.hide_cursor(false);
                *grabbed = false;
              }
            }
            _ => (),
          }
        }
      }
```

We also want to use `DeviceEvent` to track mouse motion. The difference between
this and the `CursorMoved` event from before is that `WindowEvent::CursorMoved`
gives the _position within the window_, while `DeviceEvent::MouseMotion` gives
the mouse's _position delta_. We're going to "grab" the mouse to lock it within
the window. When the mouse goes all the way to left and hits x=0 we'd stop
getting CursorMoved events, but we want to keep turning the view as long as the
user keeps turning the mouse. By using `MouseMotion` events we can track the
mouse's intended movement even while the cursor is grabbed.

Also, this is the part where you'd invert the X or Y movement effect if you
wanted to offer that option to users.

```rust
      // Always track the mouse motion, but only update the orientation if
      // we're "grabbed".
      Event::DeviceEvent {
        event: DeviceEvent::MouseMotion { delta: (dx, dy) },
        ..
      } => {
        if *grabbed {
          output.orientation_change.0 -= dx as f32;
          output.orientation_change.1 -= dy as f32;
        }
      }
```

Next, if the user clicks in the window we'll grab the cursor. There's a literal
`grab_cursor` call which _on windows_ will automatically hide the cursor too,
but on mac and some linux you have to issue `hide_cursor` as a separate command.
We'll just do both, since it doesn't hurt to tell the already-hidden cursor to
hide again on windows.

```rust
      // Left clicking in the window causes the mouse to get grabbed
      Event::WindowEvent {
        event:
          WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
          },
        ..
      } => {
        if *grabbed {
          debug!("Click! We already have the mouse grabbed.");
        } else {
          debug!("Click! Grabbing the mouse.");
          window.grab_cursor(true).expect("Failed to grab the mouse!");
          window.hide_cursor(true);
          *grabbed = true;
        }
      }
```

If the focus is lost, we want to automatically release any "grab". This is just
the same two calls with reverse values.

```rust
      // Automatically release the mouse when focus is lost
      Event::WindowEvent {
        event: WindowEvent::Focused(false),
        ..
      } => {
        if *grabbed {
          debug!("Lost Focus, releasing the mouse grab...");
          window
            .grab_cursor(false)
            .expect("Failed to release the mouse grab!");
          window.hide_cursor(false);
          *grabbed = false;
        } else {
          debug!("Lost Focus when mouse wasn't grabbed.");
        }
      }
```

Finally, we'll update our window size still. I'm not sure we're using that any
more, but oh well. We can just track it anyway.

```rust
      // Update our size info if the window changes size.
      Event::WindowEvent {
        event: WindowEvent::Resized(logical),
        ..
      } => {
        output.new_frame_size = Some((logical.width, logical.height));
      }
```

And at the end, after the event polling, we want to be sure to hand over a clone
of the `keys_held` set _only if_ we're grabbed. Otherwise the program would do
stuff even if it's out of focus. I'm sure there's some program that wants to do
that, but not us.

```rust
    output.keys_held = if *grabbed {
      keys_held.clone()
    } else {
      HashSet::new()
    };
```

And everything works!

Except we can't roll yet.

# Quaternion Free Camera (Slightly Slower, More Freedom)

Now we're gonna use Quaternions. They're not super covered in the Khan Academy
"Vector and Matrix" math course that I linked last lesson, at least not from
what I saw in their table of contents listing. Instead, try [this link
here](https://www.3dgep.com/understanding-quaternions/) to learn all about
them.They're sure _weird_. They're 4D! Isn't that already pretty weird all on
its own?

TODO
