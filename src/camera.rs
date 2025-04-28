use cgmath::InnerSpace;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    eye: [f32; 3],
    lens_radius: f32,
    u_axis: [f32; 3], // camera "right"
    z_near: f32,
    v_axis: [f32; 3], // camera "up"
    z_far: f32,
    w_axis: [f32; 3], // camera looks "backwards"
    projection: u32,
    horizontal: [f32; 3],
    _pad2: f32,
    vertical: [f32; 3],
    _pad3: f32,
    lower_left_corner: [f32; 3],
    _pad4: f32,
}

impl CameraUniform {
    pub fn default() -> CameraUniform {
        CameraUniform {
            eye: (0.0, 0.0, 0.0).into(),
            u_axis: (0.0, 0.0, 0.0).into(),
            v_axis: (0.0, 0.0, 0.0).into(),
            w_axis: (0.0, 0.0, 0.0).into(),
            horizontal: (0.0, 0.0, 0.0).into(),
            vertical: (0.0, 0.0, 0.0).into(),
            lower_left_corner: (0.0, 0.0, 0.0).into(),
            lens_radius: 0.0,
            z_near: 0.0,
            z_far: 0.0,
            projection: 0,

            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aperture: f32,
    pub focus_distance: f32,
    pub projection: Projection,

    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub rot: f32,
    pub t: f32,
}

#[repr(u32)]
#[derive(Clone)]
pub enum Projection {
    Orthograpic = 0,
    Perspective = 1
}

impl Camera {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let uniform = CameraUniform::default();

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
            eye: (0.0, 1.0, 3.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: -cgmath::Vector3::unit_y(),
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 1.0,
            zfar: 100.0,
            rot: 0.0,
            aperture: 0.5,
            focus_distance: 1.0,
            projection: Projection::Perspective,

            uniform,
            buffer,
            bind_group,
            bind_group_layout,
            t: 0.0,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {

        self.t += 0.01;
        // self.eye = (-self.t.sin()*1.0 - 1.0, self.t.sin()*1.0 + 1.0, self.t.sin()*1.0 + 2.0).into();
        // self.fovy = self.t.sin()*45.0 + 90.0;
        self.focus_distance = self.t.sin()*0.5 + 3.5;


        let theta = self.fovy.to_radians();
        let half_height = (theta * 0.5).tan();
        let half_width = self.aspect * half_height;

        let w_axis = (self.eye - self.target).normalize();
        let u_axis = self.up.cross(w_axis).normalize();
        let v_axis = w_axis.cross(u_axis);

        let horizontal = 2.0 * half_width * self.focus_distance * u_axis;
        let vertical = 2.0 * half_height * self.focus_distance * v_axis;
        let lower_left_corner = self.eye - (horizontal * 0.5) - (vertical * 0.5) - (self.focus_distance * w_axis);
        
        self.uniform.w_axis = w_axis.into();
        self.uniform.u_axis = u_axis.into();
        self.uniform.v_axis = v_axis.into();
        self.uniform.eye = self.eye.into();
        self.uniform.lens_radius = self.aperture * 0.5;
        self.uniform.horizontal = horizontal.into();
        self.uniform.vertical = vertical.into();
        self.uniform.lower_left_corner = lower_left_corner.into();
        self.uniform.projection = self.projection.clone() as u32;

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
