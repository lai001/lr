// @group(1)
// @binding(0)
// var<storage, read_write> query: array<vec4<u32>>;

@group(0)
@binding(0)
var feed_back_texture: texture_storage_2d<rgba16uint, read>;

@group(0)
@binding(1)
var page_table: texture_storage_2d<rgba8uint, write>;

@compute
@workgroup_size(16, 16, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // var length = arrayLength(&query);

    var data = textureLoad(feed_back_texture, global_id.xy);
    if data.w == u32(1) {
        let x = data.x;
        let y = data.y;
        let mipmap_level = data.z;
        textureStore(page_table, data.xy, vec4<u32>(x, y, mipmap_level, u32(0)));
    }
}