use std::{collections::HashMap, sync::Mutex};

use dashmap::DashMap;

pub mod forward;
pub mod voxelization;

pub struct CommandEncoder<'renderer, 'camera> {
	renderer: &'renderer crate::Renderer,
	camera: Option<&'camera crate::camera::Camera>,
	inner_camera: Option<dashmap::mapref::one::Ref<'renderer, u64, crate::camera::InternalCamera>>,
	encoders: Vec<wgpu::CommandBuffer>,
	resources: DashMap<core::any::TypeId, Box<dyn crate::Resource>>,
	surface: Mutex<Option<wgpu::SurfaceTexture>>,
}

impl<'renderer, 'camera> CommandEncoder<'renderer, 'camera> {
	pub(crate) fn new(
		renderer: &'renderer crate::Renderer,
		camera: Option<&'camera crate::camera::Camera>,
	) -> Self {
		let inner_camera = match camera {
			Some(cam) => Some(renderer.renderer.cameras.get(&cam.id).unwrap()),
			None => None,
		};

		Self {
			renderer,
			camera,
			inner_camera,
			encoders: Vec::new(),
			resources: DashMap::new(),
			surface: Mutex::new(None),
		}
	}

	pub fn get_camera(&self) -> &Option<&crate::camera::Camera> {
		&self.camera
	}

	pub fn get_camera_bind_group(&self) -> Option<&crate::camera::CameraBindGroupNT> {
		if self.inner_camera.is_some() {
			Some(&self.inner_camera.as_ref().unwrap().bind_group)
		} else {
			None
		}
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.renderer.renderer.device
	}

	pub fn queue(&self) -> &wgpu::Queue {
		&self.renderer.renderer.queue
	}

	pub fn get_meshes(&self) -> MeshData<'renderer> {
		MeshData {
			meshes: self.renderer.renderer.meshes.iter().collect(),
		}
	}

	pub fn get_materials(&self) -> MaterialData<'renderer> {
		let mut materials = HashMap::new();

		for material in self.renderer.renderer.materials.iter() {
			materials.insert(*material.key(), material);
		}

		MaterialData { materials }
	}

	pub(crate) fn get_surface_texture_view(&self) -> wgpu::TextureView {
		// TODO: Replace with new camera/global resolution specific function

		let mut lock = self.surface.lock().unwrap();
		
		if lock.is_none() {
			*lock = Some(self.renderer.renderer.surface.get_current_texture().unwrap());

			lock.as_ref().unwrap().texture.create_view(&wgpu::TextureViewDescriptor::default())
		} else {
			lock.as_ref().unwrap().texture.create_view(&wgpu::TextureViewDescriptor::default())
		}
	}

	pub fn begin_pass<T: RenderPassTrait>(&mut self, pass: &mut T) {
		let mut global_resource_manager = self.renderer.get_resource_manager();
		let bundle = pass.execute(&self, &mut global_resource_manager);
		if bundle.is_some() {
			self.encoders.push(bundle.unwrap());
		}
	}

	pub fn finish(self) {
		self.renderer
			.renderer
			.queue
			.submit(self.encoders.into_iter());

		let mut lock = self.surface.lock().unwrap();

		if lock.is_some() {
			lock.take().unwrap().present();
		}
	}
}

pub trait RenderPassTrait {
	// const Dependencies: IntoIterator<>;
	// type Dependencies;

	fn execute<'manager>(
		&mut self,
		command_encoder: &'manager CommandEncoder,
		global_resources: &mut crate::ResourceManagerHandle<'manager>,
	) -> Option<wgpu::CommandBuffer>;
}

pub struct MeshData<'renderer> {
	pub meshes: Vec<dashmap::mapref::multiple::RefMulti<'renderer, crate::Id, crate::mesh::Mesh>>,
}

pub struct MaterialData<'renderer> {
	pub materials: HashMap<
		crate::Id,
		dashmap::mapref::multiple::RefMulti<'renderer, crate::Id, crate::mesh::Material>,
	>,
}
