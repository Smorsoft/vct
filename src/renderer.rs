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
	pub materials: Vec<mesh::Material>,
	pub render_pipeline: wgpu::RenderPipeline,
	pub model_bind_group_layout: wgpu::BindGroupLayout,
	pub material_bind_group_layout: wgpu::BindGroupLayout,
	pub camera_bind_group: wgpu::BindGroup,
	pub depth_buffer: mesh::Texture,
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

		let material_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("Material Bind group layout"),
					entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0,
							visibility: wgpu::ShaderStages::FRAGMENT,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
								view_dimension: wgpu::TextureViewDimension::D2,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 1,
							visibility: wgpu::ShaderStages::FRAGMENT,
							ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 2,
							visibility: wgpu::ShaderStages::FRAGMENT,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
								view_dimension: wgpu::TextureViewDimension::D2,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 3,
							visibility: wgpu::ShaderStages::FRAGMENT,
							ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 4,
							visibility: wgpu::ShaderStages::FRAGMENT,
							ty: wgpu::BindingType::Texture {
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
								view_dimension: wgpu::TextureViewDimension::D2,
								multisampled: false,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 5,
							visibility: wgpu::ShaderStages::FRAGMENT,
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

		Self {
			context: Box::pin(context),
			camera_buffer,
			meshes: Vec::new(),
			materials: Vec::new(),
			render_pipeline,
			model_bind_group_layout,
			material_bind_group_layout,
			camera_bind_group,
			depth_buffer,
		}
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.context.config.width = width;
		self.context.config.height = height;
		self.context.surface.configure(&self.context.device, &self.context.config);

		self.depth_buffer = mesh::Texture::create_depth_texture(&self.context.device, &self.context.config)
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
				render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
				render_pass
					.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				for primitive in mesh.primitives.iter() {
					render_pass.set_bind_group(
						2,
						&self.materials[primitive.material].bind_group,
						&[],
					);
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

		let (document, buffers, textures) = gltf::import(path).unwrap();
		// let document = gltf::Gltf::open(&path).unwrap();
		// let buffers = gltf::import_buffers(
		// 	&document,
		// 	Some(&std::path::Path::new("examples/sponza/")),
		// 	None,
		// )
		// .unwrap();

		for scene in document.scenes() {
			for node in scene.nodes() {
				self.check_node(node, &mut cameras, &buffers, &textures)
			}
		}

		return cameras;
	}

	fn check_node(
		&mut self,
		node: gltf::Node<'_>,
		cameras: &mut Vec<crate::camera::Camera>,
		buffers: &Vec<gltf::buffer::Data>,
		textures: &Vec<gltf::image::Data>,
	) {
		for child in node.children() {
			self.check_node(child, cameras, buffers, &textures);
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
			let mesh = self.get_mesh(node, buffers, &textures);

			self.meshes.push(mesh);
		} else if let Some(light) = node.light() {
			match light.kind() {
				gltf::khr_lights_punctual::Kind::Directional => {}
				gltf::khr_lights_punctual::Kind::Point => {}
				gltf::khr_lights_punctual::Kind::Spot { .. } => {}
			}
		}
	}

	fn get_mesh(
		&mut self,
		node: gltf::Node<'_>,
		buffers: &Vec<gltf::buffer::Data>,
		textures: &Vec<gltf::image::Data>,
	) -> mesh::Mesh {
		use wgpu::util::DeviceExt;
		let mesh = node.mesh().unwrap();
		let mut vertices: Vec<mesh::VertexAttributes> = Vec::new();
		let mut indices = Vec::new();
		let mut primitives = Vec::new();

		for gltf_primitive in mesh.primitives() {
			let material = gltf_primitive.material();
			self.get_material(&material, &textures);

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
					// color0: [0.0; 3],
					tex_coord0: [0.0; 2],
					// tex_coord1: [0.0; 2],
				});
			}

			let vertex_end = vertices.len();

			primitives.push(mesh::Primitive {
				start: start as u64,
				end: end as u64,
				vertex_start: vertex_start as u64,
				vertex_end: vertex_end as u64,
				material: self.materials.len() - 1,
			});
			{
				if let Some(tangents) = reader.read_tangents() {
					for (i, tangent) in tangents.enumerate() {
						vertices[vertex_start + i].tangent = tangent;
					}
				}

				if let Some(normals) = reader.read_normals() {
					for (i, normal) in normals.enumerate() {
						vertices[vertex_start + i].normal = normal;
					}
				}

				if let Some(tex_coord0) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
					for (i, tex_coord0) in tex_coord0.enumerate() {
						vertices[vertex_start + i].tex_coord0 = tex_coord0;
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

	fn get_material(&mut self, material: &gltf::Material, textures: &Vec<gltf::image::Data>) {
		use image::GenericImageView;

		const DEFAULT_DIFFUSE: &[u8] =
			include_bytes!("renderer/default_textures/DiffuseLargeMap.png");
		const DEFAULT_METAL: &[u8] =
			include_bytes!("renderer/default_textures/MetallicRoughnessMap.png");
		const DEFAULT_NORMAL: &[u8] = include_bytes!("renderer/default_textures/NormalMap.png");

		let new_default_sampler = |renderer: &Renderer| -> wgpu::Sampler {
			return renderer
				.context
				.device
				.create_sampler(&wgpu::SamplerDescriptor {
					label: Some("Default Sampler"),
					address_mode_u: wgpu::AddressMode::Repeat,
					address_mode_v: wgpu::AddressMode::Repeat,
					address_mode_w: wgpu::AddressMode::Repeat,
					mag_filter: wgpu::FilterMode::Nearest,
					min_filter: wgpu::FilterMode::Nearest,
					..Default::default()
				});
		};

		let (diffuse_texture, diffuse_view, diffuse_sampler) = if let Some(texture_info) =
			material.pbr_metallic_roughness().base_color_texture()
		{
			let texture_data = &textures[texture_info.texture().source().index()];
			let (texture, view, sampler) = match texture_data.format {
				gltf::image::Format::R8G8B8 => {
					let mut data = Vec::new();

					for (i, _) in texture_data.pixels.iter().enumerate().step_by(3) {
						data.push(texture_data.pixels[i]);
						data.push(texture_data.pixels[i + 1]);
						data.push(texture_data.pixels[i + 2]);
						data.push(255);
					}

					let (texture, view) =
						self.get_texture(&data[..], texture_data.width, texture_data.height);

					// let sampler = self.context.device.create_sampler(&wgpu::SamplerDescriptor {
					// 	label: Some("Custom Sampler"),
					// 	address_mode_u: texture_info.texture().sampler().

					// });

					let sampler = new_default_sampler(&self);

					(texture, view, sampler)
				}
				_ => {
					let image = image::load_from_memory(DEFAULT_DIFFUSE).unwrap();
					let dimensions = image.dimensions();
					let (texture, view) =
						self.get_texture(&image.to_rgba8(), dimensions.0, dimensions.1);
					(texture, view, new_default_sampler(self))
				}
			};

			(texture, view, sampler)
		} else {
			let image = image::load_from_memory(DEFAULT_DIFFUSE).unwrap();
			let dimensions = image.dimensions();
			let (texture, view) = self.get_texture(&image.to_rgba8(), dimensions.0, dimensions.1);
			(texture, view, new_default_sampler(self))
		};

		let (metal_texture, metal_view, metal_sampler) = {
			let image = image::load_from_memory(DEFAULT_METAL).unwrap();
			let dimensions = image.dimensions();
			let (texture, view) = self.get_texture(&image.to_rgba8(), dimensions.0, dimensions.1);
			(texture, view, new_default_sampler(self))
		};

		let (normal_texture, normal_view, normal_sampler) = {
			let image = image::load_from_memory(DEFAULT_NORMAL).unwrap();
			let dimensions = image.dimensions();
			let (texture, view) = self.get_texture(&image.to_rgba8(), dimensions.0, dimensions.1);
			(texture, view, new_default_sampler(self))
		};

		let bind_group = self
			.context
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &self.material_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
					},
					wgpu::BindGroupEntry {
						binding: 2,
						resource: wgpu::BindingResource::TextureView(&metal_view),
					},
					wgpu::BindGroupEntry {
						binding: 3,
						resource: wgpu::BindingResource::Sampler(&metal_sampler),
					},
					wgpu::BindGroupEntry {
						binding: 4,
						resource: wgpu::BindingResource::TextureView(&normal_view),
					},
					wgpu::BindGroupEntry {
						binding: 5,
						resource: wgpu::BindingResource::Sampler(&normal_sampler),
					},
				],
				label: Some("material bind group"),
			});

		self.materials.push(mesh::Material {
			diffuse: mesh::Texture {
				texture: diffuse_texture,
				view: diffuse_view,
				sampler: diffuse_sampler,
			},
			metallic_roughness: mesh::Texture {
				texture: metal_texture,
				view: metal_view,
				sampler: metal_sampler,
			},
			normal: mesh::Texture {
				texture: normal_texture,
				view: normal_view,
				sampler: normal_sampler,
			},
			bind_group,
		});
	}

	fn get_texture(
		&mut self,
		bytes: &[u8],
		width: u32,
		height: u32,
	) -> (wgpu::Texture, wgpu::TextureView) {
		let texture_size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};

		let texture = self
			.context
			.device
			.create_texture(&wgpu::TextureDescriptor {
				size: texture_size,
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Rgba8UnormSrgb,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				label: Some("Texture"),
				view_formats: &[],
			});

		self.context.queue.write_texture(
			wgpu::ImageCopyTexture {
				texture: &texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			&bytes,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * width),
				rows_per_image: Some(height),
			},
			texture_size,
		);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		return (texture, view);
	}
}
