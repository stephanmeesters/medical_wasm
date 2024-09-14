use triangle::TrianglePipeline;

pub mod triangle;

pub struct Pipelines {
    triangle_pipeline: TrianglePipeline,
}

impl Pipelines {
    pub fn new(surface_config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) -> Self {
        let triangle_pipeline = TrianglePipeline::new(&surface_config, &device);

        Pipelines { triangle_pipeline }
    }

    pub fn render(
        &self,
        surface_config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.triangle_pipeline
            .pass(surface_config, device, queue, output_view, encoder);
    }
}
