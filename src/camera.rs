use std::{borrow::BorrowMut, rc::Rc};

use wgpu_helper::{
	bind_group::{BindGroup, BindGroupType},
	*,
};

#[derive(Copy, Clone)]
pub enum Resolution {
	UseGlobalResolution,
	Custom([u32; 2]),
}

pub const CAMERA_BIND_GROUP_LAYOUT: &'static wgpu::BindGroupLayoutDescriptor =
	&wgpu::BindGroupLayoutDescriptor {
		label: Some("Camera Bind group layout"),
		entries: &[wgpu::BindGroupLayoutEntry {
			binding: 0,
			visibility: wgpu::ShaderStages::VERTEX,
			ty: wgpu::BindingType::Buffer {
				ty: wgpu::BufferBindingType::Uniform,
				has_dynamic_offset: false,
				min_binding_size: core::num::NonZeroU64::new(
					core::mem::size_of::<types::mat4x4f>() as u64,
				),
			},
			count: None,
		}],
	};

#[derive(BindGroup)]
#[layout(CAMERA_BIND_GROUP_LAYOUT)]
pub struct CameraBindGroup<'a> {
	pub camera: &'a Buffer<types::mat4x4f>,
}

pub struct Camera {
	pub(crate) renderer: Rc<crate::InternalRenderer>,
	pub(crate) id: crate::Id,
}

impl Camera {
	pub(crate) fn new(renderer: Rc<crate::InternalRenderer>, id: crate::Id) -> Self {
		Self { renderer, id }
	}

	pub fn get_resolution(&self) -> [u32; 2] {
		let inner = &self.renderer.cameras.get(&self.id).unwrap();

		match inner.resolution {
			Resolution::Custom(res) => res,
			Resolution::UseGlobalResolution => {
				self.renderer.get_resolution()
			}
		}
	}

	pub fn new_depth_buffer(&self) -> crate::mesh::Texture {
		let res = self.get_resolution();

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

		let texture = self.renderer.device.create_texture(&desc);

		let view = texture.create_view(&Default::default());
		let sampler = self
			.renderer
			.device
			.create_sampler(&wgpu::SamplerDescriptor {
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

		crate::mesh::Texture {
			texture,
			view,
			sampler,
		}
	}

	pub fn position(&self) -> glm::Vec3 {
		self.renderer.cameras.get(&self.id).unwrap().position
	}

	pub fn set_position(&self, position: glm::Vec3) {
		let mut inner = self.renderer.cameras.get_mut(&self.id).unwrap();
		inner.position = position;
		inner.dirty = true;
	}

	pub fn rotation(&self) -> glm::Quat {
		self.renderer.cameras.get(&self.id).unwrap().rotation
	}

	pub fn set_rotation(&self, rotation: glm::Quat) {
		let mut inner = self.renderer.cameras.get_mut(&self.id).unwrap();
		inner.rotation = rotation;
		inner.dirty = true;
	}

	pub fn resolution(&self) -> Resolution {
		self.renderer.cameras.get(&self.id).unwrap().resolution
	}

	pub fn set_resolution(&self, resolution: Resolution) {
		let mut inner = self.renderer.cameras.get_mut(&self.id).unwrap();
		inner.resolution = resolution;
		inner.dirty = true;
	}

	pub fn fovy(&self) -> f32 {
		self.renderer.cameras.get(&self.id).unwrap().fovy
	}

	pub fn set_fovy(&self, fovy: f32) {
		let mut inner = self.renderer.cameras.get_mut(&self.id).unwrap();
		inner.fovy = fovy;
		inner.dirty = true;
	}

	pub fn znear(&self) -> f32 {
		self.renderer.cameras.get(&self.id).unwrap().znear
	}

	pub fn set_znear(&self, znear: f32) {
		let mut inner = self.renderer.cameras.get_mut(&self.id).unwrap();
		inner.znear = znear;
		inner.dirty = true;
	}

	pub fn zfar(&self) -> f32 {
		self.renderer.cameras.get(&self.id).unwrap().zfar
	}

	pub fn set_zfar(&self, zfar: f32) {
		let mut inner = self.renderer.cameras.get_mut(&self.id).unwrap();
		inner.zfar = zfar;
		inner.dirty = true;
	}
}

pub struct CameraDescriptor {
	pub position: glm::Vec3,
	pub rotation: glm::Quat,
	pub resolution: Resolution,
	pub fovy: f32,
	pub znear: f32,
	pub zfar: f32,
}
pub(crate) struct InternalCamera {
	pub position: glm::Vec3,
	pub rotation: glm::Quat,
	pub resolution: Resolution,
	pub fovy: f32,
	pub znear: f32,
	pub zfar: f32,

	pub dirty: bool,

	pub buffer: Buffer<types::mat4x4f>,
	pub bind_group: CameraBindGroupNT,
}

impl InternalCamera {
	pub fn new(renderer: &crate::InternalRenderer, descriptor: &CameraDescriptor) -> Self {
		let buffer = Buffer::<types::mat4x4f>::new(
			&renderer.device,
			wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
			false,
		);

		let camera_bind_group = CameraBindGroup { camera: &buffer };

		let bind_group = camera_bind_group.to_bind_group(&renderer.device, None);

		drop(camera_bind_group);

		Self {
			position: descriptor.position,
			rotation: descriptor.rotation,
			resolution: descriptor.resolution,
			fovy: descriptor.fovy,
			znear: descriptor.znear,
			zfar: descriptor.zfar,

			dirty: true,

			buffer,
			bind_group,
		}
	}

	pub fn update(&mut self, renderer: &crate::InternalRenderer) {
		if !self.dirty {
			return;
		}

		self.dirty = false;

		let mut view = glm::translate(&glm::Mat4x4::identity(), &self.position);
		view = glm::quat_to_mat4(&self.rotation) * view;

		let aspect_ratio = match self.resolution {
			Resolution::Custom(res) => res[0] as f32 / res[1] as f32,
			Resolution::UseGlobalResolution => {
				let global_resolution = renderer.get_resolution();
				global_resolution[0] as f32 / global_resolution[1] as f32
			}
		};

		let proj = glm::perspective(
			aspect_ratio,
			self.fovy * (std::f32::consts::PI / 180.0),
			self.znear,
			self.zfar,
		);

		let matrix: [[f32; 4]; 4] = (proj * view).into();

		self.buffer.write_to(&renderer.queue, &matrix.into());
	}
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glm::Mat4x4 = glm::Mat4x4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
