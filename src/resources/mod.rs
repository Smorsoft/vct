use crate::Renderer;
// use wgpu_helper::bind_group::BindGroupType;

pub struct DepthBufferResource {
	pub depth_buffer: crate::mesh::Texture,
}

impl crate::Resource for DepthBufferResource {
	fn updated_settings(&mut self, _renderer: &Renderer) {}
}
