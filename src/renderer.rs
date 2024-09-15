use std::iter;

use wgpu::MultisampleState;
use winit::window::Window;

use crate::{
    fpscounter::{self, FPSCounter},
    pipelines::Pipelines,
};

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: Pipelines,
    fpscounter: FPSCounter,
    multisample_framebuffer: wgpu::TextureView
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("cant request adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("cant create device");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let pipelines = Pipelines::new(&surface_config, &device);

        let fpscounter = FPSCounter::new();

        let multisample_framebuffer = Renderer::create_multisampled_framebuffer(&device, &surface_config, 4);

        Self {
            surface,
            device,
            queue,
            surface_config,
            pipelines,
            fpscounter,
            multisample_framebuffer
        }
    }

    pub fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        sc_desc: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Multisampled Framebuffer"),
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        multisampled_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            ..wgpu::TextureViewDescriptor::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.pipelines.render(
            &self.surface_config,
            &self.device,
            &self.queue,
            &output_view,
            &self.multisample_framebuffer,
            &mut encoder,
        );

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.fpscounter.tick();

        Ok(())
    }

    pub fn get_fps(&self) -> String {
        self.fpscounter.print()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}
