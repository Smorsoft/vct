#![allow(non_camel_case_types)]

use super::new_host_shareable;

new_host_shareable!(i32, "i32", i32Buffer);
new_host_shareable!(u32, "u32", u32Buffer);
new_host_shareable!(f32, "f32", f32Buffer);