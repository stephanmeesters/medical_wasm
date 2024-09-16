use mesh_pipeline::MeshPipeline;
use triangle_pipeline::TrianglePipeline;

use crate::camera::Camera;

pub mod mesh_pipeline;
pub mod triangle_pipeline;

pub struct Pipelines {
    triangle_pipeline: TrianglePipeline,
    mesh_pipeline: MeshPipeline,
}

impl Pipelines {
    pub fn new(
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        camera: &Camera,
    ) -> Self {
        let triangle_pipeline = TrianglePipeline::new(&surface_config, &device);
        let mesh_pipeline = MeshPipeline::new(&surface_config, &device, &camera);

        Pipelines {
            triangle_pipeline,
            mesh_pipeline,
        }
    }

    pub fn render(
        &self,
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        multisample_framebuffer_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
    ) {
        self.triangle_pipeline
            .pass(surface_config, device, queue, output_view, encoder);
        self.mesh_pipeline
            .pass(output_view, multisample_framebuffer_view, encoder, camera);
    }
}
