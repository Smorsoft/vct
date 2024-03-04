const WIDTH = 50.0;

@group(0) @binding(0)
var source: texture_3d<f32>;

@group(0) @binding(1)
var output: texture_storage_3d<rgba8unorm, write>;


@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id : vec3<u32>,) {
	let offset = vec2<u32>(0u, 1u);
	
	let value = (
		textureLoad(source, 2u * id.xyz + offset.xxx, 0) + 
		textureLoad(source, 2u * id.xyz + offset.xyx, 0) + 
		textureLoad(source, 2u * id.xyz + offset.xyy, 0) + 
		textureLoad(source, 2u * id.xyz + offset.xxy, 0) + 
		textureLoad(source, 2u * id.xyz + offset.yxx, 0) + 
		textureLoad(source, 2u * id.xyz + offset.yyx, 0) + 
		textureLoad(source, 2u * id.xyz + offset.yxy, 0) + 
		textureLoad(source, 2u * id.xyz + offset.yyy, 0) 
	) * 0.125;

	textureStore(output, id, value);
}
