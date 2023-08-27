struct Element {
    virtual_index_x: i32,
    virtual_index_y: i32,
    physical_offset_x: i32,
    physical_offset_y: i32,
    physical_array_index: i32,
    virtual_mimap: i32,
};

@group(1)
@binding(0)
var<storage, read> query: array<Element>;

@group(0)
@binding(0)
var page_table: texture_storage_2d_array<rgba16uint, write>;

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var length = arrayLength(&query);
    for (var i: u32 = u32(0); i < length ; i++) {
        var element = query[i];
        var value: vec4<u32> = vec4<u32>(u32(element.physical_offset_x), u32(element.physical_offset_y), u32(element.physical_array_index), u32(1));
        textureStore(page_table, vec2<i32>(element.virtual_index_x, element.virtual_index_y), element.virtual_mimap, value);
    }
}