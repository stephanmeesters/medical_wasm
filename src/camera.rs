use cgmath::InnerSpace;
use wgpu::util::DeviceExt;

// #[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // model_view_proj: [[f32; 4]; 4], // 16x4 = 64
    position: [f32; 3],
    _pad1: f32,        // Padding of 4 bytes
    direction: [f32; 3],
    _pad2: f32,        // Padding of 4 bytes
    up: [f32; 3],
    _pad3: f32,        // Padding of 4 bytes
    side: [f32; 3],
    _pad4: f32,        // Padding of 4 bytes
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub rot: f32,
    pub t: f32
}

impl Camera {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        // use cgmath::SquareMatrix;
        let uniform = CameraUniform {
            // model_view_proj: cgmath::Matrix4::identity().into(),
            direction: (0.0, 0.0, 5.0).into(),
            position: (0.0, 0.0, 5.0).into(),
            up: (0.0, 0.0, 1.0).into(),
            side: (0.0, 0.0, 1.0).into(),
            // padding: (0.0, 0.0, 0.0, 0.0).into()
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            eye: (0.0, 0.0, 5.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 1.0,
            zfar: 100.0,
            rot: 0.0,
            uniform,
            buffer,
            bind_group,
            bind_group_layout,
            t: 0.0
        }
    }

    fn build_view_projection_matrix(&mut self) -> cgmath::Matrix4<f32> {
        self.t += 1.0;
        self.rot = f32::sin(self.t * 0.01)*0.4;
        // let mut dist = 3.0 + 1.0 * f32::sin(self.rot);
        // dist *= 0.5;
        let dist = 1.0;
        let elev = 0.0;
        self.eye = cgmath::point3(f32::sin(self.rot) * dist, elev, f32::cos(self.rot) * dist);
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
        // println!("{:?}", view * cgmath::vec4(0.0, 0.0, 0.0, 1.0));
        // view
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.build_view_projection_matrix();
        // self.uniform.model_view_proj = (OPENGL_TO_WGPU_MATRIX * self.build_view_projection_matrix()).into();
        self.uniform.position = self.eye.into();
        self.uniform.direction = (self.target - self.eye).normalize().into();
        self.uniform.up = self.up.into();
        self.uniform.side = self.up.cross(cgmath::vec3(self.eye.x, self.eye.y, self.eye.z)).normalize().into();
        // println!("{:?}", self.uniform);
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}
