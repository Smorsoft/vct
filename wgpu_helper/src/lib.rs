use core::marker::PhantomData;

pub mod bind_group;
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

	unsafe fn as_bytes(&self) -> &[u8] {
		::core::slice::from_raw_parts(
			(self as *const Self) as *const u8,
			::core::mem::size_of::<Self>(),
		)
	}
}

pub trait BufferTrait: Sized {
	type Source: HostShareable;

	fn get_slice(&self) -> wgpu::BufferSlice;

	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a>;

	fn get_buffer(&self) -> &wgpu::Buffer;

	fn get_size(&self) -> u64;

	fn get_offset(&self, offset: u64) -> u64;

	fn copy_to_buffer<T: BufferTrait>(&self, encoder: &mut wgpu::CommandEncoder, destination: &T) {
		encoder.copy_buffer_to_buffer(
			self.get_buffer(),
			self.get_offset(0),
			destination.get_buffer(),
			destination.get_offset(0),
			self.get_size(),
		);
	}

	fn map_async(
		&self,
		mode: wgpu::MapMode,
		callback: impl FnOnce(Result<(), wgpu::BufferAsyncError>) + wgpu::WasmNotSend + 'static,
	) {
		self.get_slice().map_async(mode, callback);
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
		let buffer_view = self.get_slice().get_mapped_range();
		unsafe { &*(buffer_view.as_ptr() as *const Self::Source) }
	}

	fn get_mapped_data_mut<'a>(&'a mut self) -> &'a mut Self::Source {
		let buffer_view = self.get_slice().get_mapped_range_mut();
		unsafe { &mut *(buffer_view.as_ptr() as *mut Self::Source) }
	}
}

pub struct Buffer<T: HostShareable> {
	buffer: wgpu::Buffer,
	phantom_data: PhantomData<T>,
}

impl<T: HostShareable> Buffer<T> {
	pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, mapped_at_creation: bool) -> Self {
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: core::mem::size_of::<T>() as u64,
			usage,
			mapped_at_creation,
		});

		Self {
			buffer,
			phantom_data: PhantomData::default(),
		}
	}

	pub fn new_init(device: &wgpu::Device, data: &T, usage: wgpu::BufferUsages) -> Self {
		use wgpu::util::DeviceExt;
		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: unsafe { data.as_bytes() },
			usage,
		});

		Self {
			buffer,
			phantom_data: PhantomData::default(),
		}
	}
}

impl<T: HostShareable> BufferTrait for Buffer<T> {
	type Source = T;
	fn get_slice(&self) -> wgpu::BufferSlice {
		self.buffer.slice(..)
	}

	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a> {
		self.buffer.as_entire_binding()
	}

	fn get_buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}

	fn get_size(&self) -> u64 {
		self.buffer.size()
	}

	fn get_offset(&self, offset: u64) -> u64 {
		offset
	}
}

pub struct RcBuffer<T: HostShareable> {
	buffer: std::rc::Rc<wgpu::Buffer>,
	range: core::ops::Range<u64>,
	phantom_data: PhantomData<T>,
}

pub struct ArcBuffer<T: HostShareable> {
	buffer: std::sync::Arc<wgpu::Buffer>,
	range: core::ops::Range<u64>,
	phantom_data: PhantomData<T>,
}
