const WIDTH = 100.0;

@group(0) @binding(0)
var voxels_color: texture_storage_3d<rgba8unorm, write>;
// @group(0) @binding(1)
// var voxels_normal: texture_storage_3d<rgba8snorm, write>;
// @group(0) @binding(2)
// var voxels_emissive: texture_storage_3d<rgba8unorm, write>;

// Data per side
// Color RGBA8Unorm - alpha translucency
// Normal RGBA8Snorm - alpha metallic (reinterperet as unorm)
// Emissive RGBA8Unorm - alpha roughness



@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) workgroup_id : vec3<u32>,) {
	
}
