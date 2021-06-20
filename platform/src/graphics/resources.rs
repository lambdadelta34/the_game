#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use super::super::APP_NAME;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    command::Level,
    device::Device,
    format::{ChannelType, Format},
    image::Layout,
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, Subpass, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::{
        BlendState, ColorBlendDesc, ColorMask, DescriptorSetLayoutBinding, DescriptorType,
        EntryPoint, Face, GraphicsPipelineDesc, ImageDescriptorType, InputAssemblerDesc, Primitive,
        PrimitiveAssemblerDesc, Rasterizer, ShaderStageFlags, Specialization,
    },
    queue::{QueueFamily, QueueGroup},
    window::{PresentationSurface, Surface},
    Features, Instance,
};
use shaderc::{Compiler, ShaderKind};
use std::iter;
use std::mem::ManuallyDrop;
use winit::event::WindowEvent;
use winit::window::Window;

#[derive(Debug)]
pub struct Resources<B: gfx_hal::Backend> {
    pub instance: B::Instance,
    pub surface: B::Surface,
    pub adapter: Adapter<B>,
    pub device: B::Device,
    pub queue_group: QueueGroup<B>,
    pub render_passes: Vec<B::RenderPass>,
    pub pipeline_layouts: Vec<B::PipelineLayout>,
    pub pipelines: Vec<B::GraphicsPipeline>,
    // pub command_buffers: Vec<B::CommandBuffer>,
    // pub fences: Vec<B::Fence>,
    // pub semaphores: Vec<B::Semaphore>,
    pub command_pool: B::CommandPool,

    pub command_buffer: B::CommandBuffer,
    pub submission_complete_fence: B::Fence,
    pub rendering_complete_semaphore: B::Semaphore,
    pub surface_color_format: Format,
    pub frame: u64,
    pub events: Vec<WindowEvent<'static>>,
}

impl Resources<back::Backend> {
    pub fn new(window: &Window) -> Result<Self, ()> {
        let (instance, surface, adapter) = {
            let instance = back::Instance::create(APP_NAME, 1).expect("Backend not supported");
            let surface = unsafe {
                instance
                    .create_surface(window)
                    .expect("Failed to create surface for window")
            };
            let adapter = instance.enumerate_adapters().remove(0);
            (instance, surface, adapter)
        };
        let (device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|family| {
                    surface.supports_queue_family(family) && family.queue_type().supports_graphics()
                })
                .expect("No compatible queue family found");
            let mut gpu = unsafe {
                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0])], Features::empty())
                    .expect("Failed to open device")
            };
            (gpu.device, gpu.queue_groups.pop().unwrap())
        };

        let (command_pool, command_buffer) = unsafe {
            let mut command_pool = device
                .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                .expect("Out of memory");
            let command_buffer = command_pool.allocate_one(Level::Primary);
            (command_pool, command_buffer)
        };
        let surface_color_format = {
            let supported_formats = surface
                .supported_formats(&adapter.physical_device)
                .unwrap_or(vec![]);
            let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);
            supported_formats
                .into_iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .unwrap_or(default_format)
        };
        let render_pass = {
            let color_attachment = Attachment {
                format: Some(surface_color_format),
                samples: 1,
                ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
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
                    .create_render_pass(
                        iter::once(color_attachment),
                        iter::once(subpass),
                        iter::empty(),
                    )
                    .expect("Out of memory")
            }
        };
        let set_layout = ManuallyDrop::new(
            unsafe {
                device.create_descriptor_set_layout(
                    vec![DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: DescriptorType::Image {
                            ty: ImageDescriptorType::Sampled {
                                with_sampler: false,
                            },
                        },
                        count: 1,
                        stage_flags: ShaderStageFlags::VERTEX,
                        immutable_samplers: false,
                    }]
                    .into_iter(),
                    iter::empty(),
                )
            }
            .expect("Can't create descriptor set layout"),
        );

        let pipeline_layout = unsafe {
            // let push_constant_bytes = std::mem::size_of::<PushConstants>() as u32;
            device
                .create_pipeline_layout(iter::once(&*set_layout), iter::empty())
                // .create_pipeline_layout(&[], &[(ShaderStageFlags::VERTEX, 0..push_constant_bytes)])
                .expect("Out of memory")
        };
        let vertex_shader = include_str!("./shaders/vertex/vs.vert");
        let fragment_shader = include_str!("./shaders/fragment/fs.frag");
        let pipeline = unsafe {
            make_pipeline::<back::Backend>(
                &device,
                &render_pass,
                &pipeline_layout,
                vertex_shader,
                fragment_shader,
            )
        };
        let submission_complete_fence = device.create_fence(true).expect("Out of memory");
        let rendering_complete_semaphore = device.create_semaphore().expect("Out of memory");
        Ok(Self {
            instance,
            surface,
            device,
            render_passes: vec![render_pass],
            pipeline_layouts: vec![pipeline_layout],
            pipelines: vec![pipeline],
            command_pool,
            submission_complete_fence,
            rendering_complete_semaphore,
            adapter,
            surface_color_format,
            command_buffer,
            queue_group,
            frame: u64::MIN,
            events: vec![],
        })
    }
}

#[derive(Debug)]
pub struct ResourceHolder(pub Resources<back::Backend>);

impl ResourceHolder {
    pub fn new(window: &Window) -> Result<Self, ()> {
        Ok(Self(Resources::new(window)?))
    }
}

impl Drop for ResourceHolder {
    fn drop(&mut self) {
        self.0.device.wait_idle().unwrap();
        unsafe {
            let Resources {
                instance,
                mut surface,
                device,
                render_passes,
                pipeline_layouts,
                command_pool,
                pipelines,
                submission_complete_fence,
                rendering_complete_semaphore,
                // fences,
                // semaphores,
                ..
            } = &mut self.0;
            device.destroy_semaphore(*rendering_complete_semaphore);
            device.destroy_fence(*submission_complete_fence);
            // for semaphore in semaphores {
            //     device.destroy_semaphore(semaphore);
            // }
            // for fence in fences {
            //     device.destroy_fence(fence);
            // }
            for pipeline in pipelines {
                device.destroy_graphics_pipeline(*pipeline);
            }
            for pipeline_layout in pipeline_layouts {
                device.destroy_pipeline_layout(*pipeline_layout);
            }
            for render_pass in render_passes {
                device.destroy_render_pass(*render_pass);
            }
            device.destroy_command_pool(*command_pool);
            surface.unconfigure_swapchain(&device);
            instance.destroy_surface(surface);
        }
        println!("resources cleared!");
    }
}

unsafe fn make_pipeline<B>(
    device: &B::Device,
    render_pass: &B::RenderPass,
    pipeline_layout: &B::PipelineLayout,
    vertex_shader: &str,
    fragment_shader: &str,
) -> B::GraphicsPipeline
where
    B: gfx_hal::Backend,
{
    let vertex_shader_module = device
        .create_shader_module(&compile_shader(vertex_shader, ShaderKind::Vertex))
        .expect("Failed to create vertex shader module");
    let fragment_shader_module = device
        .create_shader_module(&compile_shader(fragment_shader, ShaderKind::Fragment))
        .expect("Failed to create fragment shader module");
    let (vs_entry, fs_entry) = (
        EntryPoint {
            entry: "main",
            module: &vertex_shader_module,
            specialization: Specialization::default(),
        },
        EntryPoint {
            entry: "main",
            module: &fragment_shader_module,
            specialization: Specialization::default(),
        },
    );
    let primitive_assembler = PrimitiveAssemblerDesc::Vertex {
        buffers: &[],
        attributes: &[],
        input_assembler: InputAssemblerDesc::new(Primitive::TriangleList),
        vertex: vs_entry,
        tessellation: None,
        geometry: None,
    };
    let mut pipeline_desc = GraphicsPipelineDesc::new(
        primitive_assembler,
        Rasterizer {
            cull_face: Face::BACK,
            ..Rasterizer::FILL
        },
        Some(fs_entry),
        pipeline_layout,
        Subpass {
            index: 0,
            main_pass: render_pass,
        },
    );
    pipeline_desc.blender.targets.push(ColorBlendDesc {
        mask: ColorMask::ALL,
        blend: Some(BlendState::ALPHA),
    });
    let pipeline = device
        .create_graphics_pipeline(&pipeline_desc, None)
        .expect("Failed to create graphics pipeline");
    device.destroy_shader_module(vertex_shader_module);
    device.destroy_shader_module(fragment_shader_module);
    pipeline
}

fn compile_shader(glsl: &str, shader_kind: ShaderKind) -> Vec<u32> {
    let mut compiler = Compiler::new().unwrap();
    let compiled_shader = compiler
        .compile_into_spirv(glsl, shader_kind, "unnamed", "main", None)
        .expect("Failed to compile shader");
    compiled_shader.as_binary().to_vec()
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PushConstants {
    pub color: [f32; 4],
    pub pos: [f32; 2],
    pub scale: [f32; 2],
}
