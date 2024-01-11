use std::ops::Range;

pub struct Mesh {
	pub vertex_buffer: wgpu::Buffer,
	pub positions: Range<wgpu::BufferAddress>,
	pub normals: Range<wgpu::BufferAddress>,
	pub colors: Range<wgpu::BufferAddress>,
	pub index_buffer: wgpu::Buffer,
	pub transform_buffer: wgpu::Buffer,
	pub model_bind_group: wgpu::BindGroup,
	pub primitives: Vec<Primitive>,
}

pub struct Primitive {
	pub index: Range<u32>,
	// pub skin: Range<wgpu::BufferAddress>,
	pub material: uuid::Uuid,
}

pub struct Material {
	pub diffuse: Texture,
	pub metallic_roughness: Texture,
	pub normal: Texture,
	pub bind_group: wgpu::BindGroup,
}

pub struct Texture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

impl Texture {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	pub fn create_depth_texture(
		device: &wgpu::Device,
		config: &wgpu::SurfaceConfiguration,
	) -> Self {
		let size = wgpu::Extent3d {
			width: config.width,
			height: config.height,
			depth_or_array_layers: 1,
		};

		let desc = wgpu::TextureDescriptor {
			label: Some("Depth Buffer"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		};

		let texture = device.create_texture(&desc);

		let view = texture.create_view(&Default::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: 0.0,
			lod_max_clamp: 100.0,
			..Default::default()
		});

		Self {
			texture,
			view,
			sampler,
		}
	}
}

pub type Index = u32;

pub struct VertexAttributes {
	// pub
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPosition([f32; 3]);

impl VertexPosition {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			3 => Float32x2,
			4 => Float32x2,
			5 => Uint32,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexColors>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &ATTRIBUTES,
		}
	}
}

// #[repr(C)]
// pub struct VertexSkinning {
// 	pub joint_index: [u16; 4],
// 	pub joint_weight: [u16; 4],
// }
