use cgmath::{InnerSpace, Vector3};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use instant::Instant;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Default, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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

pub struct Camera {
    pub eye: cgmath::Vector3<f32>,
    pub target: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aperture: f32,
    pub focus_distance: f32,
    pub projection: Projection,

    // Dolly camera state
    yaw: f32,
    pitch: f32,
    move_speed: f32,       // units per second
    look_sensitivity: f32, // radians per second at full deflection
    last_update: Instant,

    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,

    gilrs: Gilrs,
    dpaddown: bool,
    dpadup: bool,
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,

    // Mouse interaction state
    mouse_right_down: bool,
    mouse_middle_down: bool,
    mouse_dx: f32,
    mouse_dy: f32,
    scroll: f32,
    mouse_look_sensitivity: f32, // radians per pixel
    mouse_pan_sensitivity: f32,  // units per pixel
    scroll_speed: f32,           // units per wheel unit
    invert_look_y: bool,
    invert_pan_y: bool,
    invert_look_x: bool,
    invert_pan_x: bool,
}

#[repr(u32)]
#[derive(Clone)]
pub enum Projection {
    Orthograpic = 0,
    Perspective = 1,
}

// Dolly-style camera (first-person-like) implemented in update()

impl Camera {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let gilrs = Gilrs::new().unwrap();
        for (_id, gamepad) in gilrs.gamepads() {
            println!("{} is {:?}", gamepad.name(), gamepad.power_info());
        }

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

        // Start in a common 3D view facing -Z
        let eye: Vector3<f32> = (0.0, 0.8, 2.5).into();
        let target: Vector3<f32> = (0.0, 0.6, 1.5).into();
        let up = -cgmath::Vector3::unit_y();

        // Derive initial yaw/pitch from eye/target
        let fwd = (target - eye).normalize();
        let yaw = fwd.z.atan2(fwd.x);
        let pitch = fwd.y.asin();

        Self {
            eye,
            target,
            up,
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 1.0,
            zfar: 1000.0,
            aperture: 0.5,
            focus_distance: 1.0,
            projection: Projection::Perspective,

            yaw,
            pitch,
            move_speed: 0.8,
            look_sensitivity: 0.8,
            last_update: Instant::now(),

            uniform,
            buffer,
            bind_group,
            bind_group_layout,

            gilrs,
            dpaddown: false,
            dpadup: false,
            left_x: 0.0,
            right_x: 0.0,
            left_y: 0.0,
            right_y: 0.0,

            mouse_right_down: false,
            mouse_middle_down: false,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            scroll: 0.0,
            mouse_look_sensitivity: 0.0035,
            mouse_pan_sensitivity: 0.003,
            scroll_speed: 0.6,
            invert_look_y: true,
            invert_pan_y: true,
            invert_look_x: true,
            invert_pan_x: true,
        }
    }

    pub fn mouse_motion(&mut self, dx: f32, dy: f32) {
        self.mouse_dx += dx;
        self.mouse_dy += dy;
    }

    pub fn mouse_buttons(&mut self, right: bool, middle: bool) {
        self.mouse_right_down = right;
        self.mouse_middle_down = middle;
    }

    pub fn mouse_wheel(&mut self, delta: f32) {
        self.scroll += delta;
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.aperture = 0.0;

        // Delta time
        let now = Instant::now();
        let mut dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;
        if !dt.is_finite() || dt <= 0.0 { dt = 1.0 / 60.0; }
        dt = dt.min(0.05);

        // Read controller state
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::ButtonPressed(Button::DPadDown, _) => self.dpaddown = true,
                EventType::ButtonReleased(Button::DPadDown, _) => self.dpaddown = false,
                EventType::ButtonPressed(Button::DPadUp, _) => self.dpadup = true,
                EventType::ButtonReleased(Button::DPadUp, _) => self.dpadup = false,

                EventType::AxisChanged(Axis::LeftStickX, val, _) => self.left_x = val,
                EventType::AxisChanged(Axis::LeftStickY, val, _) => self.left_y = val,
                EventType::AxisChanged(Axis::RightStickX, val, _) => self.right_x = val,
                EventType::AxisChanged(Axis::RightStickY, val, _) => self.right_y = val,
                _ => {}
            }
        }

        // Deadzones
        let dz = 0.12;
        let lx = if self.left_x.abs() < dz { 0.0 } else { self.left_x };
        let ly = if self.left_y.abs() < dz { 0.0 } else { self.left_y };
        let rx = if self.right_x.abs() < dz { 0.0 } else { self.right_x };
        let ry = if self.right_y.abs() < dz { 0.0 } else { self.right_y };

        // Look: controller + mouse (right button drag)
        let mut yaw_delta = rx * self.look_sensitivity * dt;
        let mut pitch_delta = ry * self.look_sensitivity * dt;
        if self.mouse_right_down {
            let x_sign = if self.invert_look_x { -1.0 } else { 1.0 };
            yaw_delta += (self.mouse_dx * x_sign) * self.mouse_look_sensitivity;
            let y_sign = if self.invert_look_y { -1.0 } else { 1.0 };
            pitch_delta += (self.mouse_dy * y_sign) * self.mouse_look_sensitivity;
        }
        self.yaw += yaw_delta;
        self.pitch += pitch_delta;
        let max_pitch = 1.54;
        if self.pitch > max_pitch { self.pitch = max_pitch; }
        if self.pitch < -max_pitch { self.pitch = -max_pitch; }

        // Basis from yaw/pitch
        let cp = self.pitch.cos();
        let forward = Vector3::new(self.yaw.cos() * cp, self.pitch.sin(), self.yaw.sin() * cp).normalize();
        let world_up = -self.up;
        let mut right = forward.cross(world_up);
        if right.magnitude2() < 1e-6 { right = Vector3::new(1.0, 0.0, 0.0); }
        let right = right.normalize();

        // Move: controller left stick (forward/back + strafe)
        let mut move_dir = forward * (-ly) + right * lx;
        let len2 = move_dir.magnitude2();
        if len2 > 1.0 { move_dir /= len2.sqrt(); }
        let controller_move = move_dir * (self.move_speed * dt);

        // Vertical via D-pad
        let mut vertical_move = Vector3::new(0.0, 0.0, 0.0);
        if self.dpadup { vertical_move += world_up * (self.move_speed * dt); }
        if self.dpaddown { vertical_move -= world_up * (self.move_speed * dt); }

        // Mouse pan with middle button: right/left and up/down in world space
        let mut pan_move = Vector3::new(0.0, 0.0, 0.0);
        if self.mouse_middle_down {
            let x_sign = if self.invert_pan_x { -1.0 } else { 1.0 };
            pan_move += right * ((self.mouse_dx * x_sign) * self.mouse_pan_sensitivity);
            let y_sign = if self.invert_pan_y { -1.0 } else { 1.0 };
            pan_move += world_up * ((self.mouse_dy * y_sign) * self.mouse_pan_sensitivity);
        }

        // Mouse wheel dolly along forward
        let dolly_move = forward * (self.scroll * self.scroll_speed);

        // Apply combined movement
        self.eye += controller_move + vertical_move + pan_move + dolly_move;

        // Keep target ahead of eye
        self.target = self.eye + forward * self.focus_distance;

        let theta = self.fovy.to_radians();
        let half_height = (theta * 0.5).tan();
        let half_width = self.aspect * half_height;

        let w_axis = (self.eye - self.target).normalize();
        let u_axis = self.up.cross(w_axis).normalize();
        let v_axis = w_axis.cross(u_axis);

        let horizontal = 2.0 * half_width * self.focus_distance * u_axis;
        let vertical = 2.0 * half_height * self.focus_distance * v_axis;
        let lower_left_corner =
            self.eye - (horizontal * 0.5) - (vertical * 0.5) - (self.focus_distance * w_axis);

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

        // Reset one-shot mouse deltas
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
        self.scroll = 0.0;
    }
}
