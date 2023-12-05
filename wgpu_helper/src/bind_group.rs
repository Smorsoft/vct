pub trait BindGroup: Sized {
	fn get_bind_group_layout_descriptor() -> ::wgpu::BindGroupLayoutDescriptor<'static>;
}