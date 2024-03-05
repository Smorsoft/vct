use ::wgpu_helper::*;
use wgpu::BufferAsyncError;

#[test]
fn new_buffer() {
	let instance = wgpu::Instance::default();

	let adapter =
		pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
			.unwrap();

	let (device, queue) = pollster::block_on(adapter.request_device(
		&wgpu::DeviceDescriptor {
			label: None,
			required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::default(),
		},
		None,
	))
	.unwrap();

	let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
			"buffer.wgsl"
		))),
	});

	let staging_buffer = wgpu_helper::Buffer::<crate::types::mat4x4f>::new(
		&device,
		wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
		false,
	);

	let orig_data: crate::types::mat4x4f = [[1.5_f32; 4]; 4].into();
	let storage_buffer = wgpu_helper::Buffer::new_init(
		&device,
		&orig_data,
		wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
	);

	let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
		label: None,
		layout: None,
		module: &cs_module,
		entry_point: "main",
	});

	let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: None,
		layout: &bind_group_layout,
		entries: &[wgpu::BindGroupEntry {
			binding: 0,
			resource: storage_buffer.get_binding(),
		}],
	});

	let mut encoder =
		device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
	{
		let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: None,
			timestamp_writes: None,
		});
		cpass.set_pipeline(&compute_pipeline);
		cpass.set_bind_group(0, &bind_group, &[]);
		cpass.insert_debug_marker("compute collatz iterations");
		cpass.dispatch_workgroups(1, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
	} 

	storage_buffer.copy_to_buffer(&mut encoder, &staging_buffer);

	queue.submit(Some(encoder.finish()));

	assert_eq!(staging_buffer.map_sync(&device), &[[3.0_f32; 4]; 4].into());
}
