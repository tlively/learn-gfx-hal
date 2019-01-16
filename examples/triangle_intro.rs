#![allow(clippy::len_zero)]

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use gfx_hal::{
  adapter::{Adapter, PhysicalDevice},
  command::{ClearColor, ClearValue, CommandBuffer, MultiShot, Primary},
  device::Device,
  format::{Aspects, ChannelType, Format, Swizzle},
  image::{Extent, Layout, SubresourceRange, Usage, ViewKind},
  pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDesc},
  pool::{CommandPool, CommandPoolCreateFlags},
  pso::{
    AttributeDesc, BakedStates, BasePipeline, BlendDesc, BlendOp, BlendState, ColorBlendDesc, ColorMask, DepthStencilDesc,
    DepthTest, DescriptorSetLayoutBinding, EntryPoint, Face, Factor, FrontFace, GraphicsPipelineDesc, GraphicsShaderSet,
    InputAssemblerDesc, LogicOp, Multisampling, PipelineCreationFlags, PipelineStage, PolygonMode, Rasterizer, Rect,
    ShaderStageFlags, Specialization, StencilTest, VertexBufferDesc, Viewport,
  },
  queue::{family::QueueGroup, Submission},
  window::{Backbuffer, Extent2D, FrameSync, PresentMode, Swapchain, SwapchainConfig},
  Backend, Gpu, Graphics, Instance, Primitive, QueueFamily, Surface,
};

use winit::{dpi::LogicalSize, CreationError, Event, EventsLoop, Window, WindowBuilder, WindowEvent};

pub const WINDOW_NAME: &str = "Triangle Intro";

pub const VERTEX_SOURCE: &str = "#version 330 core
layout (location = 0) in vec2 position;

void main()
{
  gl_Position = vec4(position, 0.0, 1.0);
}";

pub const FRAGMENT_SOURCE: &str = "#version 330 core
out vec4 FragColor;

void main()
{
  FragColor = vec4(1.0);
}";

pub struct Triangle {
  pub points: [[f32; 2]; 3],
}

pub struct HalState {
  current_frame: usize,
  frames_in_flight: usize,
  in_flight_fences: Vec<<back::Backend as Backend>::Fence>,
  render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
  image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore>,
  command_buffers: Vec<CommandBuffer<back::Backend, Graphics, MultiShot, Primary>>,
  command_pool: ManuallyDrop<CommandPool<back::Backend, Graphics>>,
  framebuffers: Vec<<back::Backend as Backend>::Framebuffer>,
  image_views: Vec<(<back::Backend as Backend>::ImageView)>,
  render_pass: ManuallyDrop<<back::Backend as Backend>::RenderPass>,
  render_area: Rect,
  queue_group: QueueGroup<back::Backend, Graphics>,
  swapchain: ManuallyDrop<<back::Backend as Backend>::Swapchain>,
  device: ManuallyDrop<back::Device>,
  _adapter: Adapter<back::Backend>,
  _surface: <back::Backend as Backend>::Surface,
  _instance: ManuallyDrop<back::Instance>,
}
impl HalState {
  /// Creates a new, fully initialized HalState.
  pub fn new(window: &Window) -> Result<Self, &'static str> {
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
          .any(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
      })
      .ok_or("Couldn't find a graphical Adapter!")?;

    // Open A Device and take out a QueueGroup
    let (device, queue_group) = {
      let queue_family = adapter
        .queue_families
        .iter()
        .find(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
        .ok_or("Couldn't find a QueueFamily with graphics!")?;
      let Gpu { device, mut queues } = unsafe {
        adapter
          .physical_device
          .open(&[(&queue_family, &[1.0; 1])])
          .map_err(|_| "Couldn't open the PhysicalDevice!")?
      };
      let queue_group = queues
        .take::<Graphics>(queue_family.id())
        .ok_or("Couldn't take ownership of the QueueGroup!")?;
      let _ = if queue_group.queues.len() > 0 {
        Ok(())
      } else {
        Err("The QueueGroup did not have any CommandQueues available!")
      }?;
      (device, queue_group)
    };

    // Create A Swapchain, this is extra long
    let (swapchain, extent, backbuffer, format, frames_in_flight) = {
      let (caps, preferred_formats, present_modes, composite_alphas) = surface.compatibility(&adapter.physical_device);
      info!("{:?}", caps);
      info!("Preferred Formats: {:?}", preferred_formats);
      info!("Present Modes: {:?}", present_modes);
      info!("Composite Alphas: {:?}", composite_alphas);
      //
      let present_mode = {
        use gfx_hal::window::PresentMode::*;
        [Mailbox, Fifo, Relaxed, Immediate]
          .iter()
          .cloned()
          .find(|pm| present_modes.contains(pm))
          .ok_or("No PresentMode values specified!")?
      };
      let composite_alpha = {
        use gfx_hal::window::CompositeAlpha::*;
        [Opaque, Inherit, PreMultiplied, PostMultiplied]
          .iter()
          .cloned()
          .find(|ca| composite_alphas.contains(ca))
          .ok_or("No CompositeAlpha values specified!")?
      };
      let format = match preferred_formats {
        None => Format::Rgba8Srgb,
        Some(formats) => match formats
          .iter()
          .find(|format| format.base_format().1 == ChannelType::Srgb)
          .cloned()
        {
          Some(srgb_format) => srgb_format,
          None => formats.get(0).cloned().ok_or("Preferred format list was empty!")?,
        },
      };
      let extent = caps.extents.end;
      let image_count = if present_mode == PresentMode::Mailbox {
        (caps.image_count.end - 1).min(3)
      } else {
        (caps.image_count.end - 1).min(2)
      };
      let image_layers = 1;
      let image_usage = if caps.usage.contains(Usage::COLOR_ATTACHMENT) {
        Usage::COLOR_ATTACHMENT
      } else {
        Err("The Surface isn't capable of supporting color!")?
      };
      let swapchain_config = SwapchainConfig {
        present_mode,
        composite_alpha,
        format,
        extent,
        image_count,
        image_layers,
        image_usage,
      };
      info!("{:?}", swapchain_config);
      //
      let (swapchain, backbuffer) = unsafe {
        device
          .create_swapchain(&mut surface, swapchain_config, None)
          .map_err(|_| "Failed to create the swapchain!")?
      };
      (swapchain, extent, backbuffer, format, image_count as usize)
    };

    // Create Our Sync Primitives
    let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
      let mut image_available_semaphores: Vec<<back::Backend as Backend>::Semaphore> = vec![];
      let mut render_finished_semaphores: Vec<<back::Backend as Backend>::Semaphore> = vec![];
      let mut in_flight_fences: Vec<<back::Backend as Backend>::Fence> = vec![];
      for _ in 0..frames_in_flight {
        in_flight_fences.push(device.create_fence(true).map_err(|_| "Could not create a fence!")?);
        image_available_semaphores.push(device.create_semaphore().map_err(|_| "Could not create a semaphore!")?);
        render_finished_semaphores.push(device.create_semaphore().map_err(|_| "Could not create a semaphore!")?);
      }
      (image_available_semaphores, render_finished_semaphores, in_flight_fences)
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
          .map_err(|_| "Couldn't create a render pass!")?
      }
    };

    // Create The ImageViews
    let image_views: Vec<_> = match backbuffer {
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
            .map_err(|_| "Couldn't create the image_view for the image!")
        })
        .collect::<Result<Vec<_>, &str>>()?,
      Backbuffer::Framebuffer(_) => unimplemented!("Can't handle framebuffer backbuffer!"),
    };

    // Create Our FrameBuffers
    let framebuffers: Vec<<back::Backend as Backend>::Framebuffer> = {
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
            .map_err(|_| "Failed to create a framebuffer!")
        })
        .collect::<Result<Vec<_>, &str>>()?
    };

    // Create Our CommandPool
    let mut command_pool = unsafe {
      device
        .create_command_pool_typed(&queue_group, CommandPoolCreateFlags::RESET_INDIVIDUAL)
        .map_err(|_| "Could not create the raw command pool!")?
    };

    // Create Our CommandBuffers
    let command_buffers: Vec<_> = framebuffers.iter().map(|_| command_pool.acquire_command_buffer()).collect();

    Ok(Self {
      _instance: ManuallyDrop::new(instance),
      _surface: surface,
      _adapter: adapter,
      device: ManuallyDrop::new(device),
      queue_group,
      swapchain: ManuallyDrop::new(swapchain),
      render_area: extent.to_extent().rect(),
      render_pass: ManuallyDrop::new(render_pass),
      image_views,
      framebuffers,
      command_pool: ManuallyDrop::new(command_pool),
      command_buffers,
      image_available_semaphores,
      render_finished_semaphores,
      in_flight_fences,
      frames_in_flight,
      current_frame: 0,
    })
  }

  fn create_pipeline(
    device: &mut back::Device, extent: Extent2D, render_pass: &<back::Backend as Backend>::RenderPass,
  ) -> Result<
    (
      Vec<<back::Backend as Backend>::DescriptorSetLayout>,
      <back::Backend as Backend>::PipelineLayout,
      <back::Backend as Backend>::GraphicsPipeline,
    ),
    &'static str,
  > {
    let mut compiler = shaderc::Compiler::new().ok_or("shaderc not found!")?;
    let vertex_compile_artifact = compiler
      .compile_into_spirv(VERTEX_SOURCE, shaderc::ShaderKind::Vertex, "vertex.vert", "main", None)
      .map_err(|_| "Couldn't compile vertex shader!")?;
    let fragment_compile_artifact = compiler
      .compile_into_spirv(FRAGMENT_SOURCE, shaderc::ShaderKind::Fragment, "fragment.frag", "main", None)
      .map_err(|_| "Couldn't compile fragment shader!")?;
    let vertex_shader_module = unsafe {
      device
        .create_shader_module(vertex_compile_artifact.as_binary_u8())
        .map_err(|_| "Couldn't make the vertex module")?
    };
    let fragment_shader_module = unsafe {
      device
        .create_shader_module(fragment_compile_artifact.as_binary_u8())
        .map_err(|_| "Couldn't make the fragment module")?
    };
    let (descriptor_set_layouts, pipeline_layout, gfx_pipeline) = {
      let (vs_entry, fs_entry) = (
        EntryPoint::<back::Backend> {
          entry: "main",
          module: &vertex_shader_module,
          specialization: Specialization {
            constants: &[],
            data: &[],
          },
        },
        EntryPoint::<back::Backend> {
          entry: "main",
          module: &fragment_shader_module,
          specialization: Specialization {
            constants: &[],
            data: &[],
          },
        },
      );
      let shaders = GraphicsShaderSet {
        vertex: vs_entry,
        hull: None,
        domain: None,
        geometry: None,
        fragment: Some(fs_entry),
      };

      let rasterizer = Rasterizer {
        depth_clamping: false,
        polygon_mode: PolygonMode::Fill,
        cull_face: Face::BACK,
        front_face: FrontFace::Clockwise,
        depth_bias: None,
        conservative: false,
      };
      let vertex_buffers: Vec<VertexBufferDesc> = Vec::new();
      let attributes: Vec<AttributeDesc> = Vec::new();

      let input_assembler = InputAssemblerDesc::new(Primitive::TriangleList);

      let blender = {
        let blend_state = BlendState::On {
          color: BlendOp::Add {
            src: Factor::One,
            dst: Factor::Zero,
          },
          alpha: BlendOp::Add {
            src: Factor::One,
            dst: Factor::Zero,
          },
        };

        BlendDesc {
          logic_op: Some(LogicOp::Copy),
          targets: vec![ColorBlendDesc(ColorMask::ALL, blend_state)],
        }
      };

      let depth_stencil = DepthStencilDesc {
        depth: DepthTest::Off,
        depth_bounds: false,
        stencil: StencilTest::Off,
      };

      let multisampling: Option<Multisampling> = None;

      let baked_states = BakedStates {
        viewport: Some(Viewport {
          rect: Rect {
            x: 0,
            y: 0,
            w: extent.width as i16,
            h: extent.width as i16,
          },
          depth: (0.0..1.0),
        }),
        scissor: Some(Rect {
          x: 0,
          y: 0,
          w: extent.width as i16,
          h: extent.height as i16,
        }),
        blend_color: None,
        depth_bounds: None,
      };

      let bindings = Vec::<DescriptorSetLayoutBinding>::new();
      let immutable_samplers = Vec::<<back::Backend as Backend>::Sampler>::new();
      let descriptor_set_layouts: Vec<<back::Backend as Backend>::DescriptorSetLayout> = vec![unsafe {
        device
          .create_descriptor_set_layout(bindings, immutable_samplers)
          .map_err(|_| "Couldn't make a DescriptorSetLayout")?
      }];
      let push_constants = Vec::<(ShaderStageFlags, std::ops::Range<u32>)>::new();
      let layout = unsafe {
        device
          .create_pipeline_layout(&descriptor_set_layouts, push_constants)
          .map_err(|_| "Couldn't create a pipeline layout")?
      };

      let subpass = Subpass {
        index: 0,
        main_pass: render_pass,
      };

      let flags = PipelineCreationFlags::empty();

      let parent = BasePipeline::None;

      let gfx_pipeline = {
        let desc = GraphicsPipelineDesc {
          shaders,
          rasterizer,
          vertex_buffers,
          attributes,
          input_assembler,
          blender,
          depth_stencil,
          multisampling,
          baked_states,
          layout: &layout,
          subpass,
          flags,
          parent,
        };

        unsafe {
          device
            .create_graphics_pipeline(&desc, None)
            .map_err(|_| "Couldn't create a graphics pipeline!")?
        }
      };

      (descriptor_set_layouts, layout, gfx_pipeline)
    };

    unsafe {
      device.destroy_shader_module(vertex_shader_module);
      device.destroy_shader_module(fragment_shader_module);
    }

    Ok((descriptor_set_layouts, pipeline_layout, gfx_pipeline))
  }

  /// Draw a frame that's just cleared to the color specified.
  pub fn draw_clear_frame(&mut self, color: [f32; 4]) -> Result<(), &'static str> {
    // SETUP FOR THIS FRAME
    let flight_fence = &self.in_flight_fences[self.current_frame];
    let image_available = &self.image_available_semaphores[self.current_frame];
    let render_finished = &self.render_finished_semaphores[self.current_frame];
    // Advance the frame _before_ we start using the `?` operator
    self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

    let (i_u32, i_usize) = unsafe {
      self
        .device
        .wait_for_fence(flight_fence, core::u64::MAX)
        .map_err(|_| "Failed to wait on the fence!")?;
      self
        .device
        .reset_fence(flight_fence)
        .map_err(|_| "Couldn't reset the fence!")?;
      let image_index = self
        .swapchain
        .acquire_image(core::u64::MAX, FrameSync::Semaphore(image_available))
        .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
      (image_index, image_index as usize)
    };

    // RECORD COMMANDS
    unsafe {
      let buffer = &mut self.command_buffers[i_usize];
      let clear_values = [ClearValue::Color(ClearColor::Float(color))];
      buffer.begin(false);
      buffer.begin_render_pass_inline(
        &self.render_pass,
        &self.framebuffers[i_usize],
        self.render_area,
        clear_values.iter(),
      );
      buffer.finish();
    }

    // SUBMISSION AND PRESENT
    let command_buffers = &self.command_buffers[i_usize..=i_usize];
    let wait_semaphores: ArrayVec<[_; 1]> = [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
    let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    // yes, you have to write it twice like this. yes, it's silly.
    let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    let submission = Submission {
      command_buffers,
      wait_semaphores,
      signal_semaphores,
    };
    let the_command_queue = &mut self.queue_group.queues[0];
    unsafe {
      the_command_queue.submit(submission, Some(flight_fence));
      self
        .swapchain
        .present(the_command_queue, i_u32, present_wait_semaphores)
        .map_err(|_| "Failed to present into the swapchain!")
    }
  }

  pub fn draw_triangle_frame(&mut self, triangle: Triangle) -> Result<(), &'static str> {
    // SETUP FOR THIS FRAME
    let flight_fence = &self.in_flight_fences[self.current_frame];
    let image_available = &self.image_available_semaphores[self.current_frame];
    let render_finished = &self.render_finished_semaphores[self.current_frame];
    // Advance the frame _before_ we start using the `?` operator
    self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

    let (i_u32, i_usize) = unsafe {
      self
        .device
        .wait_for_fence(flight_fence, core::u64::MAX)
        .map_err(|_| "Failed to wait on the fence!")?;
      self
        .device
        .reset_fence(flight_fence)
        .map_err(|_| "Couldn't reset the fence!")?;
      let image_index = self
        .swapchain
        .acquire_image(core::u64::MAX, FrameSync::Semaphore(image_available))
        .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
      (image_index, image_index as usize)
    };

    // RECORD COMMANDS
    unsafe {
      let buffer = &mut self.command_buffers[i_usize];
      const TRIANGLE_CLEAR: [ClearValue; 1] = [ClearValue::Color(ClearColor::Float([0.1, 0.2, 0.3, 1.0]))];
      buffer.begin(false);
      {
        let _encoder = buffer.begin_render_pass_inline(
          &self.render_pass,
          &self.framebuffers[i_usize],
          self.render_area,
          TRIANGLE_CLEAR.iter(),
        );
        //encoder.bind_graphics_pipeline(&self.pipeline);
        //let buffers: ArrayList<[_; 1]> = [(&self.buffer, 0)].into();
        //encoder.bind_vertex_buffers(0, buffers);
        //encoder.draw(0 .. 3, 0 .. 1);
      }
      buffer.finish();
    }

    // SUBMISSION AND PRESENT
    let command_buffers = &self.command_buffers[i_usize..=i_usize];
    let wait_semaphores: ArrayVec<[_; 1]> = [(image_available, PipelineStage::COLOR_ATTACHMENT_OUTPUT)].into();
    let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    // yes, you have to write it twice like this. yes, it's silly.
    let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
    let submission = Submission {
      command_buffers,
      wait_semaphores,
      signal_semaphores,
    };
    let the_command_queue = &mut self.queue_group.queues[0];
    unsafe {
      the_command_queue.submit(submission, Some(flight_fence));
      self
        .swapchain
        .present(the_command_queue, i_u32, present_wait_semaphores)
        .map_err(|_| "Failed to present into the swapchain!")
    }
  }
}
impl core::ops::Drop for HalState {
  /// We have to clean up "leaf" elements before "root" elements. Basically, we
  /// clean up in reverse of the order that we created things.
  fn drop(&mut self) {
    let _ = self.device.wait_idle();
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
      for framebuffer in self.framebuffers.drain(..) {
        self.device.destroy_framebuffer(framebuffer);
      }
      for image_view in self.image_views.drain(..) {
        self.device.destroy_image_view(image_view);
      }
      // LAST RESORT STYLE CODE, NOT TO BE IMITATED LIGHTLY
      use core::ptr::read;
      self
        .device
        .destroy_command_pool(ManuallyDrop::into_inner(read(&mut self.command_pool)).into_raw());
      self
        .device
        .destroy_render_pass(ManuallyDrop::into_inner(read(&mut self.render_pass)));
      self
        .device
        .destroy_swapchain(ManuallyDrop::into_inner(read(&mut self.swapchain)));
      ManuallyDrop::drop(&mut self.device);
      ManuallyDrop::drop(&mut self._instance);
    }
  }
}

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
    let output = WindowBuilder::new()
      .with_title(title)
      .with_dimensions(size)
      .build(&events_loop);
    output.map(|window| Self { events_loop, window })
  }
}
impl Default for WinitState {
  /// Makes an 800x600 window with the `WINDOW_NAME` value as the title.
  /// ## Panics
  /// If a `CreationError` occurs.
  fn default() -> Self {
    Self::new(
      WINDOW_NAME,
      LogicalSize {
        width: 800.0,
        height: 600.0,
      },
    )
    .expect("Could not create a window!")
  }
}

#[derive(Debug, Clone, Default)]
pub struct UserInput {
  pub end_requested: bool,
  pub new_frame_size: Option<(f64, f64)>,
  pub new_mouse_position: Option<(f64, f64)>,
}
impl UserInput {
  pub fn poll_events_loop(events_loop: &mut EventsLoop) -> Self {
    let mut output = UserInput::default();
    events_loop.poll_events(|event| match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => output.end_requested = true,
      Event::WindowEvent {
        event: WindowEvent::Resized(logical),
        ..
      } => {
        output.new_frame_size = Some((logical.width, logical.height));
      }
      Event::WindowEvent {
        event: WindowEvent::CursorMoved { position, .. },
        ..
      } => {
        output.new_mouse_position = Some((position.x, position.y));
      }
      _ => (),
    });
    output
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalState {
  pub frame_width: f64,
  pub frame_height: f64,
  pub mouse_x: f64,
  pub mouse_y: f64,
}
impl LocalState {
  pub fn update_from_input(&mut self, input: UserInput) {
    if let Some(frame_size) = input.new_frame_size {
      self.frame_width = frame_size.0;
      self.frame_height = frame_size.1;
    }
    if let Some(position) = input.new_mouse_position {
      self.mouse_x = position.0;
      self.mouse_y = position.1;
    }
  }
}

fn do_the_render(hal_state: &mut HalState, local_state: &LocalState) -> Result<(), &'static str> {
  let r = (local_state.mouse_x / local_state.frame_width) as f32;
  let g = (local_state.mouse_y / local_state.frame_height) as f32;
  let b = (r + g) * 0.3;
  let a = 1.0;
  hal_state.draw_clear_frame([r, g, b, a])
}

fn main() {
  simple_logger::init().unwrap();

  let mut winit_state = WinitState::default();

  let mut hal_state = match HalState::new(&winit_state.window) {
    Ok(state) => state,
    Err(e) => panic!(e),
  };

  let (frame_width, frame_height) = winit_state
    .window
    .get_inner_size()
    .map(|logical| logical.into())
    .unwrap_or((0.0, 0.0));
  let mut local_state = LocalState {
    frame_width,
    frame_height,
    mouse_x: 0.0,
    mouse_y: 0.0,
  };

  loop {
    let inputs = UserInput::poll_events_loop(&mut winit_state.events_loop);
    if inputs.end_requested {
      break;
    }
    local_state.update_from_input(inputs);
    if let Err(e) = do_the_render(&mut hal_state, &local_state) {
      error!("Rendering Error: {:?}", e);
      break;
    }
  }
}
