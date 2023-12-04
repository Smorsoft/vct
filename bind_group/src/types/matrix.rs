#![allow(non_camel_case_types)]

use super::new_host_shareable;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat2x2<T>([[T; 2]; 2]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat2x3<T>([[T; 2]; 3]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat2x4<T>([[T; 2]; 4]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat3x2<T>([[T; 3]; 2]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat3x3<T>([[T; 3]; 3]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat3x4<T>([[T; 3]; 4]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat4x2<T>([[T; 4]; 2]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat4x3<T>([[T; 4]; 3]);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct mat4x4<T>([[T; 4]; 4]);

pub type mat2x2f = mat2x2<f32>;
pub type mat2x3f = mat2x3<f32>;
pub type mat2x4f = mat2x4<f32>;
pub type mat3x2f = mat3x2<f32>;
pub type mat3x3f = mat3x3<f32>;
pub type mat3x4f = mat3x4<f32>;
pub type mat4x2f = mat4x2<f32>;
pub type mat4x3f = mat4x3<f32>;
pub type mat4x4f = mat4x4<f32>;

new_host_shareable!(mat2x2<f32>, "mat2x2<f32>", mat2x2fBuffer);
new_host_shareable!(mat2x3<f32>, "mat2x3<f32>", mat2x3fBuffer);
new_host_shareable!(mat2x4<f32>, "mat2x4<f32>", mat2x4fBuffer);
new_host_shareable!(mat3x2<f32>, "mat3x2<f32>", mat3x2fBuffer);
new_host_shareable!(mat3x3<f32>, "mat3x3<f32>", mat3x3fBuffer);
new_host_shareable!(mat3x4<f32>, "mat3x4<f32>", mat3x4fBuffer);
new_host_shareable!(mat4x2<f32>, "mat4x2<f32>", mat4x2fBuffer);
new_host_shareable!(mat4x3<f32>, "mat4x3<f32>", mat4x3fBuffer);
new_host_shareable!(mat4x4<f32>, "mat4x4<f32>", mat4x4fBuffer);



impl ::core::ops::Deref for mat2x2<f32> {
	type Target = [[f32; 2]; 2];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat2x3<f32> {
	type Target = [[f32; 2]; 3];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat2x4<f32> {
	type Target = [[f32; 2]; 4];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat3x2<f32> {
	type Target = [[f32; 3]; 2];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat3x3<f32> {
	type Target = [[f32; 3]; 3];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat3x4<f32> {
	type Target = [[f32; 3]; 4];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat4x2<f32> {
	type Target = [[f32; 4]; 2];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat4x3<f32> {
	type Target = [[f32; 4]; 3];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ::core::ops::Deref for mat4x4<f32> {
	type Target = [[f32; 4]; 4];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ::core::convert::From<[[f32; 2]; 2]> for mat2x2<f32> {
	fn from(value: [[f32; 2]; 2]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 2]; 3]> for mat2x3<f32> {
	fn from(value: [[f32; 2]; 3]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 2]; 4]> for mat2x4<f32> {
	fn from(value: [[f32; 2]; 4]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 3]; 2]> for mat3x2<f32> {
	fn from(value: [[f32; 3]; 2]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 3]; 3]> for mat3x3<f32> {
	fn from(value: [[f32; 3]; 3]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 3]; 4]> for mat3x4<f32> {
	fn from(value: [[f32; 3]; 4]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 4]; 2]> for mat4x2<f32> {
	fn from(value: [[f32; 4]; 2]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 4]; 3]> for mat4x3<f32> {
	fn from(value: [[f32; 4]; 3]) -> Self {
		Self(value)
	}
}
impl ::core::convert::From<[[f32; 4]; 4]> for mat4x4<f32> {
	fn from(value: [[f32; 4]; 4]) -> Self {
		Self(value)
	}
}