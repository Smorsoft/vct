use std::ops::Range;

use wgpu_helper::{
	bind_group::{BindGroup, BindGroupType},
	*,
};

pub const COMPUTE_MESH_BIND_GROUP_LAYOUT: &'static wgpu::BindGroupLayoutDescriptor =
	&wgpu::BindGroupLayoutDescriptor {
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
	};

#[derive(BindGroup)]
#[layout(COMPUTE_MESH_BIND_GROUP_LAYOUT)]
pub struct ComputeMeshBindGroup {}

pub struct Mesh {
	pub vertex_buffer: wgpu::Buffer,
	pub positions: Range<wgpu::BufferAddress>,
	pub normals: Range<wgpu::BufferAddress>,
	pub colors: Range<wgpu::BufferAddress>,
	pub index_buffer: wgpu::Buffer,
	pub transform_buffer: wgpu::Buffer,
	pub model_bind_group: wgpu::BindGroup,
	pub primitives: Vec<Primitive>,
}

pub struct Primitive {
	pub index: Range<u32>,
	// pub skin: Range<wgpu::BufferAddress>,
	pub material: crate::Id,
}

pub struct Material {
	pub diffuse: Texture,
	pub metallic_roughness: Texture,
	pub normal: Texture,
	pub bind_group: wgpu::BindGroup,
}

pub struct Texture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}


pub type Index = u32;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexPosition([f32; 3]);

impl VertexPosition {
	pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			0 => Float32x3,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexPosition>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexNormals {
	pub normals: [i8; 4],
	pub tangents: [i8; 4],
}

impl VertexNormals {
	pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			1 => Uint32,
			2 => Uint32,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexNormals>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}

/// TODO: Figure out why uv0 cant be unpacked as u16 from u32
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexColors {
	pub uv0: [f32; 2],
	pub uv1: [f32; 2],
	pub color: [u8; 4],
}

impl VertexColors {
	pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
		const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
			3 => Float32x2,
			4 => Float32x2,
			5 => Uint32,
		];

		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<VertexColors>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: ATTRIBUTES,
		}
	}
}

#[repr(C)]
pub struct VertexSkinning {
	pub joint_index: [u16; 4],
	pub joint_weight: [u16; 4],
}
