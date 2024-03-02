use crate::Renderer;
// use wgpu_helper::bind_group::BindGroupType;

pub struct DepthBufferResource {
	pub depth_buffer: crate::mesh::Texture,
}

impl crate::Resource for DepthBufferResource {
	fn updated_settings(&mut self, renderer: &Renderer) {
		let res = renderer.get_resolution();

		let size = wgpu::Extent3d {
			width: res[0],
			height: res[1],
			depth_or_array_layers: 1,
		};

		let desc = wgpu::TextureDescriptor {
			label: Some("Depth Buffer"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: crate::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		};

		let texture = renderer.device().create_texture(&desc);

		let view = texture.create_view(&Default::default());
		let sampler = renderer.device().create_sampler(&wgpu::SamplerDescriptor {
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

		self.depth_buffer = crate::mesh::Texture {
			texture,
			view,
			sampler,
		};
	}
}
