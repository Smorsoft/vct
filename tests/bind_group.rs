use ::wgpu_helper::*;
use wgpu_helper::bind_group::*;

#[repr(transparent)]
pub struct TestBindGroup(::wgpu::BindGroup);

impl BindGroup for TestBindGroup {
	type Source = TestBindGroupType<'static>;
}

pub struct TestBindGroupType<'a> {
	pub indices: &'a crate::Buffer<crate::types::mat4x4f>,
}

static TEST_BIND_GROUP_LAYOUT: std::sync::OnceLock<wgpu::BindGroupLayout> =
	std::sync::OnceLock::new();

impl<'a> BindGroupType for TestBindGroupType<'a> {
	type BindGroup = TestBindGroup;

	const BIND_GROUP_LAYOUT_DESCRIPTOR: &'static wgpu::BindGroupLayoutDescriptor<'static> =
		&wgpu::BindGroupLayoutDescriptor {
			label: Some("CRinge"),
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

	fn get_bind_group_lock() -> &'static std::sync::OnceLock<wgpu::BindGroupLayout> {
		&TEST_BIND_GROUP_LAYOUT
	}

	fn to_bind_group(&self, device: &wgpu::Device, label: Option<&str>) -> Self::BindGroup {
		unsafe {
			Self::BindGroup::from_untyped(device.create_bind_group(&wgpu::BindGroupDescriptor {
				label,
				layout: Self::get_bind_group_layout(device),
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: self.indices.get_binding(),
				}],
			}))
		}
	}
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
			"bind_group.wgsl"
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

	let bind_group = TestBindGroupType {
		indices: &storage_buffer,
	};

	let official_bind_group = bind_group.to_bind_group(&device, None);

	let bind_group_layout = TestBindGroupType::get_bind_group_layout(&device);

	let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Test Pipeline"),
		bind_group_layouts: &[bind_group_layout],
		push_constant_ranges: &[],
	});

	let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
		label: None,
		layout: Some(&compute_pipeline_layout),
		module: &cs_module,
		entry_point: "main",
	});

	let mut encoder =
		device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
	{
		let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: None,
			timestamp_writes: None,
		});
		cpass.set_pipeline(&compute_pipeline);
		cpass.set_bind_group(0, unsafe { official_bind_group.as_untyped() }, &[]);
		cpass.insert_debug_marker("compute collatz iterations");
		cpass.dispatch_workgroups(1, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
	}

	storage_buffer.copy_to_buffer(&mut encoder, &staging_buffer);

	queue.submit(Some(encoder.finish()));

	assert_eq!(staging_buffer.map_sync(&device), &[[3.0_f32; 4]; 4].into());
}
