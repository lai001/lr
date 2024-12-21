#ifndef CAMERA_FRUSTUM
#define CAMERA_FRUSTUM

struct Frustum {
    near_0: vec3<f32>,
    near_1: vec3<f32>,
    near_2: vec3<f32>,
    near_3: vec3<f32>,
    far_0: vec3<f32>,
    far_1: vec3<f32>,
    far_2: vec3<f32>,
    far_3: vec3<f32>,
}

#endif