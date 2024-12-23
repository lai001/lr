#ifndef LIGHT_CULLING_WGSL
#define LIGHT_CULLING_WGSL

#include "camera_frustum.wgsl"
#include "cluster_light.wgsl"

struct Sphere3D {
    center: vec3<f32>,
    radius: f32,
}

struct Plane3D {
    normal_vector: vec3<f32>,
    point: vec3<f32>,
}

struct FrustumPlanes {
    left_plane: Plane3D,
    right_plane: Plane3D,
    top_plane: Plane3D,
    bottom_plane: Plane3D,
    front_plane: Plane3D,
    back_plane: Plane3D,
}

fn frustum_planes_new(frustum: Frustum) -> FrustumPlanes {
    var frustum_planes: FrustumPlanes;
    var left_plane = Plane3D(
        normalize(cross(frustum.far_2 - frustum.near_2, frustum.near_3 - frustum.near_2)),
        (frustum.far_2 + frustum.far_3 + frustum.near_2 + frustum.near_3) / 4.0
    );
    var right_plane = Plane3D(
        normalize(cross(frustum.near_0 - frustum.near_1, frustum.far_1 - frustum.near_1)),
        (frustum.far_0 + frustum.far_1 + frustum.near_0 + frustum.near_1) / 4.0
    );
    var top_plane = Plane3D(
        normalize(cross(frustum.near_3 - frustum.near_0, frustum.far_0 - frustum.near_0)),
        (frustum.far_0 + frustum.far_3 + frustum.near_0 + frustum.near_3) / 4.0
    );
    var bottom_plane = Plane3D(
        normalize(cross(frustum.far_1 - frustum.near_1, frustum.near_2 - frustum.near_1)),
        (frustum.far_1 + frustum.far_2 + frustum.near_1 + frustum.near_2) / 4.0
    );
    var front_plane = Plane3D(
        normalize(cross(frustum.near_2 - frustum.near_1, frustum.near_0 - frustum.near_1)),
        (frustum.near_0 + frustum.near_1 + frustum.near_2 + frustum.near_3) / 4.0
    );
    var back_plane = Plane3D(
        normalize(cross(frustum.far_0 - frustum.far_1, frustum.far_2 - frustum.far_1)),
        (frustum.far_0 + frustum.far_1 + frustum.far_2 + frustum.far_3) / 4.0
    );
    return FrustumPlanes (
        left_plane,
        right_plane,
        top_plane,
        bottom_plane,
        front_plane,
        back_plane
    );
}

fn plane_signed_distance_to_point(plane: Plane3D, target_point: vec3<f32>) -> f32 {
    var x = vec4<f32>(
        plane.normal_vector.x,
        plane.normal_vector.y,
        plane.normal_vector.z,
        -dot(plane.normal_vector, plane.point)
    );
    var y = vec4<f32>(target_point.x, target_point.y, target_point.z, 1.0);
    return dot(x, y);
}

fn plane_is_inside(plane: Plane3D, sphere3d: Sphere3D) -> bool {
    let signed_distance = plane_signed_distance_to_point(plane, sphere3d.center);
    if (signed_distance > sphere3d.radius) {
        return false;
    } else {
        return true;
    }
}

fn is_sphere_visible_to_frustum(sphere3d: Sphere3D, frustum: Frustum) -> bool {
    var frustum_planes = frustum_planes_new(frustum);
    return plane_is_inside(frustum_planes.left_plane, sphere3d)
        && plane_is_inside(frustum_planes.right_plane, sphere3d)
        && plane_is_inside(frustum_planes.top_plane, sphere3d)
        && plane_is_inside(frustum_planes.bottom_plane, sphere3d)
        && plane_is_inside(frustum_planes.front_plane, sphere3d)
        && plane_is_inside(frustum_planes.back_plane, sphere3d);
}

fn get_sub_frustum_by_id(global_id: vec3<u32>, frustum: Frustum) -> Frustum {
    var point3_near = frustum.near_3 + (frustum.far_3 - frustum.near_3) * (f32(global_id.z) / 10.0);
    var point3_far = frustum.near_3 + (frustum.far_3 - frustum.near_3) * (f32(global_id.z + 1) / 10.0);

    var point0_near = frustum.near_0 + (frustum.far_0 - frustum.near_0) * (f32(global_id.z) / 10.0);
    var point0_far = frustum.near_0 + (frustum.far_0 - frustum.near_0) * (f32(global_id.z + 1) / 10.0);

    var point2_near = frustum.near_2 + (frustum.far_2 - frustum.near_2) * (f32(global_id.z) / 10.0);
    var point2_far = frustum.near_2 + (frustum.far_2 - frustum.near_2) * (f32(global_id.z + 1) / 10.0);

    var step_horizontal_near = (point0_near - point3_near) / 10.0;
    var step_vertical_near = (point2_near - point3_near) / 10.0;

    var step_horizontal_far = (point0_far - point3_far) / 10.0;
    var step_vertical_far = (point2_far - point3_far) / 10.0;

    var near_3 = point3_near + step_horizontal_near * f32(global_id.x) + step_vertical_near * f32(global_id.y);
    var near_0 = near_3 + step_horizontal_near;
    var near_2 = near_3 + step_vertical_near;
    var near_1 = near_0 + step_vertical_near;

    var far_3 = point3_far + step_horizontal_far * f32(global_id.x) + step_vertical_far * f32(global_id.y);
    var far_0 = far_3 + step_horizontal_far;
    var far_2 = far_3 + step_vertical_far;
    var far_1 = far_0 + step_vertical_far;

    return Frustum(
        near_0,
        near_1,
        near_2,
        near_3,
        far_0,
        far_1,
        far_2,
        far_3
    );
}

@group(0) @binding(0) var<storage, read> point_light_shapes: array<Sphere3D>;
@group(0) @binding(1) var<uniform> camera_frustum: Frustum;
@group(0) @binding(2) var<storage, read_write> cluster_lights: array<u32>;
@group(0) @binding(3) var<storage, read_write> cluster_light_indices: array<ClusterLightIndex>;

@compute
@workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var index = global_id.z * 100 + global_id.y * 10 + global_id.x;
    var frustum = get_sub_frustum_by_id(global_id, camera_frustum);
    var length_of_point_light_shapes: u32 = arrayLength(&point_light_shapes);
    var cluster_light_index: ClusterLightIndex = ClusterLightIndex(0, 0);
    cluster_light_index.offset = index * length_of_point_light_shapes;
    var write_index = cluster_light_index.offset;
    for (var i = 0u; i < length_of_point_light_shapes ; i++) {
        var sphere3d: Sphere3D = point_light_shapes[i];
        var is_visible = is_sphere_visible_to_frustum(sphere3d, frustum);
        if (is_visible) {
            cluster_light_index.count = cluster_light_index.count + 1;
            cluster_lights[write_index] = i;
            write_index = write_index + 1;
        }
    }
    cluster_light_indices[index] = cluster_light_index;
}

#endif