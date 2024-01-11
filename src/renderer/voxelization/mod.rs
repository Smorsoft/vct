use std::pin::Pin;
use wgpu::util::DeviceExt;

use super::context::GraphicsContext;

mod meshify;
mod textures;

pub struct Voxelization {
	voxel_color: super::mesh::Texture,
	meshify: meshify::Meshify,
}

impl Voxelization {
	pub fn new(context: &Pin<Box<GraphicsContext>>) -> Self {
		let meshify = meshify::Meshify::new(context);

		let size = wgpu::Extent3d {
			width: 512,
			height: 512,
			depth_or_array_layers: 512,
		};

		let sampler_descriptor = wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: 0.0,
			lod_max_clamp: 100.0,
			..Default::default()
		};

		let voxel_color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Voxel Color Texture"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D3,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		});

		let voxel_color_view = voxel_color_texture.create_view(&Default::default());
		let voxel_color_sampler = context.device.create_sampler(&sampler_descriptor);

		let voxel_color = super::mesh::Texture {
			texture: voxel_color_texture,
			view: voxel_color_view,
			sampler: voxel_color_sampler,
		};

		

		Self {
			voxel_color,
			meshify,
		}
	}

	pub fn voxelize(&mut self, context: &Pin<Box<GraphicsContext>>) {

	}

	pub fn meshify(&mut self, context: &Pin<Box<GraphicsContext>>) {
		self.meshify.meshify(context, &self.voxel_color);
	}

	pub fn render(
		&mut self,
		context: &Pin<Box<GraphicsContext>>,
		depth_buffer: &crate::renderer::mesh::Texture,
		camera_bind_group: &wgpu::BindGroup,
	) {
		self.meshify
			.render(context, depth_buffer, camera_bind_group);
	}
}
