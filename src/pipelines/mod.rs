use mesh_pipeline::MeshPipeline;
use raytrace_pipeline::RaytracePipeline;

use crate::camera::Camera;

pub mod mesh_pipeline;
pub mod raytrace_pipeline;
pub mod triangle_pipeline;

pub struct Pipelines {
    mesh_pipeline: MeshPipeline,
    raytrace_pipeline: RaytracePipeline,
}

impl Pipelines {
    pub fn new(
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        camera: &Camera,
    ) -> Self {
        let mesh_pipeline = MeshPipeline::new(&surface_config, &device, &camera);
        let raytrace_pipeline = RaytracePipeline::new(&device);

        Pipelines {
            mesh_pipeline,
            raytrace_pipeline,
        }
    }

    pub fn render(
        &self,
        output_view: &wgpu::TextureView,
        multisample_framebuffer_view: &wgpu::TextureView,
        depthbuffer_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
    ) {
        self.mesh_pipeline.pass(
            output_view,
            multisample_framebuffer_view,
            depthbuffer_view,
            encoder,
            camera,
        );
        self.raytrace_pipeline.pass(encoder);
    }
}
