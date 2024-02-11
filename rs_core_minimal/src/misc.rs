pub fn calculate_mipmap_level(length: u32) -> u32 {
    let mut mipmap_level: u32 = 1;
    let mut length = length;
    while length > 4 {
        length /= 2;
        mipmap_level += 1;
    }
    return mipmap_level;
}
