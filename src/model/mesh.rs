use core::ops::Range;

pub type Index = u32;

pub struct Mesh {
	pub index_buffer: wgpu::Buffer,
	pub vertex_buffer: wgpu::Buffer,
	pub positions: Range<wgpu::BufferAddress>,
	pub normals: Range<wgpu::BufferAddress>,
	pub colors: Range<wgpu::BufferAddress>,
	pub transform_buffer: wgpu::Buffer,
	pub model_bind_group: wgpu::BindGroup,
	pub primitives: Vec<Primitive>,
}

pub struct Primitive {
	pub index: Range<u32>,
	pub material: uuid::Uuid,
}


#[repr(transparent)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPosition([f32; 3]);

impl VertexPosition {
	pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			0 => Float32x3,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexPosition>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexNormals {
	pub normals: [i8; 4],
	pub tangents: [i8; 4],
}

impl VertexNormals {
	pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			1 => Uint32,
			2 => Uint32,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexNormals>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}

/// TODO: Figure out why uv0 cant be unpacked as u16 from u32 
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexColors {
	pub uv0: [f32; 2],
	pub uv1: [f32; 2],
	pub color: [u8; 4],
}

impl VertexColors {
	pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			3 => Float32x2,
			4 => Float32x2,
			5 => Uint32,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexColors>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}

#[repr(C)]
pub struct VertexSkinning {
	pub joint_index: [u16; 4],
	pub joint_weight: [u16; 4],
}

pub trait LoadMesh<'a> {
	type VertexPositionIterator: Iterator<Item=&'a VertexPosition>;
	type VertexNormalsIterator: Iterator<Item=&'a VertexNormals>;
	type VertexColorIterator: Iterator<Item=&'a VertexColors>;
	type VertexSkinningIterator: Iterator<Item=&'a VertexSkinning>;
	type PrimitiveRangeIterator: Iterator<Item=Primitive>;

	fn get_mesh(&self) -> Mesh {
		todo!()
	}

	fn get_num_vertices(&self) -> usize;
	fn get_num_indices(&self) -> usize;
	fn get_vertex_position_iterator(&'a self) -> Self::VertexPositionIterator;
	fn get_vertex_normal_iterator(&'a self) -> Self::VertexNormalsIterator;
	fn get_vertex_color_iterator(&'a self) -> Self::VertexColorIterator;
	fn get_vertex_skinning_iterator(&'a self) -> Self::VertexSkinningIterator;

	fn get_primitives(&'a self) -> Self::PrimitiveRangeIterator;
}