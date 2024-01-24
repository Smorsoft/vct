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
}

impl Camera {
	pub fn new(
		position: glm::Vec3,
		fovy: f32,
		aspect: f32,
		zfar: f32,
		znear: f32,
		rotation: glm::Quat,
	) -> Self {
		Self {
			position,
			rotation,
			fovy,
			aspect,
			zfar,
			znear,
		}
	}

	pub fn build_view_projection_matrix(&self) -> glm::Mat4x4 {
		let mut view = glm::translate(&glm::Mat4x4::identity(), &self.position);
		view = glm::quat_to_mat4(&self.rotation) * view;

		let proj = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);

		return OPENGL_TO_WGPU_MATRIX * proj * view;
	}

	pub fn update(&self, queue: &mut wgpu::Queue, camera_buffer: &mut wgpu::Buffer) {
		let mut camera_uniform = CameraUniform::new();

		camera_uniform.update_view_proj(&self);

		queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
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
