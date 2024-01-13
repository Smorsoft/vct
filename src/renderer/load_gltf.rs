use std::collections::HashMap;

use super::context::GraphicsContext;

use super::mesh::*;

pub fn load_gltf<P: AsRef<std::path::Path>>(
	context: &GraphicsContext,
	meshes: &mut Vec<Mesh>,
	materials: &mut HashMap<uuid::Uuid, Material>,
	model_bind_group_layout: &wgpu::BindGroupLayout,
	material_bind_group_layout: &wgpu::BindGroupLayout,
	path: P,
	_is_static: bool,
) -> Vec<crate::camera::Camera> {
	let mut cameras = Vec::new();

	let (document, buffers, textures) = gltf::import(path).unwrap();

	for scene in document.scenes() {
		for node in scene.nodes() {
			check_node(
				context,
				meshes,
				materials,
				model_bind_group_layout,
				material_bind_group_layout,
				node,
				&mut cameras,
				&buffers,
				&textures,
			)
		}
	}

	return cameras;
}

fn check_node(
	context: &GraphicsContext,
	meshes: &mut Vec<Mesh>,
	materials: &mut HashMap<uuid::Uuid, Material>,
	model_bind_group_layout: &wgpu::BindGroupLayout,
	material_bind_group_layout: &wgpu::BindGroupLayout,
	node: gltf::Node<'_>,
	cameras: &mut Vec<crate::camera::Camera>,
	buffers: &Vec<gltf::buffer::Data>,
	textures: &Vec<gltf::image::Data>,
) {
	for child in node.children() {
		check_node(
			context,
			meshes,
			materials,
			model_bind_group_layout,
			material_bind_group_layout,
			child,
			cameras,
			buffers,
			&textures,
		);
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
		let mesh = get_mesh(
			context,
			meshes,
			materials,
			model_bind_group_layout,
			material_bind_group_layout,
			node,
			buffers,
			&textures,
		);

		meshes.push(mesh);
	} else if let Some(light) = node.light() {
		match light.kind() {
			gltf::khr_lights_punctual::Kind::Directional => {}
			gltf::khr_lights_punctual::Kind::Point => {}
			gltf::khr_lights_punctual::Kind::Spot { .. } => {}
		}
	}
}

fn get_mesh(
	context: &GraphicsContext,
	meshes: &mut Vec<Mesh>,
	materials: &mut HashMap<uuid::Uuid, Material>,
	model_bind_group_layout: &wgpu::BindGroupLayout,
	material_bind_group_layout: &wgpu::BindGroupLayout,
	node: gltf::Node<'_>,
	buffers: &Vec<gltf::buffer::Data>,
	textures: &Vec<gltf::image::Data>,
) -> Mesh {
	const I8_MAX: f32 = i8::MAX as f32;
	// const U8_MAX: f32 = u8::MAX as f32;

	use wgpu::util::DeviceExt;
	let mesh = node.mesh().unwrap();
	let mut vertex_data: Vec<u8> = Vec::new();
	let mut vertex_count = 0 as usize;

	let mut indices = Vec::new();
	let mut primitives = Vec::new();

	for gltf_primitive in mesh.primitives() {
		let material = gltf_primitive.material();
		let id = uuid::Uuid::new_v4();
		get_material(
			context,
			&id,
			meshes,
			materials,
			model_bind_group_layout,
			material_bind_group_layout,
			&material,
			&textures,
		);

		let reader = gltf_primitive.reader(|buffer| Some(&buffers[buffer.index()]));

		align_vector(
			&mut indices,
			context.device.limits().min_storage_buffer_offset_alignment as usize,
			0,
		);

		let start = indices.len();

		let index_offset = vertex_count;

		// for index in reader.read_indices().unwrap().into_u32() {
		// 	index_data.extend_from_slice(bytemuck::cast_slice(&[index + index_offset as u32]))
		// }

		indices.append(
			&mut reader
				.read_indices()
				.unwrap()
				.into_u32()
				.map(|v| (v + index_offset as u32))
				.collect::<Vec<u32>>(),
		);

		let end = indices.len();

		let positions = reader.read_positions().unwrap();
		for position in positions {
			vertex_count += 1;
			vertex_data.extend_from_slice(bytemuck::cast_slice(&position));
		}

		primitives.push(Primitive {
			index: (start as u32)..(end as u32),
			material: id,
		});
	}

	let positions = 0..(vertex_data.len() as wgpu::BufferAddress);

	align_vector(
		&mut vertex_data,
		context.device.limits().min_storage_buffer_offset_alignment as usize,
		0,
	);

	// Normals
	let normals_start = vertex_data.len() as wgpu::BufferAddress;
	for gltf_primitive in mesh.primitives() {
		let reader = gltf_primitive.reader(|buffer| Some(&buffers[buffer.index()]));

		let tangents: Box<dyn std::iter::Iterator<Item = [f32; 4]>> = match reader.read_tangents() {
			Some(t) => Box::new(t),
			None => Box::new(std::iter::repeat([0.0_f32; 4]).take(vertex_count)),
		};

		let normals: Box<dyn std::iter::Iterator<Item = [f32; 3]>> = match reader.read_normals() {
			Some(t) => Box::new(t),
			None => Box::new(std::iter::repeat([0.0_f32; 3]).take(vertex_count)),
		};

		for (tangent, normal) in tangents.zip(normals) {
			let vertex_normals = VertexNormals {
				normals: [
					(I8_MAX * normal[0]) as i8,
					(I8_MAX * normal[1]) as i8,
					(I8_MAX * normal[2]) as i8,
					0,
				],
				tangents: [
					(I8_MAX * tangent[0]) as i8,
					(I8_MAX * tangent[1]) as i8,
					(I8_MAX * tangent[2]) as i8,
					(I8_MAX * tangent[3]) as i8,
				],
			};

			vertex_data.extend_from_slice(bytemuck::cast_slice(&[vertex_normals]));
		}
	}

	let normals = normals_start..(vertex_data.len() as wgpu::BufferAddress);

	align_vector(
		&mut vertex_data,
		context.device.limits().min_storage_buffer_offset_alignment as usize,
		0,
	);

	// Colors
	let colors_start = vertex_data.len() as wgpu::BufferAddress;
	for gltf_primitive in mesh.primitives() {
		let reader = gltf_primitive.reader(|buffer| Some(&buffers[buffer.index()]));

		let colors: Box<dyn std::iter::Iterator<Item = [u8; 4]>> =
			match reader.read_colors(0).map(|v| v.into_rgba_u8()) {
				Some(t) => Box::new(t),
				None => Box::new(std::iter::repeat([0, 0, 0, u8::MAX]).take(vertex_count)),
			};

		let uv0: Box<dyn std::iter::Iterator<Item = [f32; 2]>> =
			match reader.read_tex_coords(0).map(|v| v.into_f32()) {
				Some(t) => Box::new(t),
				None => Box::new(std::iter::repeat([0.0; 2]).take(vertex_count)),
			};

		let uv1: Box<dyn std::iter::Iterator<Item = [f32; 2]>> =
			match reader.read_tex_coords(1).map(|v| v.into_f32()) {
				Some(t) => Box::new(t),
				None => Box::new(std::iter::repeat([0.0; 2]).take(vertex_count)),
			};

		for ((uv0, uv1), color) in uv0.zip(uv1).zip(colors) {
			let vertex_colors = VertexColors { uv0, uv1, color };

			vertex_data.extend_from_slice(bytemuck::cast_slice(&[vertex_colors]));
		}
	}

	let colors = colors_start..(vertex_data.len() as wgpu::BufferAddress);

	let vertex_buffer = context
		.device
		.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("A Vertex Buffer"),
			contents: &vertex_data[..],
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
		});

	let index_buffer = context
		.device
		.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("A Index Buffer"),
			contents: bytemuck::cast_slice(&indices[..]),
			usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
		});

	let transform_buffer = context
		.device
		.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("A transform buffer"),
			contents: bytemuck::cast_slice(&node.transform().matrix()),
			usage: wgpu::BufferUsages::UNIFORM
				| wgpu::BufferUsages::COPY_DST
				| wgpu::BufferUsages::COPY_SRC,
		});

	let model_bind_group = context
		.device
		.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("A model bind group"),
			layout: model_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: transform_buffer.as_entire_binding(),
			}],
		});

	Mesh {
		vertex_buffer,
		positions,
		normals,
		colors,
		index_buffer,
		transform_buffer,
		model_bind_group,
		primitives,
	}
}

fn get_material(
	context: &GraphicsContext,
	id: &uuid::Uuid,
	meshes: &mut Vec<Mesh>,
	materials: &mut HashMap<uuid::Uuid, Material>,
	model_bind_group_layout: &wgpu::BindGroupLayout,
	material_bind_group_layout: &wgpu::BindGroupLayout,
	material: &gltf::Material,
	textures: &Vec<gltf::image::Data>,
) {
	use image::GenericImageView;

	const DEFAULT_DIFFUSE: &[u8] = include_bytes!("default_textures/DiffuseLargeMap.png");
	const DEFAULT_METAL: &[u8] = include_bytes!("default_textures/MetallicRoughnessMap.png");
	const DEFAULT_NORMAL: &[u8] = include_bytes!("default_textures/NormalMap.png");

	let new_default_sampler = |context: &GraphicsContext| -> wgpu::Sampler {
		return context.device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("Default Sampler"),
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Linear,
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
					get_texture(context, &data[..], texture_data.width, texture_data.height);
				let sampler = new_default_sampler(context);
				(texture, view, sampler)
			}
			gltf::image::Format::R8G8B8A8 => {
				let (texture, view) = get_texture(
					context,
					&texture_data.pixels,
					texture_data.width,
					texture_data.height,
				);
				let sampler = new_default_sampler(context);
				(texture, view, sampler)
			}
			_ => {
				let image = image::load_from_memory(DEFAULT_DIFFUSE).unwrap();
				let dimensions = image.dimensions();
				let (texture, view) =
					get_texture(context, &image.to_rgba8(), dimensions.0, dimensions.1);
				(texture, view, new_default_sampler(context))
			}
		};

		(texture, view, sampler)
	} else {
		let image = image::load_from_memory(DEFAULT_DIFFUSE).unwrap();
		let dimensions = image.dimensions();
		let (texture, view) = get_texture(context, &image.to_rgba8(), dimensions.0, dimensions.1);
		(texture, view, new_default_sampler(context))
	};

	let (metal_texture, metal_view, metal_sampler) = {
		let image = image::load_from_memory(DEFAULT_METAL).unwrap();
		let dimensions = image.dimensions();
		let (texture, view) = get_texture(context, &image.to_rgba8(), dimensions.0, dimensions.1);
		(texture, view, new_default_sampler(context))
	};

	let (normal_texture, normal_view, normal_sampler) = {
		let image = image::load_from_memory(DEFAULT_NORMAL).unwrap();
		let dimensions = image.dimensions();
		let (texture, view) = get_texture(context, &image.to_rgba8(), dimensions.0, dimensions.1);
		(texture, view, new_default_sampler(context))
	};

	let bind_group = context
		.device
		.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: material_bind_group_layout,
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

	materials.insert(
		id.to_owned(),
		Material {
			diffuse: Texture {
				texture: diffuse_texture,
				view: diffuse_view,
				sampler: diffuse_sampler,
			},
			metallic_roughness: Texture {
				texture: metal_texture,
				view: metal_view,
				sampler: metal_sampler,
			},
			normal: Texture {
				texture: normal_texture,
				view: normal_view,
				sampler: normal_sampler,
			},
			bind_group,
		},
	);
}

fn get_texture(
	context: &GraphicsContext,
	bytes: &[u8],
	width: u32,
	height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
	let texture_size = wgpu::Extent3d {
		width,
		height,
		depth_or_array_layers: 1,
	};

	let texture = context.device.create_texture(&wgpu::TextureDescriptor {
		size: texture_size,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Rgba8UnormSrgb,
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		label: Some("Texture"),
		view_formats: &[],
	});

	context.queue.write_texture(
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

	let view = texture.create_view(&Default::default());

	return (texture, view);
}

fn align_vector<T: Sized + Clone>(vec: &mut Vec<T>, alignment: usize, fill_value: T) {
	assert_eq!(alignment % core::mem::size_of::<T>(), 0);

	let alignment_sized = alignment / core::mem::size_of::<T>();

	if vec.len() % alignment_sized != 0 {
		let aligned_len =
			f64::ceil(vec.len() as f64 / alignment_sized as f64) as usize * alignment_sized;

		for _ in 0..(aligned_len - vec.len()) {
			vec.push(fill_value.to_owned());
		}
	}

	assert_eq!((vec.len() * core::mem::size_of::<T>()) % alignment, 0);
}
