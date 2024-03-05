use ::wgpu_helper::*;
use wgpu_helper::bind_group::*;

#[repr(C)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct TestHostShareable {
	first: u32,
	padding: [u32; 3],
	second: types::mat4x4f,
}

impl crate::HostShareable for TestHostShareable {}

const TEST_BIND_GROUP_LAYOUT: &'static wgpu::BindGroupLayoutDescriptor<'static> =
	&wgpu::BindGroupLayoutDescriptor {
		label: Some("new Host Shareable test"),
		entries: &[wgpu::BindGroupLayoutEntry {
			binding: 0,
			visibility: wgpu::ShaderStages::COMPUTE,
			ty: wgpu::BindingType::Buffer {
				ty: wgpu::BufferBindingType::Storage { read_only: false },
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		}],
	};

#[derive(BindGroup)]
#[layout(TEST_BIND_GROUP_LAYOUT)]
pub struct TestBindGroup<'a> {
	pub data: &'a crate::Buffer<TestHostShareable>,
}


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
			"new_hostshareable.wgsl"
		))),
	});

	let staging_buffer = wgpu_helper::Buffer::<TestHostShareable>::new(
		&device,
		wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
		false,
	);

	let orig_data = TestHostShareable {
		first: 1,
		padding: [0; 3],
		second: [[1.5_f32; 4]; 4].into(),
	};

	let storage_buffer = wgpu_helper::Buffer::new_init(
		&device,
		&orig_data,
		wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
	);

	let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Test Layout"),
		bind_group_layouts: &[
			TestBindGroup::get_bind_group_layout(&device),
		],
		push_constant_ranges: &[]
	});

	let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
		label: None,
		layout: Some(&compute_pipeline_layout),
		module: &cs_module,
		entry_point: "main",
	});
	
	let bind_group = TestBindGroup {
		data: &storage_buffer,
	};

	let bind_group_bind_group = bind_group.to_bind_group(&device, None);

	let mut encoder =
		device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
	{
		let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: None,
			timestamp_writes: None,
		});

		cpass.set_pipeline(&compute_pipeline);
		cpass.set_bind_group(0, unsafe { bind_group_bind_group.as_untyped() }, &[]);
		cpass.insert_debug_marker("compute collatz iterations");
		cpass.dispatch_workgroups(1, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
	} 

	storage_buffer.copy_to_buffer(&mut encoder, &staging_buffer);

	queue.submit(Some(encoder.finish()));

	let assert_data = TestHostShareable {
		first: orig_data.first * 2,
		padding: [0; 3],
		second: [[4.5_f32; 4]; 4].into()
	};

	assert_eq!(staging_buffer.map_sync(&device), &assert_data);
}
