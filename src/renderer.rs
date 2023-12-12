use std::{collections::HashMap, ops::ControlFlow, pin::Pin};

use winit::window::Window;

mod context;
pub mod lights;
pub mod mesh;
mod render_pass;
mod voxelization;

pub struct Renderer {
	pub context: Pin<Box<context::GraphicsContext>>,
	pub camera_buffer: wgpu::Buffer,
	pub meshes: Vec<mesh::Mesh>,
	pub render_pipeline: wgpu::RenderPipeline,
	pub model_bind_group_layout: wgpu::BindGroupLayout,
	pub camera_bind_group: wgpu::BindGroup,
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
					bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout],
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
						buffers: &[mesh::VertexAttributes::desc()],
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
						cull_mode: Some(wgpu::Face::Back),
						polygon_mode: wgpu::PolygonMode::Fill,
						unclipped_depth: false,
						conservative: false,
					},
					depth_stencil: None,
					multisample: wgpu::MultisampleState {
						count: 1,
						mask: !0,
						alpha_to_coverage_enabled: false,
					},
					multiview: None,
				});

		Self {
			context: Box::pin(context),
			camera_buffer,
			meshes: Vec::new(),
			render_pipeline,
			model_bind_group_layout,
			camera_bind_group,
		}
	}

	pub fn update_camera(&mut self, camera: &super::camera::Camera) {
		camera.update(&mut self.context.queue, &mut self.camera_buffer)
	}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.context.surface.get_current_texture()?;

		let view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

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
				depth_stencil_attachment: None,
				occlusion_query_set: None,
				timestamp_writes: None,
			});
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

			for mesh in self.meshes.iter() {
				render_pass.set_bind_group(1, &mesh.model_bind_group, &[]);
				render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
				render_pass
					.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				for primitive in mesh.primitives.iter() {
					render_pass.draw_indexed(
						(primitive.start as u32)..(primitive.end as u32),
						0,
						0..1,
					);
				}
			}
		}
		self.context.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		return Ok(());
	}

	pub fn load_gltf<P: AsRef<std::path::Path>>(
		&mut self,
		path: P,
		_is_static: bool,
	) -> Vec<super::camera::Camera> {
		let mut cameras = Vec::new();

		// let (document, buffers, _) = gltf::import(path).unwrap();
		let document = gltf::Gltf::open(&path).unwrap();
		let buffers = gltf::import_buffers(
			&document,
			Some(&std::path::Path::new("examples/sponza/")),
			None,
		)
		.unwrap();

		for scene in document.scenes() {
			for node in scene.nodes() {
				self.check_node(node, &mut cameras, &buffers)
			}
		}

		return cameras;
	}

	fn check_node(
		&mut self,
		node: gltf::Node<'_>,
		cameras: &mut Vec<crate::camera::Camera>,
		buffers: &Vec<gltf::buffer::Data>,
	) {
		for child in node.children() {
			self.check_node(child, cameras, buffers);
		}

		use gltf::camera::Projection::Perspective;
		if node.camera().is_some() {
			let transform = node.transform().decomposed();
			let camera = node.camera().unwrap();
			match camera.projection() {
				Perspective(perspective) => {
					cameras.push(crate::camera::Camera::new(
						transform.0.into(),
						perspective.yfov(),
						perspective.aspect_ratio().unwrap_or(16.0 / 9.0),
						perspective.zfar().unwrap_or_default(),
						perspective.znear(),
						transform.1.into(),
					));
				}
				_ => {}
			}
		} else if node.mesh().is_some() {
			let mesh = self.get_mesh(node, buffers);

			self.meshes.push(mesh);
		} else if let Some(light) = node.light() {
			match light.kind() {
				gltf::khr_lights_punctual::Kind::Directional => {}
				gltf::khr_lights_punctual::Kind::Point => {

				}
				gltf::khr_lights_punctual::Kind::Spot { .. } => {}
			}
		}
	}

	fn get_mesh(&mut self, node: gltf::Node<'_>, buffers: &Vec<gltf::buffer::Data>) -> mesh::Mesh {
		use wgpu::util::DeviceExt;
		let mesh = node.mesh().unwrap();
		let mut vertices = Vec::new();
		let mut indices = Vec::new();
		let mut primitives = Vec::new();

		for gltf_primitive in mesh.primitives() {
			let material = gltf_primitive.material();
			let reader = gltf_primitive.reader(|buffer| Some(&*buffers[buffer.index()]));
			let start = indices.len();

			let index_offset = vertices.len();

			indices.append(
				&mut reader
					.read_indices()
					.unwrap()
					.into_u32()
					.map(|v| (v + index_offset as u32))
					.collect::<Vec<u32>>(),
			);

			let end = indices.len();

			let vertex_start = vertices.len();

			let positions = reader.read_positions().unwrap();
			for position in positions {
				vertices.push(mesh::VertexAttributes {
					position,
					tangent: [0.0; 4],
					normal: [0.0; 3],
					color0: [0.0; 3],
					tex_coord0: [0.0; 2],
					tex_coord1: [0.0; 2],
				});
			}

			let vertex_end = vertices.len();

			primitives.push(mesh::Primitive {
				start: start as u64,
				end: end as u64,
				vertex_start: vertex_start as u64,
				vertex_end: vertex_end as u64,
			});
			{
				if let Some(tangents) = reader.read_tangents() {
					for (i, tangent) in tangents.enumerate() {
						vertices[i].tangent = tangent;
					}
				}

				if let Some(normals) = reader.read_normals() {
					for (i, normal) in normals.enumerate() {
						vertices[i].normal = normal;
					}
				}

				if let Some(colors0) = reader.read_colors(0).map(|v| v.into_rgb_f32()) {
					for (i, color0) in colors0.enumerate() {
						vertices[i].color0 = color0;
					}
				}

				if let Some(tex_coord0) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
					for (i, tex_coord0) in tex_coord0.enumerate() {
						vertices[i].tex_coord0 = tex_coord0;
					}
				}

				if let Some(tex_coord1) = reader.read_tex_coords(1).map(|v| v.into_f32()) {
					for (i, tex_coord1) in tex_coord1.enumerate() {
						vertices[i].tex_coord1 = tex_coord1;
					}
				}
			}
		}

		let vertex_buffer =
			self.context
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("A Vertex Buffer"),
					contents: bytemuck::cast_slice(&vertices[..]),
					usage: wgpu::BufferUsages::VERTEX,
				});

		let index_buffer =
			self.context
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("A Index Buffer"),
					contents: bytemuck::cast_slice(&indices[..]),
					usage: wgpu::BufferUsages::INDEX,
				});

		let transform_buffer =
			self.context
				.device
				.create_buffer_init(&wgpu::util::BufferInitDescriptor {
					label: Some("A transform buffer"),
					contents: bytemuck::cast_slice(&node.transform().matrix()),
					usage: wgpu::BufferUsages::UNIFORM
						| wgpu::BufferUsages::COPY_DST
						| wgpu::BufferUsages::COPY_SRC,
				});

		let model_bind_group = self
			.context
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("A model bind group"),
				layout: &self.model_bind_group_layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: transform_buffer.as_entire_binding(),
				}],
			});

		mesh::Mesh {
			vertex_buffer,
			index_buffer,
			transform_buffer,
			model_bind_group,
			primitives,
		}
	}
}
