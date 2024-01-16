pub struct GraphicsContext {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
}

impl GraphicsContext {
	pub async fn new(window: &winit::window::Window) -> Self {
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			..Default::default()
		});
		let surface = unsafe { instance.create_surface(window).unwrap() };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.unwrap();

		let mut limits = wgpu::Limits::default();
		limits.max_buffer_size = 268_435_456 * 4;
		limits.max_storage_buffer_binding_size = 134_217_728 * 8;
		limits.max_uniform_buffer_binding_size = 134_217_728 * 8;

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					features: wgpu::Features::empty(),
					// WebGL doesn't support all of wgpu's features, so if
					// we're building for the web we'll have to disable some.
					limits,
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
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		Self {
			surface,
			device,
			queue,
			config,
		}
	}
}
