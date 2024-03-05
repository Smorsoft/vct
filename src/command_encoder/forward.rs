use crate::command_encoder::*;
use wgpu_helper::types;
use wgpu_helper::bind_group::{BindGroup, BindGroupType};

pub struct ForwardRenderingPass {
	render_pipeline: wgpu::RenderPipeline,
	voxels_read_bind_group_layout: wgpu::BindGroupLayout,
}

impl ForwardRenderingPass {
	pub fn new(renderer: &crate::Renderer) -> Self {
		let shader = renderer
			.device()
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Shader"),
				source: wgpu::ShaderSource::Wgsl(
					include_str!("./shaders/forward.wgsl").into(),
				),
			});

		let voxels_read_bind_group_layout = renderer.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Voxels Bind group layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: false },
						view_dimension: wgpu::TextureViewDimension::D3,
						multisampled: false,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
					count: None,
				},
			],
		});

		let render_pipeline_layout =
			renderer
				.device()
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: Some("Render Pipeline layout"),
					bind_group_layouts: &[
						&crate::camera::CameraBindGroup::get_bind_group_layout(&renderer.device()),
						&crate::ModelBindGroup::get_bind_group_layout(&renderer.device()),
						&crate::MaterialBindGroup::get_bind_group_layout(&renderer.device()),
						&voxels_read_bind_group_layout,
					],
					push_constant_ranges: &[],
				});

		let render_pipeline =
			renderer
				.device()
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: Some("Render Pipeline"),
					layout: Some(&render_pipeline_layout),
					vertex: wgpu::VertexState {
						module: &shader,
						entry_point: "vs_main",
						buffers: &[
							crate::mesh::VertexPosition::desc(),
							crate::mesh::VertexNormals::desc(),
							crate::mesh::VertexColors::desc(),
						],
					},
					fragment: Some(wgpu::FragmentState {
						module: &shader,
						entry_point: "fs_main",
						targets: &[Some(wgpu::ColorTargetState {
							format: renderer.renderer.config.lock().unwrap().format, // TODO: Change for better
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

		Self { render_pipeline, voxels_read_bind_group_layout }
	}
}

impl RenderPassTrait for ForwardRenderingPass {
	fn execute<'manager>(&mut self, command_encoder: &'manager CommandEncoder, global_resources: &mut crate::ResourceManagerHandle<'manager>) -> Option<wgpu::CommandBuffer> {	
		let view = command_encoder.get_surface_texture_view();
		let camera = command_encoder.get_camera().unwrap();
		let depth_buffer = if global_resources.get_resource::<crate::resources::DepthBufferResource>().is_none() {
			let depth_buffer = camera.new_depth_buffer();
			
			let depth_buffer_resource = crate::resources::DepthBufferResource {
				depth_buffer,
			};
			
			global_resources.insert_resource(depth_buffer_resource);
			global_resources.get_resource::<crate::resources::DepthBufferResource>().unwrap()
		} else {
			global_resources.get_resource::<crate::resources::DepthBufferResource>().unwrap()
		};

		let mut encoder = command_encoder.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: None,
		});

		let meshes = command_encoder.get_meshes();
		let materials = command_encoder.get_materials();

		let voxels_resource = global_resources.get_resource::<super::voxelization::VoxelsResource>().unwrap();
		let voxels_bind_group = command_encoder.device().create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &self.voxels_read_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&voxels_resource.color.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&voxels_resource.color.sampler),
				},
			]
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
			render_pass.set_bind_group(0, unsafe { &command_encoder.get_camera_bind_group().unwrap().as_untyped() }, &[]);
			render_pass.set_bind_group(3, &voxels_bind_group, &[]);

			for mesh in meshes.meshes.iter() {
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
						&materials.materials.get(&primitive.material).unwrap().bind_group,
						&[],
					);
					render_pass.draw_indexed(primitive.index.to_owned(), 0, 0..1);
				}
			}
		}

		
		
		Some(encoder.finish())
	}
}
