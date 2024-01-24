use std::sync::OnceLock;

pub trait BindGroupType: Sized {
	const BIND_GROUP_LAYOUT_DESCRIPTOR: &'static ::wgpu::BindGroupLayoutDescriptor<'static>;
	type BindGroup: BindGroup;
	
	fn get_bind_group_lock() -> &'static OnceLock<::wgpu::BindGroupLayout>;
	
	fn get_bind_group_layout(device: &::wgpu::Device) -> &::wgpu::BindGroupLayout {
		Self::get_bind_group_lock().get_or_init(|| {
			device.create_bind_group_layout(Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
		})
	}

	fn to_bind_group(&self, device: &::wgpu::Device, label: Option<&str>) -> Self::BindGroup;
}

pub trait BindGroup: Sized {
	type Source: BindGroupType;

	unsafe fn from_untyped(untyped: ::wgpu::BindGroup) -> Self {
		assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<::wgpu::BindGroup>());
		core::mem::transmute_copy::<core::mem::ManuallyDrop<::wgpu::BindGroup>, Self>(&core::mem::ManuallyDrop::new(untyped))
	}
	unsafe fn as_untyped(&self) -> &::wgpu::BindGroup {
		assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<::wgpu::BindGroup>());
		::core::mem::transmute(self)
	}
}
