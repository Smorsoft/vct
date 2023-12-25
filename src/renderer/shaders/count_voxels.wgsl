@group(0) @binding(0)
var voxels_texture: texture_3d<u32>;

@group(0) @binding(1)
var<storage, read_write> voxel_sum: atomic<i32>;

@compute @workgroup_size(1)
fn get_voxel_sum(
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
) {
    if (unpack4x8unorm(textureLoad(voxels_texture, workgroup_id, 0).r).w > 0.0) {
        atomicAdd(&voxel_sum, 1);
    }
}