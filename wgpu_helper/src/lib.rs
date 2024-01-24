pub mod types;
pub mod bind_group;

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

	fn create_buffer(device: &wgpu::Device, usage: wgpu::BufferUsages, mapped_at_creation: bool) -> Self::Buffer {
		create_buffer::<Self>(device, usage, mapped_at_creation)
	}

	fn create_buffer_init(&self, device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self::Buffer {
		create_buffer_init::<Self>(device, self, usage)
	}
}

pub trait Buffer: Sized {
	type Source: HostShareable;

	unsafe fn from_buffer(buffer: ::wgpu::Buffer) -> Self {
		assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<::wgpu::Buffer>());
		std::mem::transmute_copy::<std::mem::ManuallyDrop<::wgpu::Buffer>, Self>(&std::mem::ManuallyDrop::new(buffer))
	}
	unsafe fn as_buffer(&self) -> &::wgpu::Buffer {
		assert_eq!(core::mem::size_of::<Self>(), core::mem::size_of::<::wgpu::Buffer>());
		::core::mem::transmute(self)
	}

	fn map_async(
		&self,
		mode: wgpu::MapMode,
		callback: impl FnOnce(Result<(), wgpu::BufferAsyncError>) + wgpu::WasmNotSend + 'static,
	) {

		let buffer = unsafe { self.as_buffer() };
		buffer.slice(..).map_async(mode, callback);
	}

	fn map_sync<'a>(&'a self, device: &wgpu::Device) -> &'a Self::Source {
		let (tx, rx) = std::sync::mpsc::channel::<Result<(), wgpu::BufferAsyncError>>();

		self.map_async(wgpu::MapMode::Read, move |res| {
			tx.send(res).unwrap();
		});

		loop {
			device.poll(wgpu::MaintainBase::Wait);

			match rx.recv() {
				Ok(_) => {
					return self.get_mapped_data();					
				}
				Err(e) => {
					panic!("{:#?}", e);
				}
			}
		}
	}

	fn map_sync_mut<'a>(&'a mut self, device: &wgpu::Device) -> &'a mut Self::Source {
		let (tx, rx) = std::sync::mpsc::channel::<Result<(), wgpu::BufferAsyncError>>();

		self.map_async(wgpu::MapMode::Read, move |res| {
			tx.send(res).unwrap();
		});

		loop {
			device.poll(wgpu::MaintainBase::Wait);

			match rx.recv() {
				Ok(_) => {
					return self.get_mapped_data_mut();					
				}
				Err(e) => {
					panic!("{:#?}", e);
				}
			}
		}
	}

	fn get_mapped_data<'a>(&'a self) -> &'a Self::Source {
		let buffer_view = unsafe { self.as_buffer() }.slice(..).get_mapped_range();
		unsafe { &*(buffer_view.as_ptr() as *const Self::Source) }
	}

	fn get_mapped_data_mut<'a>(&'a mut self) -> &'a mut Self::Source {
		let buffer_view = unsafe { self.as_buffer() }.slice(..).get_mapped_range_mut();
		unsafe { &mut *(buffer_view.as_ptr() as *mut Self::Source) }
	}
}

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