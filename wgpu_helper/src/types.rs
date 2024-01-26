macro_rules! new_host_shareable {
	($type:ty, $wgsl_name:literal, $buffer_name:ident, $flags:expr) => {
		impl crate::ToWGSL for $type {
			fn to_wgsl() -> crate::WGSL {
				crate::WGSL::Ty($wgsl_name.into())
			}
		}

		impl crate::HostShareable for $type {}
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
