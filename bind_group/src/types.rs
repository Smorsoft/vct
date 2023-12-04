macro_rules! new_host_shareable {
	($type:ty, $wgsl_name:literal, $buffer_name:ident, $flags:expr) => {
		#[repr(transparent)]
		pub struct $buffer_name(::wgpu::Buffer);

		impl crate::Buffer for $buffer_name {
			type Source = $type;

			unsafe fn from_buffer(buffer: ::wgpu::Buffer) -> Self {
				::core::mem::transmute(buffer)
			}

			unsafe fn as_buffer(&self) -> &::wgpu::Buffer {
				::core::mem::transmute(self)
			}
		}

		impl crate::ToWGSL for $type {
			fn to_wgsl() -> crate::WGSL {
				crate::WGSL::Ty($wgsl_name.into())
			}
		}

		impl crate::HostShareable for $type {
			const REQUIRED_BUFFER_USAGE_FLAGS: ::wgpu::BufferUsages = $flags;
			type Buffer = $buffer_name;
		}
	};
	($type:ty, $wgsl_name:literal, $buffer_name:ident) => {
		new_host_shareable!(
			$type,
			$wgsl_name,
			$buffer_name,
			::wgpu::BufferUsages::empty()
		);
	};
}

pub(crate) use new_host_shareable;

mod vector;
pub use vector::*;

mod matrix;
pub use matrix::*;
