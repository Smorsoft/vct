#![allow(non_camel_case_types)]

use super::new_host_shareable;

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct vec2<T>([T; 2]);

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct vec3<T>([T; 3]);

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct vec4<T>([T; 4]);

pub type vec2i = vec2<i32>;
pub type vec3i = vec3<i32>;
pub type vec4i = vec4<i32>;

pub type vec2u = vec2<u32>;
pub type vec3u = vec3<u32>;
pub type vec4u = vec4<u32>;

pub type vec2f = vec2<f32>;
pub type vec3f = vec3<f32>;
pub type vec4f = vec4<f32>;

new_host_shareable!(vec2<i32>, "vec2<i32>", vec2iBuffer);
new_host_shareable!(vec3<i32>, "vec3<i32>", vec3iBuffer);
new_host_shareable!(vec4<i32>, "vec4<i32>", vec4iBuffer);
new_host_shareable!(vec2<u32>, "vec2<u32>", vec2uBuffer);
new_host_shareable!(vec3<u32>, "vec3<u32>", vec3uBuffer);
new_host_shareable!(vec4<u32>, "vec4<u32>", vec4uBuffer);
new_host_shareable!(vec2<f32>, "vec2<f32>", vec2fBuffer);
new_host_shareable!(vec3<f32>, "vec3<f32>", vec3fBuffer);
new_host_shareable!(vec4<f32>, "vec4<f32>", vec4fBuffer);

impl core::convert::From<[f32; 2]> for vec2<f32> {
	fn from(value: [f32; 2]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[f32; 3]> for vec3<f32> {
	fn from(value: [f32; 3]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[f32; 4]> for vec4<f32> {
	fn from(value: [f32; 4]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[i32; 2]> for vec2<i32> {
	fn from(value: [i32; 2]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[i32; 3]> for vec3<i32> {
	fn from(value: [i32; 3]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[i32; 4]> for vec4<i32> {
	fn from(value: [i32; 4]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[u32; 2]> for vec2<u32> {
	fn from(value: [u32; 2]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[u32; 3]> for vec3<u32> {
	fn from(value: [u32; 3]) -> Self {
		Self(value)
	}
}

impl core::convert::From<[u32; 4]> for vec4<u32> {
	fn from(value: [u32; 4]) -> Self {
		Self(value)
	}
}