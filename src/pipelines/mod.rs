// use mesh_pipeline::MeshPipeline;
use medical_pipeline::MedicalPipeline;
// use raytrace_pipeline::RaytracePipeline;
use sampletexture_pipeline::SampleTexturePipeline;

use crate::camera::Camera;

pub mod medical_pipeline;
pub mod mesh_pipeline;
pub mod raytrace_pipeline;
pub mod sampletexture_pipeline;
pub mod triangle_pipeline;

pub struct Pipelines {
    // mesh_pipeline: MeshPipeline,
    // raytrace_pipeline: RaytracePipeline,
    medical_pipeline: MedicalPipeline,
    sample_pipeline: SampleTexturePipeline,
}

impl Pipelines {
    pub fn new(
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera,
    ) -> Self {
        // let mesh_pipeline = MeshPipeline::new(&surface_config, &device, &camera);
        // let raytrace_pipeline = RaytracePipeline::new(&surface_config, &device, &camera);
        let medical_pipeline = MedicalPipeline::new(&surface_config, &device, &queue, &camera);
        let sample_pipeline =
            SampleTexturePipeline::new(&surface_config, &device, medical_pipeline.create_view());

        Pipelines {
            // mesh_pipeline,
            // raytrace_pipeline,
            medical_pipeline,
            sample_pipeline,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        output_view: &wgpu::TextureView,
        _multisample_framebuffer_view: &wgpu::TextureView,
        _depthbuffer_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder, // _camera: &Camera,
    ) {
        // self.mesh_pipeline.pass(
        //     output_view,
        //     multisample_framebuffer_view,
        //     depthbuffer_view,
        //     encoder,
        //     camera,
        // );
        // self.raytrace_pipeline.pass(encoder);
        self.medical_pipeline.pass(encoder);
        self.sample_pipeline.pass(device, output_view, encoder);
    }
}
