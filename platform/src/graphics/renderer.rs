use super::resources::{PushConstants, ResourceHolder, Resources};
use crate::window::Window;
use gfx_hal::{
    command::{
        ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, RenderAttachmentInfo,
        SubpassContents,
    },
    device::Device,
    image::Extent,
    pool::CommandPool,
    pso::{Rect, ShaderStageFlags, Viewport},
    queue::Queue,
    window::{Extent2D, PresentationSurface, Surface, SwapchainConfig},
};
use queue::{event::Event, receiver::Receiver};
use std::borrow::Borrow;
use std::iter;
use world::WorldState;

// TODO: remove winit dependency
use winit::event::{Event as WEvent, WindowEvent};

#[derive(Debug)]
pub struct Renderer<'a> {
    pub resources: ResourceHolder,
    pub events: Receiver<Event<WEvent<'a, ()>>>,
    pub surface_extent: Extent2D,
}

impl<'a> Renderer<'a> {
    pub fn new(window: &Window, events: Receiver<Event<WEvent<'a, ()>>>) -> Result<Self, ()> {
        let resources = ResourceHolder::new(&window.window)?;

        let caps = surface.capabilities(&adapter.physical_device);
        let mut swapchain_config =
            SwapchainConfig::from_caps(&caps, *surface_color_format, self.surface_extent);
        self.surface_extent = swapchain_config.extent;

        Ok(Self {
            resources,
            events,
            surface_extent: window.surface_extent,
        })
    }

    pub fn update(&self, world: &WorldState) {
        let event = self.events.try_recv().unwrap();
        match event.payload {
            // redraw continiously
            WEvent::MainEventsCleared => {
                // Renderer::draw(&mut resources, &world, &mut extent);
                self.draw(&world);
            }
            WEvent::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { .. },
                ..
            } => {
                // self.recreate_swapchain();
            }
            _ => {}
        }
    }

    // fn draw(resources: &mut ResourceHolder, world: &WorldState, extent: &mut Extent2D) {
    fn draw(&mut self, world: &WorldState) {
        let Resources {
            adapter,
            command_buffer,
            command_pool,
            device,
            mut frame,
            pipeline_layouts,
            pipelines,
            queue_group,
            render_passes,
            rendering_complete_semaphore: semaphore,
            submission_complete_fence: fence,
            surface,
            surface_color_format,
            ..
        } = &mut self.resources.0;
        frame += 1;
        println!("FRAME {}", frame);
        unsafe {
            // We refuse to wait more than a second, to avoid hanging.
            let render_timeout_ns = 1_000_000_000;
            device
                .wait_for_fence(&fence, render_timeout_ns)
                .expect("Out of memory or device lost");
            device.reset_fence(&mut fence).expect("Out of memory");
            command_pool.reset(false);
        }
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
        let fat = swapchain_config.framebuffer_attachment();
        let framebuffer = unsafe {
            device
                .create_framebuffer(
                    &render_passes[0],
                    iter::once(fat),
                    Extent {
                        width: self.surface_extent.width,
                        height: self.surface_extent.height,
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
                    w: self.surface_extent.width as i16,
                    h: self.surface_extent.height as i16,
                },
                depth: 0.0..1.0,
            }
        };

        let small = [0.33, 0.33];
        let triangles = &[
            // Red triangle
            PushConstants {
                color: [1.0, 0.0, 0.0, 1.0],
                pos: [world.player.0, world.player.1],
                scale: small,
            },
        ];
        unsafe {
            command_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
            command_buffer.set_viewports(0, iter::once(viewport.clone()));
            command_buffer.set_scissors(0, iter::once(viewport.rect));
            command_buffer.begin_render_pass(
                &render_passes[0],
                &framebuffer,
                viewport.rect,
                iter::once(RenderAttachmentInfo {
                    image_view: surface_image.borrow(),
                    clear_value: ClearValue {
                        color: ClearColor {
                            float32: [0.0, 0.0, 0.0, 1.0],
                        },
                    },
                }),
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

        let queue = &mut queue_group.queues[0];
        unsafe {
            queue.submit(
                iter::once(&*command_buffer),
                iter::empty(),
                iter::once(&semaphore),
                Some(&mut fence),
            );
            queue
                .present(&mut surface, surface_image, Some(&mut semaphore))
                .unwrap();
            &device.destroy_framebuffer(framebuffer);
        }
    }
}

unsafe fn push_constant_bytes<T>(push_constants: &T) -> &[u32] {
    let size_in_bytes = std::mem::size_of::<T>();
    let size_in_u32s = size_in_bytes / std::mem::size_of::<u32>();
    let start_ptr = push_constants as *const T as *const u32;
    std::slice::from_raw_parts(start_ptr, size_in_u32s)
}
