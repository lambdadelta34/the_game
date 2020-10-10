use super::resources::{PushConstants, ResourceHolder, Resources};
use super::window::Window;
use gfx_hal::{
    command::{ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents},
    device::Device,
    image::Extent,
    pool::CommandPool,
    pso::{Rect, ShaderStageFlags, Viewport},
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

    pub fn draw(
        resource_holder: &mut ResourceHolder,
        surface_extent: &mut Extent2D,
        start_time: std::time::Instant,
    ) {
        let resources: &mut Resources<_> = &mut resource_holder.0;
        let Resources {
            adapter,
            command_buffer,
            command_pool,
            device,
            frame,
            pipeline_layouts,
            pipelines,
            queue_group,
            render_passes,
            rendering_complete_semaphore: semaphore,
            submission_complete_fence: fence,
            surface,
            surface_color_format,
            ..
        } = resources;
        println!("FRAME {}", frame);
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

        let anim = start_time.elapsed().as_secs_f32().sin() * 0.5 + 0.5;
        let small = [0.33, 0.33];
        let triangles = &[
            // Red triangle
            PushConstants {
                color: [1.0, 0.0, 0.0, 1.0],
                pos: [-0.5, -0.5],
                scale: small,
            },
            // Green triangle
            PushConstants {
                color: [0.0, 1.0, 0.0, 1.0],
                pos: [0.0, -0.5],
                scale: small,
            },
            // Blue triangle
            PushConstants {
                color: [0.0, 0.0, 1.0, 1.0],
                pos: [0.5, -0.5],
                scale: small,
            },
            // Blue <-> cyan animated triangle
            PushConstants {
                color: [0.0, anim, 1.0, 1.0],
                pos: [-0.5, 0.5],
                scale: small,
            },
            // Down <-> up animated triangle
            PushConstants {
                color: [1.0, 1.0, 1.0, 1.0],
                pos: [0.0, 0.5 - anim * 0.5],
                scale: small,
            },
            // Small <-> big animated triangle
            PushConstants {
                color: [1.0, 1.0, 1.0, 1.0],
                pos: [0.5, 0.5],
                scale: [0.33 + anim * 0.33, 0.33 + anim * 0.33],
            },
        ];
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
            for triangle in triangles {
                command_buffer.push_graphics_constants(
                    &pipeline_layouts[0],
                    ShaderStageFlags::VERTEX,
                    0,
                    push_constant_bytes(triangle),
                );
                command_buffer.draw(0..3, 0..1);
            }
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

        *frame += 1;
    }
}

unsafe fn push_constant_bytes<T>(push_constants: &T) -> &[u32] {
    let size_in_bytes = std::mem::size_of::<T>();
    let size_in_u32s = size_in_bytes / std::mem::size_of::<u32>();
    let start_ptr = push_constants as *const T as *const u32;
    std::slice::from_raw_parts(start_ptr, size_in_u32s)
}
