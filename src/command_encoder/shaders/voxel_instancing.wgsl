const WIDTH = 50.0;

struct Vertex {
    position: vec3<f32>,
    color: vec3<f32>,
};

@group(0) @binding(0)
var voxels_color: texture_3d<f32>;

@group(0) @binding(1)
var<storage, read_write> vertices: array<array<Vertex, 8>>;
@group(0) @binding(2)
var<storage, read_write> indices: array<array<u32, 36>>;

@group(0) @binding(3)
var<storage, read_write> voxel_sum: atomic<i32>;

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) workgroup_id : vec3<u32>,) {
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

        var index: i32 = atomicAdd(&voxel_sum, 1);

        var grid_size: vec3<f32> = vec3(
            f32(textureDimensions(voxels_color).x), 
            f32(textureDimensions(voxels_color).y), 
            f32(textureDimensions(voxels_color).z)
        );
        var voxel_size: f32 = (WIDTH / grid_size.r);
        var half_voxel_size: f32 = voxel_size / 2.0;
        var voxel_position: vec3<f32> = vec3(f32(workgroup_id.r), f32(workgroup_id.g), f32(workgroup_id.b));
        var center_position: vec3<f32> = vec3(
            (-WIDTH/2.0) + (voxel_position.r * voxel_size),
            (-WIDTH/2.0) + (voxel_position.g * voxel_size),
            (-WIDTH/2.0) + (voxel_position.b * voxel_size),
        );


        vertices[index][0].position = vec3(center_position.x + half_voxel_size, center_position.y + half_voxel_size, center_position.z + half_voxel_size);
        vertices[index][1].position = vec3(center_position.x + half_voxel_size, center_position.y + half_voxel_size, center_position.z - half_voxel_size);
        vertices[index][2].position = vec3(center_position.x + half_voxel_size, center_position.y - half_voxel_size, center_position.z + half_voxel_size);
        vertices[index][3].position = vec3(center_position.x + half_voxel_size, center_position.y - half_voxel_size, center_position.z - half_voxel_size);
        vertices[index][4].position = vec3(center_position.x - half_voxel_size, center_position.y + half_voxel_size, center_position.z + half_voxel_size);
        vertices[index][5].position = vec3(center_position.x - half_voxel_size, center_position.y + half_voxel_size, center_position.z - half_voxel_size);
        vertices[index][6].position = vec3(center_position.x - half_voxel_size, center_position.y - half_voxel_size, center_position.z + half_voxel_size);
        vertices[index][7].position = vec3(center_position.x - half_voxel_size, center_position.y - half_voxel_size, center_position.z - half_voxel_size);

        {
            var color: vec3<f32> = textureLoad(voxels_color, workgroup_id, 0).rgb;


            vertices[index][0].color = color;
            vertices[index][1].color = color;
            vertices[index][2].color = color;
            vertices[index][3].color = color;
            vertices[index][4].color = color;
            vertices[index][5].color = color;
            vertices[index][6].color = color;
            vertices[index][7].color = color;
        }

        var index_offset: u32 = u32(index) * 8u;

        // Right Side
        indices[index][0] = index_offset + 0u;
        indices[index][1] = index_offset + 1u;
        indices[index][2] = index_offset + 3u;
        indices[index][3] = index_offset + 0u;
        indices[index][4] = index_offset + 3u;
        indices[index][5] = index_offset + 2u;

        // Top Side
        indices[index][6] = index_offset + 0u;
        indices[index][7] = index_offset + 4u;
        indices[index][8] = index_offset + 1u;

        indices[index][9] = index_offset + 4u;
        indices[index][10] = index_offset + 5u;
        indices[index][11] = index_offset + 1u;

        // Front Side
        indices[index][12] = index_offset + 1u;
        indices[index][13] = index_offset + 7u;
        indices[index][14] = index_offset + 3u;

        indices[index][15] = index_offset + 1u;
        indices[index][16] = index_offset + 5u;
        indices[index][17] = index_offset + 7u;

        // Left Side
        indices[index][18] = index_offset + 4u;
        indices[index][19] = index_offset + 5u;
        indices[index][20] = index_offset + 7u;

        indices[index][21] = index_offset + 4u;
        indices[index][22] = index_offset + 7u;
        indices[index][23] = index_offset + 6u;

        // Back Side
        indices[index][24] = index_offset + 0u;
        indices[index][25] = index_offset + 6u;
        indices[index][26] = index_offset + 2u;

        indices[index][27] = index_offset + 0u;
        indices[index][28] = index_offset + 4u;
        indices[index][29] = index_offset + 6u;

        // Bottom Side
        indices[index][30] = index_offset + 2u;
        indices[index][31] = index_offset + 6u;
        indices[index][32] = index_offset + 3u;

        indices[index][33] = index_offset + 6u;
        indices[index][34] = index_offset + 7u;
        indices[index][35] = index_offset + 3u;
    }
}
