use winit::{
    event::*,
    window::{Window},
};
use wgpu::{BlendOperation,BlendFactor,util::DeviceExt};
pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
    vert_index_buffers: VertexAndIndexBuffer,
}

struct ShaderModuleSet {
    vertex: wgpu::ShaderModule,
    fragment: wgpu::ShaderModule
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

struct VertexAndIndexBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32
}


unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[ // 3.
            wgpu::VertexAttributeDescriptor {
                offset: 0, // 4.
                shader_location: 0, // 5.
                format: wgpu::VertexFormat::Float3, // 6.
            },
            wgpu::VertexAttributeDescriptor {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float3,
            }
            ]
        }
    }
}

impl Renderer {

    fn create_shader_modules(device: &wgpu::Device) -> ShaderModuleSet {
        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shaders/shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shaders/shader.frag.spv"));
        ShaderModuleSet {
            vertex: vs_module,
            fragment: fs_module
        }
    }

    fn create_pipeline(device : &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> wgpu::RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout 1"),
            push_constant_ranges: &[],
            bind_group_layouts: &[],
        });

        let shader_modules = Renderer::create_shader_modules(&device);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline 1"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shader_modules.vertex,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shader_modules.fragment,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face : wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                clamp_depth: false,
                depth_bias_clamp: 0.0,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,                 
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,                 
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    Vertex::desc()
                ]
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,

        });
        render_pipeline
        
    }

    async fn get_adapter(instance: &wgpu::Instance,surface: &wgpu::Surface) -> wgpu::Adapter {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();
        adapter
    }

    async fn get_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device,wgpu::Queue) {
        let optional_features = wgpu::Features::empty();
        let required_features = wgpu::Features::empty();
        let adapter_features = adapter.features();
        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: (optional_features & adapter_features) | required_features,
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();
        (device,queue)
    }

    fn create_swapchain(device: &wgpu::Device,surface: &wgpu::Surface, size: winit::dpi::PhysicalSize<u32>) -> (wgpu::SwapChainDescriptor,wgpu::SwapChain) {
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        (sc_desc,swap_chain)
    }

    fn get_vertices(device: &wgpu::Device) -> VertexAndIndexBuffer {
        const VERTICES: &[Vertex] = &[
            Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
            Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
            Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
            Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
            Vertex { position: [0.44147372, 0.2347359, 0.0],color: [0.5, 0.0, 0.5] }, // E
        ];

        const INDICES: &[u16] = &[
            0, 1, 4,
            1, 2, 4,
            2, 3, 4,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;
        println!("Number indices: {}",num_indices);
        let num_vertices = VERTICES.len() as u32;
        VertexAndIndexBuffer {
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices
        }
    }

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe {instance.create_surface(window)};

        let adapter = Renderer::get_adapter(&instance, &surface).await;
        let (device,queue) = Renderer::get_device_and_queue(&adapter).await;
        let (sc_desc,swap_chain) = Renderer::create_swapchain(&device, &surface, size);

        let render_pipeline = Renderer::create_pipeline(&device,&sc_desc);
        
        let vert_index_buffers = Renderer::get_vertices(&device); 
       
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            size,
            vert_index_buffers,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("Resized: {:?}",new_size);
        self.size = new_size;
        let (sc_desc,swap_chain) = Renderer::create_swapchain(&self.device, &self.surface, self.size);
        self.sc_desc = sc_desc;
        self.swap_chain = swap_chain;
    }

    // input() won't deal with GPU code, so it can be synchronous
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
       // println!("update");
    }

    pub fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame();
        if let Ok(f) = frame {
            let frame_tex = f.output;
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
            });

            { //Begin scope to fix borrow problem. Encoder gets borrowed mutable at .begin_render_pass
                let clear_color : wgpu::Color = wgpu::Color{r: 0.1, g: 0.15, b: 0.8, a: 1.0};
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[
                            wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame_tex.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(clear_color),
                                    store: true,
                                },
                            }
                        ],
                        depth_stencil_attachment: None,
                    });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, self.vert_index_buffers.vertex_buffer.slice(..));

                render_pass.set_index_buffer(self.vert_index_buffers.index_buffer.slice(..));
                render_pass.draw_indexed(0..self.vert_index_buffers.num_indices, 0,0..1);
            
            } //end scope for borrow problem. Now encoder is no longer borrowed mutably.

            self.queue.submit(Some(encoder.finish()));

        } else {
            println!("Error getting next frame...");
        }      
        
    }//End render
}