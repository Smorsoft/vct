use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glm::Mat4x4 = glm::Mat4x4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


pub struct Camera {
	pub position: glm::Vec3,
	pub rotation: glm::Quat,
	pub aspect: f32,
	pub fovy: f32,
	pub znear: f32,
	pub zfar: f32,


	pub camera_buffer: wgpu::Buffer,
	pub camera_bind_group: wgpu::BindGroup,
}

impl Camera {
	pub fn new(device: &mut wgpu::Device,  bind_group_layout: &mut wgpu::BindGroupLayout, position: glm::Vec3, fovy: f32, aspect: f32, zfar: f32, znear: f32, rotation: glm::Quat) -> Self {
		let camera_uniform = CameraUniform::new();
		let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Camera Buffer"),
			contents: bytemuck::cast_slice(&[camera_uniform]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: camera_buffer.as_entire_binding(),
			}],
			label: Some("camera_bind_group"),
		});


		
		Self {
			position,
			rotation,
			fovy,
			aspect,
			zfar,
			znear,
			camera_buffer,
			camera_bind_group,
		}
	}

	pub fn build_view_projection_matrix(&self) -> glm::Mat4x4 {
		let mut view = glm::translate(&glm::Mat4x4::identity(), &self.position);
		view = view * glm::quat_to_mat4(&self.rotation);

		let proj = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);
	
		return OPENGL_TO_WGPU_MATRIX * proj * view;
	}

	pub fn update(&self, queue: &mut wgpu::Queue) {
		let mut camera_uniform = CameraUniform::new();

		camera_uniform.update_view_proj(&self);

		queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
	pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
	pub fn new() -> Self {
		Self {
			view_proj: glm::Mat4x4::identity().into(),
		}
	}

	pub fn filled(camera: &Camera) -> Self {
		Self {
			view_proj: camera.build_view_projection_matrix().into(),
		}
	}

	pub fn update_view_proj(&mut self, camera: &Camera) {
		self.view_proj = camera.build_view_projection_matrix().into();
	}
}
