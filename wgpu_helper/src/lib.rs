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
	unsafe fn as_bytes(&self) -> &[u8] {
		::core::slice::from_raw_parts(
			(self as *const Self) as *const u8,
			::core::mem::size_of::<Self>(),
		)
	}
}

pub trait BindGroupItem: Sized {
	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a>;
}

pub trait BufferTrait: Sized + BindGroupItem {
	type Source: HostShareable;

	fn get_slice(&self) -> wgpu::BufferSlice;

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

	fn write_to(&mut self, queue: &wgpu::Queue, new_data: &Self::Source) {
		queue.write_buffer(self.get_buffer(), 0, unsafe { new_data.as_bytes() })
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

impl<T: HostShareable> BindGroupItem for Buffer<T> {
	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a> {
		self.buffer.as_entire_binding()
	}
}

impl<T: HostShareable> BufferTrait for Buffer<T> {
	type Source = T;
	fn get_slice(&self) -> wgpu::BufferSlice {
		self.buffer.slice(..)
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

pub struct ArcBuffer<T: HostShareable> {
	buffer: std::sync::Arc<wgpu::Buffer>,
	range: core::ops::Range<u64>,
	phantom_data: PhantomData<T>,
}

impl<T: HostShareable> ArcBuffer<T> {
	pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, mapped_at_creation: bool) -> Self {
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: core::mem::size_of::<T>() as u64,
			usage,
			mapped_at_creation,
		});

		let range = 0..buffer.size();

		Self {
			buffer: buffer.into(),
			range,
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

		let range = 0..buffer.size();

		Self {
			buffer: buffer.into(),
			range,
			phantom_data: PhantomData::default(),
		}
	}
}

impl<T: HostShareable> BindGroupItem for ArcBuffer<T> {
	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a> {
		wgpu::BindingResource::Buffer(wgpu::BufferBinding {
			buffer: self.get_buffer(),
			offset: self.range.start,
			size: core::num::NonZeroU64::new(
				self.range.end - self.range.start
			),
		})
	}
}

impl<T: HostShareable> BufferTrait for ArcBuffer<T> {
	type Source = T;
	fn get_slice(&self) -> wgpu::BufferSlice {
		self.buffer.slice(self.range.to_owned())
	}

	fn get_buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}

	fn get_size(&self) -> u64 {
		self.range.end - self.range.start
	}

	fn get_offset(&self, offset: u64) -> u64 {
		offset + self.range.start
	}
}

impl BindGroupItem for wgpu::Sampler {
	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a> {
		wgpu::BindingResource::Sampler(&self)
	}
}

impl BindGroupItem for wgpu::TextureView {
	fn get_binding<'a>(&'a self) -> wgpu::BindingResource<'a> {
		wgpu::BindingResource::TextureView(&self)
	}
}