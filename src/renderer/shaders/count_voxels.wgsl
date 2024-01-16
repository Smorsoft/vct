@group(0) @binding(0)
var voxels_color: texture_3d<f32>;

@group(0) @binding(1)
var<storage, read_write> voxel_sum: atomic<i32>;

@compute @workgroup_size(1)
fn get_voxel_sum(
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
) {
    if (textureLoad(voxels_color, workgroup_id, 0).a > 0.0) {
        if (textureLoad(voxels_color, vec3(workgroup_id.x - 1u, workgroup_id.y, workgroup_id.z), 0).a > 0.0 &&
            textureLoad(voxels_color, vec3(workgroup_id.x + 1u, workgroup_id.y, workgroup_id.z), 0).a > 0.0 &&
            textureLoad(voxels_color, vec3(workgroup_id.x, workgroup_id.y - 1u, workgroup_id.z), 0).a > 0.0 &&
            textureLoad(voxels_color, vec3(workgroup_id.x, workgroup_id.y + 1u, workgroup_id.z), 0).a > 0.0 &&
            textureLoad(voxels_color, vec3(workgroup_id.x, workgroup_id.y, workgroup_id.z - 1u), 0).a > 0.0 &&
            textureLoad(voxels_color, vec3(workgroup_id.x, workgroup_id.y, workgroup_id.z + 1u), 0).a > 0.0
        ) {
            return;
        }

        atomicAdd(&voxel_sum, 1);
    }
}