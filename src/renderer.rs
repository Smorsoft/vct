use std::{collections::HashMap, pin::Pin};

use winit::window::Window;

pub mod context;
pub mod lights;
pub mod load_gltf;
pub mod mesh;
mod render_pass;
mod voxelization;

pub struct Renderer {
	pub context: Pin<Box<context::GraphicsContext>>,
	pub camera_buffer: wgpu::Buffer,
	pub meshes: Vec<mesh::Mesh>,
	pub materials: HashMap<uuid::Uuid, mesh::Material>,
	pub render_pipeline: wgpu::RenderPipeline,
	pub model_bind_group_layout: wgpu::BindGroupLayout,
	pub material_bind_group_layout: wgpu::BindGroupLayout,
	pub camera_bind_group: wgpu::BindGroup,
	pub depth_buffer: mesh::Texture,
	pub voxelization: voxelization::Voxelization,
	pub render_voxels: bool,
}

impl Renderer {
	pub async fn new(window: &Window) -> Self {
		use wgpu::util::DeviceExt;
		let context = context::GraphicsContext::new(window).await;

		let camera_buffer = context
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Camera Buffer"),
				contents: bytemuck::cast_slice(&[[0.0_f32; 4]; 4]),
				usage: wgpu::BufferUsages::UNIFORM
					| wgpu::BufferUsages::COPY_DST
					| wgpu::BufferUsages::COPY_SRC,
			});

		let shader = context
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Shader"),
				source: wgpu::ShaderSource::Wgsl(
					include_str!("renderer/shaders/shader.wgsl").into(),
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

		let camera_bind_group = context
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("Camera Bind Group"),
				layout: &camera_bind_group_layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: camera_buffer.as_entire_binding(),
				}],
			});

		let model_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("Model Bind group layout"),
					entries: &[wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}],
				});

		let material_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("Material Bind group layout"),
					entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0,
							visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
								view_dimension: wgpu::TextureViewDimension::D2,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 1,
							visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 2,
							visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
								view_dimension: wgpu::TextureViewDimension::D2,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 3,
							visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 4,
							visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
								view_dimension: wgpu::TextureViewDimension::D2,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 5,
							visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
							count: None,
						},
					],
				});

		let render_pipeline_layout =
			context
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("Render Pipeline layout"),
					bind_group_layouts: &[
						&camera_bind_group_layout,
						&model_bind_group_layout,
						&material_bind_group_layout,
					],
					push_constant_ranges: &[],
				});

		let render_pipeline =
			context
				.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: Some("Render Pipeline"),
					layout: Some(&render_pipeline_layout),
					vertex: wgpu::VertexState {
						module: &shader,
						entry_point: "vs_main",
						buffers: &[
							mesh::VertexPosition::desc(),
							mesh::VertexNormals::desc(),
							mesh::VertexColors::desc(),
						],
					},
					fragment: Some(wgpu::FragmentState {
						module: &shader,
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
						format: mesh::Texture::DEPTH_FORMAT,
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

		let depth_buffer = mesh::Texture::create_depth_texture(&context.device, &context.config);

		let pinned_context = Box::pin(context);

		let voxelization = voxelization::Voxelization::new(
			&pinned_context,
			&model_bind_group_layout,
			&material_bind_group_layout,
		);

		Self {
			context: pinned_context,
			camera_buffer,
			meshes: Vec::new(),
			materials: HashMap::new(),
			render_pipeline,
			model_bind_group_layout,
			material_bind_group_layout,
			camera_bind_group,
			depth_buffer,
			voxelization,
			render_voxels: false,
		}
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.context.config.width = width;
		self.context.config.height = height;
		self.context
			.surface
			.configure(&self.context.device, &self.context.config);

		self.depth_buffer =
			mesh::Texture::create_depth_texture(&self.context.device, &self.context.config)
	}

	pub fn update_camera(&mut self, camera: &super::camera::Camera) {
		camera.update(&mut self.context.queue, &mut self.camera_buffer)
	}

	pub fn voxelize(&mut self) {
		self.voxelization
			.voxelize(&self.context, &self.meshes, &self.materials);
		self.voxelization.meshify(&self.context);
	}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		if self.render_voxels {
			self.voxelization
				.render(&self.context, &self.depth_buffer, &self.camera_bind_group);
		} else {
			let output = self.context.surface.get_current_texture()?;

			let view = output.texture.create_view(&Default::default());

			let mut encoder =
				self.context
					.device
					.create_command_encoder(&wgpu::CommandEncoderDescriptor {
						label: Some("Render Encoder"),
					});

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
						view: &self.depth_buffer.view,
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
				render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

				for mesh in self.meshes.iter() {
					render_pass.set_bind_group(1, &mesh.model_bind_group, &[]);
					render_pass
						.set_vertex_buffer(0, mesh.vertex_buffer.slice(mesh.positions.to_owned()));
					render_pass.set_vertex_buffer(1, mesh.vertex_buffer.slice(mesh.normals.to_owned()));
					render_pass.set_vertex_buffer(2, mesh.vertex_buffer.slice(mesh.colors.to_owned()));
					render_pass
						.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
					for primitive in mesh.primitives.iter() {
						render_pass.set_bind_group(
							2,
							&self.materials[&primitive.material].bind_group,
							&[],
						);
						render_pass.draw_indexed(primitive.index.to_owned(), 0, 0..1);
					}
				}
			}
			self.context.queue.submit(std::iter::once(encoder.finish()));
			output.present();
		}

		return Ok(());
	}
}
