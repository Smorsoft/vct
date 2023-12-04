use bind_group::*;
use wgpu::BufferAsyncError;

#[test]
fn new_buffer() {
	use crate::HostShareable;
	use wgpu::util::DeviceExt;

	let instance = wgpu::Instance::default();

	let adapter =
		pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
			.unwrap();

	let (device, queue) = pollster::block_on(adapter.request_device(
		&wgpu::DeviceDescriptor {
			label: None,
			features: wgpu::Features::empty(),
			limits: wgpu::Limits::default(),
		},
		None,
	))
	.unwrap();

	let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
			"test_shaders/shader.wgsl"
		))),
	});

	let size = std::mem::size_of::<crate::types::mat4x4f>() as wgpu::BufferAddress;
	let staging_buffer = crate::create_buffer::<crate::types::mat4x4f>(
		&device,
		wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
		false,
	);

	let orig_data: crate::types::mat4x4f = [[1.5_f32; 4]; 4].into();
	let storage_buffer = crate::create_buffer_init(
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
			resource: unsafe { storage_buffer.as_buffer() }.as_entire_binding(),
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

	encoder.copy_buffer_to_buffer(
		unsafe { storage_buffer.as_buffer() },
		0,
		unsafe { staging_buffer.as_buffer() },
		0,
		size,
	);

	queue.submit(Some(encoder.finish()));

	let (tx, rx) = std::sync::mpsc::channel::<Result<(), BufferAsyncError>>();

	staging_buffer.map_async(wgpu::MapMode::Read, move |res| {
		tx.send(res).unwrap();
	});

	'main: loop {
		device.poll(wgpu::MaintainBase::Poll);

		match rx.recv() {
			Ok(_) => {
				let data = staging_buffer.get_mapped_data();
				println!("{:?}", data);

				break 'main;
			}
			Err(e) => {
				panic!("{:?}", e);
			}
		}
	}
}
