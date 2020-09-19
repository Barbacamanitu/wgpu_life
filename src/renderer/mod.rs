use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use std::borrow::Cow;
use wgpu::{Surface,Adapter,DeviceDescriptor};
pub struct Renderer {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    size: winit::dpi::PhysicalSize<u32>,
}

struct ShaderModuleSet {
    vertex: wgpu::ShaderModule,
    fragment: wgpu::ShaderModule
}

impl Renderer {


    fn create_shader_modules(device: &wgpu::Device) -> ShaderModuleSet {
        let vs_src = include_str!("../../shaders/shader.vert");
        let fs_src = include_str!("../../shaders/shader.frag");
        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler.compile_into_spirv(vs_src, shaderc::ShaderKind::Vertex, "shader.vert", "main", None).unwrap();
        let fs_spirv = compiler.compile_into_spirv(fs_src, shaderc::ShaderKind::Fragment, "shader.frag", "main", None).unwrap();
        let vs_data = Cow::from(vs_spirv.as_binary());
        let fs_data = Cow::from(fs_spirv.as_binary());
  
        
        let vs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(vs_data));
        let fs_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(fs_data));
        ShaderModuleSet {
            vertex: vs_module,
            fragment: fs_module
        }
    }

    fn create_pipeline(device : &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> wgpu::RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mypipeline"),
            push_constant_ranges: &[],
            bind_group_layouts: &[],
        });

        let shader_modules = Renderer::create_shader_modules(&device);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("the pipe"),
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
                front_face : wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                clamp_depth: false,
                depth_bias_clamp: 0.0,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[]
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
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        (sc_desc,swap_chain)
    }

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe {instance.create_surface(window)};

        let adapter = Renderer::get_adapter(&instance, &surface).await;
        let (device,queue) = Renderer::get_device_and_queue(&adapter).await;
        let (sc_desc,swap_chain) = Renderer::create_swapchain(&device, &surface, size);

        let render_pipeline = Renderer::create_pipeline(&device,&sc_desc);
        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            size,
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
        match frame {
            Ok(frame) => {
                let frame_tex = frame.output;
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        
                {
                    
                let clear_color : wgpu::Color = wgpu::Color{r: 1.0, g: 0.0, b: 0.0, a: 1.0};
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
                render_pass.draw(0..3, 0..1);
                
            }
            self.queue.submit(
                Some(encoder.finish())
            );
        },
            Err(_) => {
                println!("Error getting frame.");
            }
        
        }
        
        
    
       
        
    }
}