pub struct Mesh {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub transform_buffer: wgpu::Buffer,
	pub model_bind_group: wgpu::BindGroup,
	pub primitives: Vec<Primitive>,
}

pub struct Primitive {
	pub start: u64,
	pub end: u64,
	pub vertex_start: u64,
	pub vertex_end: u64,
	pub material: usize,
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

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexAttributes {
	pub position: [f32; 3],
	pub tangent: [f32; 4],
	pub normal: [f32; 3],
	pub tex_coord0: [f32; 2],
}

impl VertexAttributes {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			0 => Float32x3,
			1 => Float32x4,
			2 => Float32x3,
			// 3 => Float32x3,
			3 => Float32x2,
			// 5 => Float32x2
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexAttributes>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}
