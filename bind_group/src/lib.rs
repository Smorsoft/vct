pub mod types;

pub enum WGSL {
	Struct {
		name: String,
		fields: Vec<WGSLStructField>,
	},
	Ty(String),
}

pub struct WGSLStructField {
	pub name: String,
	pub ty: String,
}

pub trait ToWGSL {
	fn to_wgsl() -> WGSL;
}

pub trait HostShareable: Sized {
	const REQUIRED_BUFFER_USAGE_FLAGS: ::wgpu::BufferUsages;
	type Buffer: Buffer;

	unsafe fn as_bytes(&self) -> &[u8] {
		::core::slice::from_raw_parts(
			(self as *const Self) as *const u8,
			::core::mem::size_of::<Self>(),
		)
	}
}

pub trait Buffer: Sized {
	type Source: HostShareable;

	unsafe fn from_buffer(buffer: ::wgpu::Buffer) -> Self;

	unsafe fn as_buffer(&self) -> &::wgpu::Buffer;

	fn map_async(
		&self,
		mode: wgpu::MapMode,
		callback: impl FnOnce(Result<(), wgpu::BufferAsyncError>) + wgpu::WasmNotSend + 'static,
	) {
		let buffer = unsafe { self.as_buffer() };
		buffer.slice(..).map_async(mode, callback);
	}

	fn get_mapped_data<'a>(&'a self) -> &'a Self::Source {
		let buffer_view = unsafe { self.as_buffer() }.slice(..).get_mapped_range();
		unsafe { &*(buffer_view.as_ptr() as *const Self::Source) }
	}
}

pub trait BufferArray: Sized {}

pub fn create_buffer<T: HostShareable + Sized>(
	device: &wgpu::Device,
	usage: ::wgpu::BufferUsages,
	mapped_at_creation: bool,
) -> T::Buffer {
	unsafe {
		T::Buffer::from_buffer(device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: std::mem::size_of::<T>() as u64,
			usage,
			mapped_at_creation,
		}))
	}
}

pub fn create_buffer_init<T: HostShareable + Sized>(
	device: &wgpu::Device,
	item: &T,
	usage: ::wgpu::BufferUsages,
) -> T::Buffer {
	use wgpu::util::DeviceExt;
	unsafe {
		T::Buffer::from_buffer(
			device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: None,
				contents: item.as_bytes(),
				usage,
			}),
		)
	}
}
