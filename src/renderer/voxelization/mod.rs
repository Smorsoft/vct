use std::{collections::HashMap, pin::Pin};

use super::context::GraphicsContext;

mod meshify;
mod textures;

pub struct Voxelization {
	voxel_color: super::mesh::Texture,
	voxelizer_layout: wgpu::PipelineLayout,
	voxelizer_pipeline: wgpu::ComputePipeline,
	voxels_bind_group_layout: wgpu::BindGroupLayout,
	mesh_bind_group_layout: wgpu::BindGroupLayout,

	meshify: meshify::Meshify,
}

impl Voxelization {
	pub fn new(
		context: &Pin<Box<GraphicsContext>>,
		model_bind_group_layout: &wgpu::BindGroupLayout,
		material_bind_group_layout: &wgpu::BindGroupLayout,
	) -> Self {
		let meshify = meshify::Meshify::new(context);

		let size = wgpu::Extent3d {
			width: 512,
			height: 512,
			depth_or_array_layers: 512,
		};

		let sampler_descriptor = wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: 0.0,
			lod_max_clamp: 100.0,
			..Default::default()
		};

		let voxel_color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Voxel Color Texture"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D3,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		});

		let voxel_color_view = voxel_color_texture.create_view(&wgpu::TextureViewDescriptor {
			label: Some("Voxel Color View"),
			format: Some(wgpu::TextureFormat::Rgba8Unorm),
			dimension: Some(wgpu::TextureViewDimension::D3),
			..Default::default()
		});
		let voxel_color_sampler = context.device.create_sampler(&sampler_descriptor);

		let voxel_color = super::mesh::Texture {
			texture: voxel_color_texture,
			view: voxel_color_view,
			sampler: voxel_color_sampler,
		};

		let voxelizer_shader = context
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Voxelizer shader"),
				source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
					"../shaders/voxelization.wgsl"
				))),
			});

		let voxels_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("Voxels BindGroup Layout"),
					entries: &[wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::StorageTexture {
							access: wgpu::StorageTextureAccess::WriteOnly,
							format: wgpu::TextureFormat::Rgba8Unorm,
							view_dimension: wgpu::TextureViewDimension::D3,
						},
						count: None,
					}],
				});

		let mesh_bind_group_layout =
			context
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					label: Some("Mesh Bind Group Layout"),
					entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: true },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 1,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: true },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 2,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: true },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 3,
							visibility: wgpu::ShaderStages::COMPUTE,
							ty: wgpu::BindingType::Buffer {
								ty: wgpu::BufferBindingType::Storage { read_only: true },
								has_dynamic_offset: false,
								min_binding_size: None,
							},
							count: None,
						},
					],
				});

		let voxelizer_layout =
			context
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("Voxelizer Layout"),
					bind_group_layouts: &[
						&voxels_bind_group_layout,
						&mesh_bind_group_layout,
						&model_bind_group_layout,
						&material_bind_group_layout,
					],
					push_constant_ranges: &[],
				});

		let voxelizer_pipeline =
			context
				.device
				.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
					label: Some("Voxelizer pipeline"),
					layout: Some(&voxelizer_layout),
					module: &voxelizer_shader,
					entry_point: "main",
				});

		Self {
			voxel_color,
			voxelizer_layout,
			voxelizer_pipeline,
			voxels_bind_group_layout,
			mesh_bind_group_layout,
			meshify,
		}
	}

	pub fn voxelize(
		&mut self,
		context: &Pin<Box<GraphicsContext>>,
		meshes: &Vec<super::mesh::Mesh>,
		materials: &HashMap<uuid::Uuid, super::mesh::Material>,
	) {
		let create_mesh_bind_group = |mesh: &super::mesh::Mesh, offset: u64, size: u64| {
			context
				.device
				.create_bind_group(&wgpu::BindGroupDescriptor {
					label: Some("Mesh Bind Group"),
					layout: &self.mesh_bind_group_layout,
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
								buffer: &mesh.index_buffer,
								offset,
								size: std::num::NonZeroU64::new(size),
							}),
						},
						wgpu::BindGroupEntry {
							binding: 1,
							resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
								buffer: &mesh.vertex_buffer,
								offset: mesh.positions.start,
								size: std::num::NonZeroU64::new(
									mesh.positions.end - mesh.positions.start,
								),
							}),
						},
						wgpu::BindGroupEntry {
							binding: 2,
							resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
								buffer: &mesh.vertex_buffer,
								offset: mesh.normals.start,
								size: std::num::NonZeroU64::new(
									mesh.normals.end - mesh.normals.start,
								),
							}),
						},
						wgpu::BindGroupEntry {
							binding: 3,
							resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
								buffer: &mesh.vertex_buffer,
								offset: mesh.colors.start,
								size: std::num::NonZeroU64::new(
									mesh.colors.end - mesh.colors.start,
								),
							}),
						},
					],
				})
		};

		let mut encoder = context
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Voxelization Encoder"),
			});

		let voxels_bind_group = context
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("Voxels Bind Group"),
				layout: &self.voxels_bind_group_layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&self.voxel_color.view),
				}],
			});

		let mut runs = Vec::new();
		for mesh in meshes.iter() {
			let mut mesh_runs = Vec::new();

			for primitive in mesh.primitives.iter() {
				let indices_num = primitive.index.end - primitive.index.start;
				let triangles_num = indices_num / 3;
				let alignment = context.device.limits().min_storage_buffer_offset_alignment * 3;
				let dispatch_group =
					(65535 /*Max num workgroup dispatches*/ / alignment) * alignment;

				let run = if triangles_num <= dispatch_group {
					let bind_group = create_mesh_bind_group(
						mesh,
						primitive.index.start as u64
							* std::mem::size_of::<super::mesh::Index>() as u64,
						(primitive.index.end - primitive.index.start) as u64
							* std::mem::size_of::<super::mesh::Index>() as u64,
					);

					vec![(bind_group, triangles_num)]
				} else {
					let mut offsets = Vec::new();
					let number_of_runs =
						f64::ceil(triangles_num as f64 / dispatch_group as f64) as u32;
						
					for i in 0..number_of_runs {
						let offset = primitive.index.start + (dispatch_group * 3 * i);

						let size = if (triangles_num - (dispatch_group * i))
							> dispatch_group
						{
							dispatch_group
						} else {
							triangles_num - (dispatch_group * i)
						};

						let bind_group = create_mesh_bind_group(
							mesh,
							offset as u64 * std::mem::size_of::<super::mesh::Index>() as u64,
							size as u64 * std::mem::size_of::<super::mesh::Index>() as u64,
						);
						offsets.push((bind_group, size));
					}

					offsets
				};

				mesh_runs.push(run);
			}

			runs.push(mesh_runs);
		}

		{
			let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
				label: Some("Voxelization pass"),
				timestamp_writes: None,
			});

			compute_pass.set_pipeline(&self.voxelizer_pipeline);
			compute_pass.set_bind_group(0, &voxels_bind_group, &[]);

			

			for (mesh, mesh_runs) in meshes.iter().zip(runs.iter()) {
				compute_pass.set_bind_group(2, &mesh.model_bind_group, &[]);
				for (primitive, prim_runs) in mesh.primitives.iter().zip(mesh_runs.iter()) {
					compute_pass.set_bind_group(3, &materials[&primitive.material].bind_group, &[]);

					for (bind_group, size) in prim_runs.iter() {
						compute_pass.set_bind_group(1, bind_group, &[]);

						compute_pass.dispatch_workgroups(*size, 1, 1);
					}
				}
			}
		}

		context.queue.submit(Some(encoder.finish()));
	}

	pub fn meshify(&mut self, context: &Pin<Box<GraphicsContext>>) {
		self.meshify.meshify(context, &self.voxel_color);
	}

	pub fn render(
		&mut self,
		context: &Pin<Box<GraphicsContext>>,
		depth_buffer: &crate::renderer::mesh::Texture,
		camera_bind_group: &wgpu::BindGroup,
	) {
		self.meshify
			.render(context, depth_buffer, camera_bind_group);
	}
}
