use super::resources::{ResourceHolder, Resources};
use super::window::Window;
use gfx_hal::{
    command::{ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents},
    device::Device,
    image::Extent,
    pool::CommandPool,
    pso::{Rect, Viewport},
    queue::{CommandQueue, Submission},
    window::{Extent2D, PresentationSurface, Surface, SwapchainConfig},
};
use std::borrow::Borrow;

#[derive(Debug)]
pub struct Renderer {
    pub window: Window,
    pub resources: ResourceHolder,
}

impl Renderer {
    pub fn new() -> Result<Self, ()> {
        let window = Window::new()?;
        let resources = ResourceHolder::new(&window.window)?;

        Ok(Self { window, resources })
    }

    pub fn draw(resource_holder: &mut ResourceHolder, surface_extent: &mut Extent2D) {
        let resources: &mut Resources<_> = &mut resource_holder.0;
        let Resources {
            adapter,
            command_buffer,
            command_pool,
            device,
            pipelines,
            queue_group,
            rendering_complete_semaphore: semaphore,
            render_passes,
            submission_complete_fence: fence,
            surface,
            surface_color_format,
            ..
        } = resources;
        unsafe {
            // We refuse to wait more than a second, to avoid hanging.
            let render_timeout_ns = 1_000_000_000;
            device
                .wait_for_fence(&fence, render_timeout_ns)
                .expect("Out of memory or device lost");
            device.reset_fence(&fence).expect("Out of memory");
            command_pool.reset(false);
        }
        let caps = surface.capabilities(&adapter.physical_device);
        let mut swapchain_config =
            SwapchainConfig::from_caps(&caps, *surface_color_format, *surface_extent);
        // This seems to fix some fullscreen slowdown on macOS.
        if caps.image_count.contains(&3) {
            swapchain_config.image_count = 3;
        }
        *surface_extent = swapchain_config.extent;
        unsafe {
            surface
                .configure_swapchain(&device, swapchain_config)
                .expect("Failed to configure swapchain");
        };
        let surface_image = unsafe {
            // We refuse to wait more than a second, to avoid hanging.
            let acquire_timeout_ns = 1_000_000_000;
            match surface.acquire_image(acquire_timeout_ns) {
                Ok((image, _)) => image,
                Err(_) => {
                    return;
                }
            }
        };
        let framebuffer = unsafe {
            device
                .create_framebuffer(
                    &render_passes[0],
                    vec![surface_image.borrow()],
                    Extent {
                        width: surface_extent.width,
                        height: surface_extent.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };
        let viewport = {
            Viewport {
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: surface_extent.width as i16,
                    h: surface_extent.height as i16,
                },
                depth: 0.0..1.0,
            }
        };
        unsafe {
            command_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
            command_buffer.set_viewports(0, &[viewport.clone()]);
            command_buffer.set_scissors(0, &[viewport.rect]);
            command_buffer.begin_render_pass(
                &render_passes[0],
                &framebuffer,
                viewport.rect,
                &[ClearValue {
                    color: ClearColor {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                }],
                SubpassContents::Inline,
            );
            command_buffer.bind_graphics_pipeline(&pipelines[0]);
            command_buffer.draw(0..3, 0..1);
            command_buffer.end_render_pass();
            command_buffer.finish();
        }
        unsafe {
            let submission = Submission {
                command_buffers: vec![&command_buffer],
                wait_semaphores: None,
                signal_semaphores: vec![&semaphore],
            };
            queue_group.queues[0].submit(submission, Some(&fence));
            queue_group.queues[0]
                .present(surface, surface_image, Some(&semaphore))
                .unwrap();
            &device.destroy_framebuffer(framebuffer);
        }
    }
}
