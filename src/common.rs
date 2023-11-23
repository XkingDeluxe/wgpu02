use wgpu::{IndexFormat, PrimitiveTopology, ShaderSource};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window
};

pub struct Inputs<'a> {
    pub source : ShaderSource<'a>,
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
}
pub async fn run(event_loop: EventLoop<()>, window: Window, inputs: Inputs<'_>, num_vertices: u32){
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let surface = unsafe {
        instance.create_surface(&window)
    };
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await.expect("Can't find right adapter");

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
        label: None,
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::default(),
    }, None).await.expect("Can't create device");

    let format = surface.get_preferred_format(&adapter).unwrap();
    let mut config = wgpu::SurfaceConfiguration{
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: inputs.source,
    });

    let pipline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipline_layout),
        vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[] },
        fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", targets: &[format.into()] }),
        primitive: wgpu::PrimitiveState{
            topology:inputs.topology,
            strip_index_format: inputs.strip_index_format,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter, &shader, &pipline_layout);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { window_id: window, event: WindowEvent::Resized(size),
            } => {
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
            }
            Event::RedrawRequested(_) => {
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: None});
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment{
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color{r: 0.0, g: 0.0, b: 0.0, a:0.0}),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..num_vertices, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();

            }
            Event::WindowEvent { 
                window_id: window, event: WindowEvent::CloseRequested, 
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    })
}