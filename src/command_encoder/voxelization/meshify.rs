use crate::command_encoder::*;
use wgpu::util::DeviceExt;
use wgpu_helper::bind_group::{BindGroup, BindGroupType};

use crate::Renderer;

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

pub struct VoxelsMeshResource {
	pub vertices: wgpu::Buffer,
	pub indices: wgpu::Buffer,
}

impl crate::Resource for VoxelsMeshResource {
	fn updated_settings(&mut self, _renderer: &Renderer) {}
}

pub struct MeshifyPass {
	empty_count_buffer: wgpu::Buffer,
	count_buffer: wgpu::Buffer,
	count_staging_buffer: wgpu::Buffer,
	count_bind_group_layout: wgpu::BindGroupLayout,
	count_pipeline: wgpu::ComputePipeline,
	instance_bind_group_layout: wgpu::BindGroupLayout,
	instance_pipeline: wgpu::ComputePipeline,
}

impl MeshifyPass {
	pub fn new(renderer: &Renderer) -> Self {
		let count_shader = renderer
			.device()
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Count Pass Shader"),
				source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
					"../shaders/count_voxels.wgsl"
				))),
			});

		let empty_count_buffer =
			renderer
				.device()
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("Count Clear Buffer"),
					contents: bytemuck::cast_slice(&[0_i32]),
					usage: wgpu::BufferUsages::COPY_SRC,
				});

		let count_buffer =
			renderer
				.device()
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("Counter buffer"),
					contents: bytemuck::cast_slice(&[0_i32]),
					usage: wgpu::BufferUsages::COPY_DST
						| wgpu::BufferUsages::COPY_SRC
						| wgpu::BufferUsages::STORAGE,
				});

		let count_staging_buffer = renderer.device().create_buffer(&wgpu::BufferDescriptor {
			label: Some("Counter Staging Buffer"),
			size: std::mem::size_of::<i32>() as u64,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
			mapped_at_creation: false,
		});

		let count_bind_group_layout =
			renderer
				.device()
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
			renderer
				.device()
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&count_bind_group_layout],
					push_constant_ranges: &[],
				});

		let count_pipeline =
			renderer
				.device()
				.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					label: Some("Voxel Count Pass"),
					layout: Some(&count_pipeline_layout),
					module: &count_shader,
					entry_point: "get_voxel_sum",
				});

		let instance_shader =
			renderer
				.device()
				.create_shader_module(wgpu::ShaderModuleDescriptor {
					label: Some("Voxel Instance Shader"),
					source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
						"../shaders/voxel_instancing.wgsl"
					))),
				});

		let instance_bind_group_layout =
			renderer
				.device()
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
			renderer
				.device()
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&instance_bind_group_layout],
					push_constant_ranges: &[],
				});

		let instance_pipeline =
			renderer
				.device()
				.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					label: Some("Voxel Instancing Pass"),
					layout: Some(&instance_pipeline_layout),
					module: &instance_shader,
					entry_point: "main",
				});

		Self {
			empty_count_buffer,
			count_buffer,
			count_staging_buffer,
			count_bind_group_layout,
			count_pipeline,

			instance_bind_group_layout,
			instance_pipeline,
		}
	}
}

impl RenderPassTrait for MeshifyPass {
	fn execute<'manager>(
		&mut self,
		command_encoder: &'manager CommandEncoder,
		global_resources: &mut crate::ResourceManagerHandle<'manager>,
	) -> Option<wgpu::CommandBuffer> {
		let mut encoder =
			command_encoder
				.device()
				.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label: Some("Count Encoder"),
				});

		let voxels_resource = global_resources
			.get_resource::<super::VoxelsResource>()
			.unwrap();

		// Clear count buffer.
		encoder.copy_buffer_to_buffer(
			&self.empty_count_buffer,
			0,
			&self.count_buffer,
			0,
			std::mem::size_of::<i32>() as u64,
		);

		let view = voxels_resource
			.color
			.texture
			.create_view(&wgpu::TextureViewDescriptor {
				label: Some("Voxel Mipmapping output"),
				base_mip_level: 1,
				mip_level_count: Some(1),
				..Default::default()
			});

		let count_bind_group =
			command_encoder
				.device()
				.create_bind_group(&wgpu::BindGroupDescriptor {
					label: None,
					layout: &self.count_bind_group_layout,
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&view),
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
				voxels_resource.color.texture.size().width,
				voxels_resource.color.texture.size().height,
				voxels_resource.color.texture.size().depth_or_array_layers,
			);
		}

		encoder.copy_buffer_to_buffer(
			&self.count_buffer,
			0,
			&self.count_staging_buffer,
			0,
			std::mem::size_of::<i32>() as u64,
		);

		command_encoder.queue().submit(Some(encoder.finish()));

		let (tx, rx) = std::sync::mpsc::channel::<Result<(), wgpu::BufferAsyncError>>();

		self.count_staging_buffer
			.slice(..)
			.map_async(wgpu::MapMode::Read, move |res| {
				tx.send(res).unwrap();
			});

		let count;
		'main: loop {
			command_encoder.device().poll(wgpu::Maintain::Wait);

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

		let mut encoder =
			command_encoder
				.device()
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
			return None;
		}

		let vertex_buffer = command_encoder
			.device()
			.create_buffer(&wgpu::BufferDescriptor {
				label: Some("Voxel Vertex Buffer"),
				size: count as u64 * (std::mem::size_of::<Vertex>() * 8) as u64,
				usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
				mapped_at_creation: false,
			});

		let index_buffer = command_encoder
			.device()
			.create_buffer(&wgpu::BufferDescriptor {
				label: Some("Voxel Index Buffer"),
				size: count as u64 * (4 * 36) as u64,
				usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
				mapped_at_creation: false,
			});

		{
			let bind_group =
				command_encoder
					.device()
					.create_bind_group(&wgpu::BindGroupDescriptor {
						label: Some("Voxel Instancing Bindgroup"),
						layout: &self.instance_bind_group_layout,
						entries: &[
							wgpu::BindGroupEntry {
								binding: 0,
								resource: wgpu::BindingResource::TextureView(&view),
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
				voxels_resource.color.texture.size().width,
				voxels_resource.color.texture.size().height,
				voxels_resource.color.texture.size().depth_or_array_layers,
			);
		}

		command_encoder
			.queue()
			.submit(std::iter::once(encoder.finish()));

		let voxel_mesh_resource = VoxelsMeshResource {
			vertices: vertex_buffer,
			indices: index_buffer,
		};

		drop(voxels_resource);

		global_resources.insert_resource(voxel_mesh_resource);

		None
	}
}

pub struct RenderMeshifyPass {
	render_pipeline: wgpu::RenderPipeline,
}

impl RenderMeshifyPass {
	pub fn new(renderer: &crate::Renderer) -> Self {
		let render_shader = renderer
			.device()
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Shader"),
				source: wgpu::ShaderSource::Wgsl(
					include_str!("../shaders/voxel_render.wgsl").into(),
				),
			});

		let render_pipeline_layout =
			renderer
				.device()
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("Render Pipeline layout"),
					bind_group_layouts: &[&crate::camera::CameraBindGroup::get_bind_group_layout(
						&renderer.device(),
					)],
					push_constant_ranges: &[],
				});

		let render_pipeline =
			renderer
				.device()
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
							format: renderer.renderer.config.lock().unwrap().format,
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
						format: crate::DEPTH_FORMAT,
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

		Self { render_pipeline }
	}
}

impl RenderPassTrait for RenderMeshifyPass {
	fn execute<'manager>(
		&mut self,
		command_encoder: &'manager CommandEncoder,
		global_resources: &mut crate::ResourceManagerHandle<'manager>,
	) -> Option<wgpu::CommandBuffer> {
		if global_resources
			.get_resource::<VoxelsMeshResource>()
			.is_none()
		{
			println!("No Voxels Mesh... Returning");
			return None;
		}

		let view = command_encoder.get_surface_texture_view();

		let camera = command_encoder.get_camera().unwrap();

		let depth_buffer = if global_resources
			.get_resource::<crate::resources::DepthBufferResource>()
			.is_none()
		{
			let depth_buffer = camera.new_depth_buffer();

			let depth_buffer_resource = crate::resources::DepthBufferResource { depth_buffer };

			global_resources.insert_resource(depth_buffer_resource);
			global_resources
				.get_resource::<crate::resources::DepthBufferResource>()
				.unwrap()
		} else {
			global_resources
				.get_resource::<crate::resources::DepthBufferResource>()
				.unwrap()
		};

		let mut encoder = command_encoder
			.device()
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		let voxels_mesh = global_resources
			.get_resource::<VoxelsMeshResource>()
			.unwrap();
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
					view: &depth_buffer.depth_buffer.view,
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
			render_pass.set_bind_group(
				0,
				unsafe {
					command_encoder
						.get_camera_bind_group()
						.unwrap()
						.as_untyped()
				},
				&[],
			);

			render_pass.set_vertex_buffer(0, voxels_mesh.vertices.slice(..));
			render_pass.set_index_buffer(voxels_mesh.indices.slice(..), wgpu::IndexFormat::Uint32);

			render_pass.draw_indexed(0..(voxels_mesh.indices.size() as u32) / 4, 0, 0..1);
		}

		Some(encoder.finish())
	}
}
