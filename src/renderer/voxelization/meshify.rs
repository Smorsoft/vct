use std::pin::Pin;
use wgpu::util::DeviceExt;

use crate::renderer::context::GraphicsContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	position: [f32; 3],
	_pad1: u32,
	color: [f32; 3],
	_pad2: u32,
}

impl Vertex {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 4]>() as u64,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x3,
				},
			],
		}
	}
}

pub struct Meshify {
	empty_count_buffer: wgpu::Buffer,
	count_buffer: wgpu::Buffer,
	count_staging_buffer: wgpu::Buffer,
	count_bind_group_layout: wgpu::BindGroupLayout,
	count_pipeline: wgpu::ComputePipeline,
	instance_bind_group_layout: wgpu::BindGroupLayout,
	instance_pipeline: wgpu::ComputePipeline,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: Option<wgpu::Buffer>,
	index_buffer: Option<wgpu::Buffer>,
}

impl Meshify {
	pub fn new(context: &Pin<Box<GraphicsContext>>) -> Self {
		let count_shader = context
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Count Pass Shader"),
				source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
					"../shaders/count_voxels.wgsl"
				))),
			});

		let empty_count_buffer =
			context
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("Count Clear Buffer"),
					contents: bytemuck::cast_slice(&[0_i32]),
					usage: wgpu::BufferUsages::COPY_SRC,
				});

		let count_buffer = context
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Counter buffer"),
				contents: bytemuck::cast_slice(&[0_i32]),
				usage: wgpu::BufferUsages::COPY_DST
					| wgpu::BufferUsages::COPY_SRC
					| wgpu::BufferUsages::STORAGE,
			});

		let count_staging_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Counter Staging Buffer"),
			size: std::mem::size_of::<i32>() as u64,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
			mapped_at_creation: false,
		});

		let count_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: None,
					entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: false },
								view_dimension: wgpu::TextureViewDimension::D3,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 1,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: false },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
					],
				});

		let count_pipeline_layout =
			context
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&count_bind_group_layout],
					push_constant_ranges: &[],
				});

		let count_pipeline =
			context
				.device
				.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					label: Some("Voxel Count Pass"),
					layout: Some(&count_pipeline_layout),
					module: &count_shader,
					entry_point: "get_voxel_sum",
				});

		let instance_shader = context
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Voxel Instance Shader"),
				source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
					"../shaders/voxel_instancing.wgsl"
				))),
			});

		let instance_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: None,
					entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: false },
								view_dimension: wgpu::TextureViewDimension::D3,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 1,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: false },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 2,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: false },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 3,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: false },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
					],
				});

		let instance_pipeline_layout =
			context
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&instance_bind_group_layout],
					push_constant_ranges: &[],
				});

		let instance_pipeline =
			context
				.device
				.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					label: Some("Voxel Instancing Pass"),
					layout: Some(&instance_pipeline_layout),
					module: &instance_shader,
					entry_point: "main",
				});

		let render_shader = context
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Shader"),
				source: wgpu::ShaderSource::Wgsl(
					include_str!("../shaders/voxel_render.wgsl").into(),
				),
			});

		let camera_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("Camera Bind group layout"),
					entries: &[wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}],
				});

		let render_pipeline_layout =
			context
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("Render Pipeline layout"),
					bind_group_layouts: &[&camera_bind_group_layout],
					push_constant_ranges: &[],
				});

		let render_pipeline =
			context
				.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: Some("Voxel Render Pipeline"),
					layout: Some(&render_pipeline_layout),
					vertex: wgpu::VertexState {
						module: &render_shader,
						entry_point: "vs_main",
						buffers: &[Vertex::desc()],
					},
					fragment: Some(wgpu::FragmentState {
						module: &render_shader,
						entry_point: "fs_main",
						targets: &[Some(wgpu::ColorTargetState {
							format: context.config.format,
							blend: Some(wgpu::BlendState::REPLACE),
							write_mask: wgpu::ColorWrites::ALL,
						})],
					}),
					primitive: wgpu::PrimitiveState {
						topology: wgpu::PrimitiveTopology::TriangleList,
						strip_index_format: None,
						front_face: wgpu::FrontFace::Ccw,
						cull_mode: None,
						polygon_mode: wgpu::PolygonMode::Fill,
						unclipped_depth: false,
						conservative: false,
					},
					depth_stencil: Some(wgpu::DepthStencilState {
						format: super::super::mesh::Texture::DEPTH_FORMAT,
						depth_write_enabled: true,
						depth_compare: wgpu::CompareFunction::Less,
						stencil: wgpu::StencilState::default(),
						bias: wgpu::DepthBiasState::default(),
					}),
					multisample: wgpu::MultisampleState {
						count: 1,
						mask: !0,
						alpha_to_coverage_enabled: false,
					},
					multiview: None,
				});

		Self {
			empty_count_buffer,
			count_buffer,
			count_staging_buffer,
			count_bind_group_layout,
			count_pipeline,

			instance_bind_group_layout,
			instance_pipeline,
			render_pipeline,
			vertex_buffer: None,
			index_buffer: None,
		}
	}

	pub fn meshify(
		&mut self,
		context: &Pin<Box<GraphicsContext>>,
		voxels: &crate::renderer::mesh::Texture,
	) {
		let mut encoder = context
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Count Encoder"),
			});

		// Clear count buffer.
		encoder.copy_buffer_to_buffer(
			&self.empty_count_buffer,
			0,
			&self.count_buffer,
			0,
			std::mem::size_of::<i32>() as u64,
		);

		let count_bind_group = context
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout: &self.count_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&voxels.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: self.count_buffer.as_entire_binding(),
					},
				],
			});

		{
			let mut counter_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
				label: Some("Count pass"),
				timestamp_writes: None,
			});

			counter_pass.set_bind_group(0, &count_bind_group, &[]);
			counter_pass.set_pipeline(&self.count_pipeline);
			counter_pass.dispatch_workgroups(
				voxels.texture.size().width,
				voxels.texture.size().height,
				voxels.texture.size().depth_or_array_layers,
			);
		}

		encoder.copy_buffer_to_buffer(
			&self.count_buffer,
			0,
			&self.count_staging_buffer,
			0,
			std::mem::size_of::<i32>() as u64,
		);

		context.queue.submit(Some(encoder.finish()));

		let (tx, rx) = std::sync::mpsc::channel::<Result<(), wgpu::BufferAsyncError>>();

		self.count_staging_buffer
			.slice(..)
			.map_async(wgpu::MapMode::Read, move |res| {
				tx.send(res).unwrap();
			});

		let count;
		'main: loop {
			context.device.poll(wgpu::Maintain::Wait);

			match rx.try_recv() {
				Ok(_) => {
					let view = self.count_staging_buffer.slice(..).get_mapped_range();

					count = *unsafe { &*(view.as_ptr() as *const i32) };
					break 'main;
				}
				Err(_) => {}
			}
		}

		self.count_staging_buffer.unmap();

		let mut encoder = context
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Meshify Encoder"),
			});

		// Clear count buffer.
		encoder.copy_buffer_to_buffer(
			&self.empty_count_buffer,
			0,
			&self.count_buffer,
			0,
			std::mem::size_of::<i32>() as u64,
		);

		if count <= 0 {
			return;
		}

		let vertex_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Voxel Vertex Buffer"),
			size: count as u64 * (std::mem::size_of::<Vertex>() * 8) as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false,
		});

		let index_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Voxel Index Buffer"),
			size: count as u64 * (4 * 36) as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false,
		});

		{
			let bind_group = context
				.device
				.create_bind_group(&wgpu::BindGroupDescriptor {
					label: Some("Voxel Instancing Bindgroup"),
					layout: &self.instance_bind_group_layout,
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&voxels.view),
						},
						wgpu::BindGroupEntry {
							binding: 1,
							resource: vertex_buffer.as_entire_binding(),
						},
						wgpu::BindGroupEntry {
							binding: 2,
							resource: index_buffer.as_entire_binding(),
						},
						wgpu::BindGroupEntry {
							binding: 3,
							resource: self.count_buffer.as_entire_binding(),
						},
					],
				});

			let mut instance_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
				label: Some("Voxel Instancing pass"),
				timestamp_writes: None,
			});

			instance_pass.set_bind_group(0, &bind_group, &[]);
			instance_pass.set_pipeline(&self.instance_pipeline);
			instance_pass.dispatch_workgroups(
				voxels.texture.size().width,
				voxels.texture.size().height,
				voxels.texture.size().depth_or_array_layers,
			);
		}

		context.queue.submit(std::iter::once(encoder.finish()));

		self.vertex_buffer = Some(vertex_buffer);
		self.index_buffer = Some(index_buffer);
	}

	pub fn render(
		&mut self,
		context: &Pin<Box<GraphicsContext>>,
		depth_buffer: &crate::renderer::mesh::Texture,
		camera_bind_group: &wgpu::BindGroup,
	) {
		if self.vertex_buffer.is_none() {
			return;
		}

		let mut encoder = context
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});

		let output = context.surface.get_current_texture().unwrap();

		let view = output.texture.create_view(&Default::default());

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.0,
							g: 0.0,
							b: 0.0,
							a: 1.0,
						}),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &depth_buffer.view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: wgpu::StoreOp::Store,
					}),
					stencil_ops: None,
				}),
				occlusion_query_set: None,
				timestamp_writes: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, camera_bind_group, &[]);

			render_pass.set_vertex_buffer(
				0,
				self.vertex_buffer
					.as_ref()
					.expect("meshify must be called before render")
					.slice(..),
			);
			render_pass.set_index_buffer(
				self.index_buffer
					.as_ref()
					.expect("meshify must be called before render")
					.slice(..),
				wgpu::IndexFormat::Uint32,
			);

			render_pass.draw_indexed(
				0..(self
					.index_buffer
					.as_ref()
					.expect("meshify must be called before render")
					.size() as u32) / 4,
				0,
				0..1,
			);
		}

		context.queue.submit(std::iter::once(encoder.finish()));
		output.present();
	}
}
