extern crate nalgebra_glm as glm;
use wgpu::rwh::HasWindowHandle;
use wgpu_helper::{bind_group::BindGroup, *};

use dashmap::DashMap;

use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	rc::Rc,
	sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use winit::window::Window;

pub mod camera;
pub mod command_encoder;
pub mod lights;
pub mod load_gltf;
pub mod mesh;
pub mod resources;
mod scene;
pub mod transform;
// pub mod model;
// pub mod texture;

pub type Id = u64;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const DIFFUSE_BUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub const MATERIAL_BIND_GROUP_LAYOUT: &'static wgpu::BindGroupLayoutDescriptor =
	&wgpu::BindGroupLayoutDescriptor {
		label: Some("Material Bind group layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Texture {
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
					view_dimension: wgpu::TextureViewDimension::D2,
					multisampled: false,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 2,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Texture {
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
					view_dimension: wgpu::TextureViewDimension::D2,
					multisampled: false,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 3,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 4,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Texture {
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
					view_dimension: wgpu::TextureViewDimension::D2,
					multisampled: false,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 5,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
	};

#[derive(BindGroup)]
#[layout(MATERIAL_BIND_GROUP_LAYOUT)]
pub struct MaterialBindGroup<'a> {
	pub t_diffuse: &'a wgpu::TextureView,
	pub s_diffuse: &'a wgpu::Sampler,
	pub t_metal: &'a wgpu::TextureView,
	pub s_metal: &'a wgpu::Sampler,
	pub t_normal: &'a wgpu::TextureView,
	pub s_normal: &'a wgpu::Sampler,
}

pub const MODEL_BIND_GROUP_LAYOUT: &'static wgpu::BindGroupLayoutDescriptor =
	&wgpu::BindGroupLayoutDescriptor {
		label: Some("Model Bind group layout"),
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::VERTEX_FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::union(
					wgpu::ShaderStages::VERTEX_FRAGMENT,
					wgpu::ShaderStages::COMPUTE,
				),
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		],
	};

#[derive(BindGroup)]
#[layout(MODEL_BIND_GROUP_LAYOUT)]
pub struct ModelBindGroup<'a> {
	pub transform: &'a wgpu_helper::Buffer<types::mat4x4f>,
	pub normal_transform: &'a wgpu_helper::Buffer<types::mat3x3f>,
}

#[derive(Clone)]
pub struct Renderer {
	renderer: Rc<InternalRenderer>,
}

impl Renderer {
	pub async fn new(window: Arc<Window>, settings: RendererSettings) -> Self {
		let renderer = InternalRenderer::new(window, settings).await;

		Self {
			renderer: Rc::new(renderer),
		}
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.renderer.device
	}

	pub fn resize(&self, width: u32, height: u32) {
		self.renderer.resize(width, height);
	}

	pub fn update(&self) {
		for mut camera in self.renderer.cameras.iter_mut() {
			camera.update(&self.renderer);
		}

		let mut dirty = self.renderer.dirty_settings.lock().unwrap();

		if *dirty {
			let mut lock = self.renderer.resource_manager.map.write().unwrap();

			for resource in lock.iter_mut() {
				resource.1.updated_settings(&self)
			}

			*dirty = false;
		}
	}

	pub fn new_command_encoder<'renderer, 'camera: 'renderer>(
		&'renderer self,
		camera: Option<&'camera camera::Camera>,
	) -> command_encoder::CommandEncoder {
		command_encoder::CommandEncoder::<'renderer, 'camera>::new(&self, camera)
	}

	pub fn new_camera(&mut self, descriptor: &camera::CameraDescriptor) -> camera::Camera {
		let id = self.renderer.new_id();

		self.renderer
			.cameras
			.insert(id, camera::InternalCamera::new(&self.renderer, descriptor));

		camera::Camera {
			renderer: self.get_handle(),
			id,
		}
	}

	pub(crate) fn get_resolution(&self) -> [u32; 2] {
		self.renderer.get_resolution()
	}

	pub(crate) fn get_resource_manager(&self) -> ResourceManagerHandle {
		self.renderer.resource_manager.get_handle()
	}

	pub fn insert_resource<T: crate::Resource>(&self, resource: T) -> Option<Box<T>> {
		self.renderer.resource_manager.insert_resource(resource)
	}

	pub(crate) fn get_handle(&self) -> Rc<InternalRenderer> {
		Rc::clone(&self.renderer)
	}

	pub fn load_gltf<P: AsRef<std::path::Path>>(
		&mut self,
		path: P,
		is_static: bool,
	) -> Vec<camera::Camera> {
		let cameras = load_gltf::load_gltf(&self.renderer, path, is_static);

		return cameras;
	}
}

pub(crate) struct InternalRenderer {
	pub surface: wgpu::Surface<'static>,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: Mutex<wgpu::SurfaceConfiguration>,
	pub settings: RendererSettings,
	dirty_settings: Mutex<bool>,

	// TODO: Better mesh, material and instance storage/references
	pub meshes: DashMap<Id, mesh::Mesh>,
	pub materials: DashMap<Id, mesh::Material>,
	pub cameras: DashMap<Id, camera::InternalCamera>,
	pub resource_manager: ResourceManager,
	current_id: core::sync::atomic::AtomicU64,
}

// TODO: Remove, currently used for load_gltf multithreading, which only touches current_id, materials and meshes as such will not crash, but still unsafe.
unsafe impl Send for InternalRenderer {}
unsafe impl Sync for InternalRenderer {}

impl InternalRenderer {
	pub async fn new(window: Arc<Window>, settings: RendererSettings) -> Self {
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			..Default::default()
		});
		let surface = instance.create_surface(window).unwrap();
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.unwrap();

		// TODO: Remove, as they are temporary to allow for voxel debug rendering
		let mut limits = wgpu::Limits::default();
		limits.max_buffer_size = 268_435_456 * 4;
		limits.max_storage_buffer_binding_size = 134_217_728 * 8;
		limits.max_uniform_buffer_binding_size = 134_217_728 * 8;

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::empty(),
					// WebGL doesn't support all of wgpu's features, so if
					// we're building for the web we'll have to disable some.
					required_limits: limits,
				},
				None,
			)
			.await
			.unwrap();

		// Config for surface
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_capabilities(&adapter).formats[0],
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			desired_maximum_frame_latency: 2,
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		Self {
			surface,
			device,
			queue,
			config: Mutex::new(config),
			settings,
			dirty_settings: Mutex::new(true),
			meshes: DashMap::new(),
			materials: DashMap::new(),
			cameras: DashMap::new(),
			resource_manager: ResourceManager::new(),
			current_id: core::sync::atomic::AtomicU64::new(0),
		}
	}

	pub fn resize(&self, width: u32, height: u32) {
		let mut conf = self.config.lock().unwrap();
		if width != conf.width || height != conf.height {
			conf.width = width;
			conf.height = height;
			self.surface.configure(&self.device, &conf);
			*self.dirty_settings.lock().unwrap() = true;
		}

	}

	pub fn get_resolution(&self) -> [u32; 2] {
		let lock = self.config.lock().unwrap();
		[lock.width, lock.height]
	}

	pub fn get_scaled_resolution(&self) -> [u32; 2] {
		todo!()
	}

	pub fn new_id(&self) -> Id {
		self.current_id
			.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
	}
}

pub struct RendererSettings {
	pub render_scale: f32,
	pub extras: HashMap<String, u8>,
}

pub trait Resource: core::any::Any + Send {
	fn updated_settings(&mut self, renderer: &Renderer);
}

pub struct ResourceManager {
	map: RwLock<HashMap<core::any::TypeId, Box<dyn Resource>>>,
}

impl ResourceManager {
	pub fn new() -> Self {
		Self {
			map: RwLock::new(HashMap::new()),
		}
	}

	pub fn get_handle<'manager>(&'manager self) -> ResourceManagerHandle<'manager> {
		ResourceManagerHandle { manager: self }
	}

	pub fn get_resource<T: Resource>(&self) -> Option<ResourceHandle<T>> {
		let read_guard = self.map.read().unwrap();
		if read_guard.contains_key(&core::any::TypeId::of::<T>()) {
			Some(ResourceHandle {
				read_guard,
				phantom: std::marker::PhantomData::default(),
			})
		} else {
			None
		}
	}

	pub fn get_mut_resource<T: Resource>(&self) -> Option<ResourceHandleMut<T>> {
		let lock = self.map.write().unwrap();
		if lock.contains_key(&core::any::TypeId::of::<T>()) {
			Some(ResourceHandleMut {
				lock,
				phantom: std::marker::PhantomData::default(),
			})
		} else {
			None
		}
	}

	pub fn insert_resource<T: Resource>(&self, resource: T) -> Option<Box<T>> {
		let mut lock = self.map.write().unwrap();
		let old = lock.insert(core::any::TypeId::of::<T>(), Box::new(resource));
		match old {
			Some(old) => Some(unsafe { Box::from_raw(Box::into_raw(old).cast::<T>()) }),
			None => None,
		}
	}

	pub fn remove_box<T: Resource>(&self) -> Option<Box<T>> {
		let mut lock = self.map.write().unwrap();
		let old = lock.remove(&core::any::TypeId::of::<T>());
		match old {
			Some(old) => Some(unsafe { Box::from_raw(Box::into_raw(old).cast::<T>()) }),
			None => None,
		}
	}

	pub fn remove<T: Resource + Copy>(&self) -> Option<T> {
		let mut lock = self.map.write().unwrap();
		let old = lock.remove(&core::any::TypeId::of::<T>());
		match old {
			Some(old) => Some(unsafe { *Box::into_raw(old).cast::<T>() }),
			None => None,
		}
	}
}

pub struct ResourceManagerHandle<'manager> {
	manager: &'manager ResourceManager,
}

impl<'manager> ResourceManagerHandle<'manager> {
	pub fn get_resource<T: Resource>(&self) -> Option<ResourceHandle<T>> {
		self.manager.get_resource()
	}

	pub fn get_mut_resource<T: Resource>(&mut self) -> Option<ResourceHandleMut<T>> {
		self.manager.get_mut_resource()
	}

	pub fn insert_resource<T: Resource>(&mut self, resource: T) -> Option<Box<T>> {
		self.manager.insert_resource(resource)
	}

	pub fn remove_box<T: Resource>(&mut self) -> Option<Box<T>> {
		self.manager.remove_box()
	}

	pub fn remove<T: Resource + Copy>(&mut self) -> Option<T> {
		self.manager.remove()
	}
}

pub struct ResourceHandle<'manager, T: Resource> {
	read_guard: RwLockReadGuard<'manager, HashMap<core::any::TypeId, Box<dyn Resource>>>,
	phantom: std::marker::PhantomData<T>,
}

impl<'manager, T: Resource> Deref for ResourceHandle<'manager, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let inner = self
			.read_guard
			.get(&core::any::TypeId::of::<T>())
			.unwrap()
			.as_ref();
		unsafe { &*(inner as *const dyn Resource as *const T) }
	}
}

pub struct ResourceHandleMut<'manager, T: Resource> {
	lock: RwLockWriteGuard<'manager, HashMap<core::any::TypeId, Box<dyn Resource>>>,
	phantom: std::marker::PhantomData<T>,
}

impl<'manager, T: Resource> Deref for ResourceHandleMut<'manager, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let inner = self
			.lock
			.get(&core::any::TypeId::of::<T>())
			.unwrap()
			.as_ref();
		unsafe { &*(inner as *const dyn Resource as *const T) }
	}
}

impl<'manager, T: Resource> DerefMut for ResourceHandleMut<'manager, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		let inner = self
			.lock
			.get_mut(&core::any::TypeId::of::<T>())
			.unwrap()
			.as_mut();
		unsafe { &mut *(inner as *mut dyn Resource as *mut T) }
	}
}
