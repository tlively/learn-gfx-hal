#![allow(clippy::len_zero)]

#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;

#[macro_use]
extern crate log;

use gfx_hal::{
  adapter::{Adapter, PhysicalDevice},
  command::{ClearColor, ClearValue, CommandBuffer, MultiShot, Primary},
  device::Device,
  error::HostExecutionError,
  format::{Aspects, ChannelType, Format, Swizzle},
  image::{Extent, Layout, SubresourceRange, ViewKind},
  pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
  pool::{CommandPool, CommandPoolCreateFlags},
  pso::{PipelineStage, Rect},
  queue::{capability::Capability, CommandQueue, Submission},
  window::{Backbuffer, Extent2D, FrameSync, PresentMode, Swapchain, SwapchainConfig},
  Backend, Gpu, Graphics, Instance, QueueFamily, Surface,
};
use winit::{dpi::LogicalSize, CreationError, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

pub const WINDOW_NAME: &str = "Hello Clear";

pub struct HalState {
  _instance: back::Instance,
  _surface: <back::Backend as Backend>::Surface,
  _adapter: Adapter<back::Backend>,
  device: back::Device,
  swapchain: <back::Backend as Backend>::Swapchain,
  command_queues: Vec<CommandQueue<back::Backend, Graphics>>,
  extent: Extent2D,
  render_pass: <back::Backend as Backend>::RenderPass,
  image_views: Vec<(<back::Backend as Backend>::ImageView)>,
  swapchain_framebuffers: Vec<<back::Backend as Backend>::Framebuffer>,
  command_pool: Option<CommandPool<back::Backend, Graphics>>,
  submission_command_buffers: Vec<CommandBuffer<back::Backend, Graphics, MultiShot, Primary>>,
  image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
  render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
  in_flight_fences: Vec<<back::Backend as Backend>::Fence>,
  current_frame: usize,
}
impl HalState {
  const MAX_FRAMES_IN_FLIGHT: usize = 3;

  pub fn new(window: &Window) -> Self {
    // Create An Instance
    let instance = back::Instance::create(WINDOW_NAME, 1);

    // Create A Surface
    let mut surface = instance.create_surface(window);

    // Select An Adapter
    let adapter = instance
      .enumerate_adapters()
      .into_iter()
      .find(|a| {
        a.queue_families
          .iter()
          .any(|qf| qf.supports_graphics() && qf.max_queues() > 0 && surface.supports_queue_family(qf))
      })
      .expect("Couldn't find a graphical Adapter!");

    // Open A Device
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

    // Create A Swapchain
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

    // Define A RenderPass
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

    // Create The ImageViews
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

    // Create Our FrameBuffers
    let swapchain_framebuffers: Vec<<back::Backend as Backend>::Framebuffer> = {
      image_views
        .iter()
        .map(|image_view| unsafe {
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

    // Create Our CommandPool
    let mut command_pool = unsafe {
      let raw_command_pool = device
        .create_command_pool(qf_id, CommandPoolCreateFlags::empty())
        .expect("Could not create the raw command pool!");
      assert!(Graphics::supported_by(queue_type));
      CommandPool::<back::Backend, Graphics>::new(raw_command_pool)
    };

    // Create Our CommandBuffers
    let submission_command_buffers: Vec<_> = swapchain_framebuffers.iter().map(|_| command_pool.acquire_command_buffer()).collect();

    // Create Our Sync Primitives
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
  }

  /// Waits until the device goes idle.
  pub fn wait_until_idle(&self) -> Result<(), HostExecutionError> {
    self.device.wait_idle()
  }

  /// Draw a frame that's just cleared to the color specified.
  pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &str> {
    unsafe {
      // give shorter names to the synchronizations for the current frame
      let fence = &self.in_flight_fences[self.current_frame];
      let image_available = &self.image_available_semaphores[self.current_frame];
      let render_finished = &self.render_finished_semaphores[self.current_frame];

      // Wait and acquire an image index, which lets us pick out the correct command buffer.
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

      // Fill up that command buffer with the instructions to clear the screen
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

      // Submit the buffer, present the image it makes
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
    }
  }
}
/*
impl core::ops::Drop for HalState {
  fn drop(&mut self) {
    use core::mem::{replace, zeroed};
    unsafe {
      for fence in self.in_flight_fences.drain(..) {
        self.device.destroy_fence(fence)
      }
      for semaphore in self.render_finished_semaphores.drain(..) {
        self.device.destroy_semaphore(semaphore)
      }
      for semaphore in self.image_available_semaphores.drain(..) {
        self.device.destroy_semaphore(semaphore)
      }
      self.command_pool.take().map(|command_pool| {
        self.device.destroy_command_pool(command_pool.into_raw());
      });
      for framebuffer in self.swapchain_framebuffers.drain(..) {
        self.device.destroy_framebuffer(framebuffer);
      }
      for image_view in self.image_views.drain(..) {
        self.device.destroy_image_view(image_view);
      }
      self.device.destroy_render_pass(replace(&mut self.render_pass, zeroed()));
      self.device.destroy_swapchain(replace(&mut self.swapchain, zeroed()));
    }
  }
}
*/

#[derive(Debug)]
pub struct WinitState {
  pub events_loop: EventsLoop,
  pub window: Window,
}
impl WinitState {
  /// Constructs a new `EventsLoop` and `Window` pair.
  ///
  /// The specified title and size are used, other elements are default.
  /// ## Failure
  /// It's possible for the window creation to fail. This is unlikely.
  pub fn new<T: Into<String>>(title: T, size: LogicalSize) -> Result<Self, CreationError> {
    let events_loop = EventsLoop::new();
    let output = WindowBuilder::new().with_title(title).with_dimensions(size).build(&events_loop);
    output.map(|window| Self { events_loop, window })
  }
}
impl Default for WinitState {
  /// Makes an 800x600 window with the `WINDOW_NAME` value as the title.
  /// ## Panics
  /// If a `CreationError` occurs.
  fn default() -> Self {
    Self::new(WINDOW_NAME, LogicalSize { width: 800.0, height: 600.0 }).expect("Could not create a window!")
  }
}

fn main() {
  env_logger::init();

  let mut winit_state = WinitState::default();

  let mut hal_state = HalState::new(&winit_state.window);

  let mut running = true;
  let (mut frame_width, mut frame_height) = winit_state.window.get_inner_size().map(|logical| logical.into()).unwrap_or((0.0, 0.0));
  let (mut mouse_x, mut mouse_y) = (0.0, 0.0);

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
  }

  // If we leave the main loop for any reason, we want to shut down as
  // gracefully as we can.
  if let Err(e) = hal_state.wait_until_idle() {
    error!("Error while waiting for the queues to idle: {}", e);
  }
}

// TODO: Theoretically one could do cleanup here? We should probably
