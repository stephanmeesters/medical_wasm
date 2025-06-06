use crate::camera::Camera;
use wgpu::util::DeviceExt;

pub struct RaytracePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Sphere {
    pos: [f32; 3],
    radius: f32,
    material: [f32; 4] 
}

impl RaytracePipeline {
    pub fn create_view(&self) -> wgpu::TextureView {
        self.texture.create_view(&Default::default())
    }

    pub fn new(surface_config: &wgpu::SurfaceConfiguration,
            device: &wgpu::Device,
            camera: &Camera) -> Self {

        let spheres = vec![
            Sphere { pos: [-0.5, 0.0, -1.0], radius:0.5, material: [1.0, 0.0, 0.0, 1.0]},
            Sphere { pos: [0.5, 0.0, 0.0], radius:0.5, material: [0.0, 0.0, 1.0, 1.0]},
            Sphere { pos: [0.0, -25.5, -1.0], radius:25.0, material: [1.0, 1.0, 1.0, 1.0]},
        ];

        let spheres_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spheres Buffer"),
            contents: bytemuck::cast_slice(&spheres),
            usage: wgpu::BufferUsages::STORAGE
        });

        let details = vec![surface_config.width as f32, surface_config.height as f32];
        let details_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("details buffer"),
            contents: bytemuck::cast_slice(&details),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        ////

        let shader_common_src = include_str!("shaders/common.wgsl");
        let shader_raytracing_src = include_str!("shaders/raytrace.wgsl");
        let shader_combined = format!("{}\n{}", shader_common_src, shader_raytracing_src);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader_ray"),
            source: wgpu::ShaderSource::Wgsl(shader_combined.into()),
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: Default::default(),
        });

        let texture_view = texture.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ray bind group layout"),
            entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: texture.format(),
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None,
            }]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: camera.buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: spheres_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: details_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raytrace Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            texture
        }
    }

    pub fn pass(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups((self.texture.width() + 7) / 8, (self.texture.height() + 7) / 8, 1);
    }
}
