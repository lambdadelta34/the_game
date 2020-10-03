use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    descriptor::PipelineLayoutAbstract,
    device::{Device, DeviceExtensions, Queue},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageUsage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::{
        vertex::{SingleBufferDefinition},
        viewport::Viewport,
        GraphicsPipeline,
    },
    swapchain,
    swapchain::{
        ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform, Swapchain,
    },
    sync,
    sync::{FlushError, GpuFuture},
};
use vulkano_win::{CreationError, VkSurfaceBuild};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use super::super::shaders::{vertex::vs, fragment::fs};

#[derive(Debug)]
pub struct WinitState {
    pub event_loop: EventLoop<()>,
    pub surface: Arc<Surface<Window>>,
    pub vulkan: Vulkan,
}
#[derive(Default, Debug, Clone)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

pub struct Vulkan {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub pipeline: Arc<
            GraphicsPipeline<
                    SingleBufferDefinition<Vertex>,
                    Box<dyn PipelineLayoutAbstract + Send + Sync>,
                    Arc<dyn RenderPassAbstract + Send + Sync>,
            >
    >,
    pub dynamic_state: DynamicState,
}

impl std::fmt::Debug for Vulkan {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "device: {:?}, queue: {:?}, swapchain: {:?}",
            self.device, self.queue, self.swapchain
        )
    }
}

impl WinitState {
    pub fn new() -> Result<Self, CreationError> {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("failed to create Vulkan instance")
        };
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();
        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device available");

        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");
        let (device, mut queues) = {
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            };

            Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue_family, 0.5)].iter().cloned(),
            )
            .expect("failed to create device")
        };
        let queue = queues.next().unwrap();
        let caps = surface
            .capabilities(physical)
            .expect("failed to get surface capabilities");
        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .expect("failed to create swapchain");

        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        ) as Arc<dyn RenderPassAbstract + Send + Sync>;

        // let pipeline = Arc::new(
        //     GraphicsPipeline::start()
        //         .vertex_input_single_buffer()
        //         .vertex_shader(vs.main_entry_point(), ())
        //         .triangle_list()
        //         .viewports_dynamic_scissors_irrelevant(1)
        //         .fragment_shader(fs.main_entry_point(), ())
        //         .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        //         .build(device.clone())
        //         .unwrap(),
        // );

        let p1 = GraphicsPipeline::start();
        let p2 = p1.vertex_input_single_buffer();
        let p3 = p2.vertex_shader(vs.main_entry_point(), ());
        let p4 = p3.triangle_list();
        let p5 = p4.viewports_dynamic_scissors_irrelevant(1);
        let p6 = p5.fragment_shader(fs.main_entry_point(), ());
        let p7 = p6.render_pass(Subpass::from(render_pass.clone(), 0).unwrap());
        let p8 = p7.build(device.clone());
        let p9 = p8.unwrap();
        let pipeline = Arc::new(p9);

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };
        let framebuffers =
            window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

        // let mut recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());
        // let mut idx = 0.;
        // let mut switch = false;

        Ok(Self {
            event_loop,
            surface,
            vulkan: Vulkan {
                device,
                queue,
                swapchain,
                framebuffers,
                pipeline,
                previous_frame_end,
                dynamic_state,
            },
        })
    }

    pub fn draw(vulkan: &mut Vulkan) {
        vulkan
            .previous_frame_end
            .as_mut()
            .unwrap()
            .cleanup_finished();
        // if recreate_swapchain {
        //     let dimensions: [u32; 2] = surface.window().inner_size().into();
        //     let (new_swapchain, new_images) = match swapchain.recreate_with_dimensions(dimensions) {
        //         Ok(r) => r,
        //         Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        //     };

        //     swapchain = new_swapchain;
        //     framebuffers =
        //         window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);
        //     recreate_swapchain = false;
        // }
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(vulkan.swapchain.clone(), None) {
                Ok(r) => r,
                // Err(AcquireError::OutOfDate) => {
                //     recreate_swapchain = true;
                //     panic!("");
                // }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            // recreate_swapchain = true;
        }
        let clear_values = vec![[1., 1., 1., 1.].into()];
        let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
            vulkan.device.clone(),
            vulkan.queue.family(),
        )
        .unwrap();

        let buf = {
            vulkano::impl_vertex!(Vertex, position, color);

            CpuAccessibleBuffer::from_iter(
                vulkan.device.clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-0.5, 0.5],
                        color: [1., 0., 0.],
                    },
                    Vertex {
                        position: [0., -0.5],
                        color: [0., 1., 0.],
                    },
                    Vertex {
                        position: [-0.5, -0.5],
                        color: [0., 0., 1.],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };
        // if switch {
        //     idx -= 0.01;
        // } else {
        //     idx += 0.01;
        // };
        // if idx >= 0.5 {
        //     switch = true;
        // } else if idx <= -0.5 {
        //     switch = false;
        // };

        builder
            .begin_render_pass(
                vulkan.framebuffers[image_num].clone(),
                false,
                clear_values,
            )
            .unwrap()
            .draw(
                vulkan.pipeline.clone(),
                &vulkan.dynamic_state,
                buf,
                (),
                (),
            )
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = vulkan
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(vulkan.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                vulkan.queue.clone(),
                vulkan.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                vulkan.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                // recreate_swapchain = true;
                println!("out of date");
                vulkan.previous_frame_end =
                    Some(sync::now(vulkan.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                vulkan.previous_frame_end =
                    Some(sync::now(vulkan.device.clone()).boxed());
            }
        }
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0., 0.],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
