pub fn calculate_max_mips(length: u32) -> u32 {
    32 - length.leading_zeros()
    // let mut mipmap_level: u32 = 1;
    // let mut length = length;
    // while length > 4 {
    //     length /= 2;
    //     mipmap_level += 1;
    // }
    // return mipmap_level;
}

pub fn calculate_mipmap_level_sizes(length: u32) -> Vec<u32> {
    let mut sizes = Vec::new();
    let mut length = length;
    while length > 0 {
        sizes.push(length);
        length /= 2;
    }
    sizes
}

pub fn get_mip_level_size(length: u32, level: u32) -> u32 {
    u32::max(1, length >> level)
}
