class AccelerationBaker {
    constructor();
    /**
     * @returns {string}
     */
    toString();
}

class BakeInfo {
    /**
     * 
     * @param {boolean} is_bake_environment 
     * @param {boolean} is_bake_irradiance 
     * @param {boolean} is_bake_brdflut 
     * @param {boolean} is_bake_pre_filter 
     * @param {number} environment_cube_map_length 
     * @param {number} irradiance_cube_map_length 
     * @param {number} irradiance_sample_count 
     * @param {number} pre_filter_cube_map_length 
     * @param {number} pre_filter_cube_map_max_mipmap_level 
     * @param {number} pre_filter_sample_count 
     * @param {number} brdflutmap_length 
     * @param {number} brdf_sample_count 
     * @param {boolean} is_read_back 
     */
    constructor(
        is_bake_environment,
        is_bake_irradiance,
        is_bake_brdflut,
        is_bake_pre_filter,
        environment_cube_map_length,
        irradiance_cube_map_length,
        irradiance_sample_count,
        pre_filter_cube_map_length,
        pre_filter_cube_map_max_mipmap_level,
        pre_filter_sample_count,
        brdflutmap_length,
        brdf_sample_count,
        is_read_back,
    );
    /**
     * @returns {string}
     */
    toString();
}