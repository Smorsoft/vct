pub struct Mesh {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub transform_buffer: wgpu::Buffer,
	pub model_bind_group: wgpu::BindGroup,
	pub primitives: Vec<Primitive>,
	// materials: Vec<>,
}

pub struct Primitive {
	pub start: u64,
	pub end: u64,
	pub vertex_start: u64,
	pub vertex_end: u64,
}


pub type Index = u32;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexAttributes {
	pub position: [f32; 3],
	pub tangent: [f32; 4],
	pub normal: [f32; 3],
	pub color0: [f32; 3],
	pub tex_coord0: [f32; 2],
	pub tex_coord1: [f32; 2],
}

impl VertexAttributes {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			0 => Float32x3,
			1 => Float32x4,
			2 => Float32x3,
			3 => Float32x3,
			4 => Float32x2,
			5 => Float32x2
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexAttributes>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}
