#ifndef COMMON_SLANG
#define COMMON_SLANG

#define GROUP_BINDING(X) [[vk::binding(X ## _BINDING, X ## _GROUP)]]

float3x3 mat4x4_to_mat3x3(float4x4 m)
{
    return float3x3(m[0].xyz, m[1].xyz, m[2].xyz);
}

float check_uv(float2 uv)
{
    if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0)
        return 1.0;
    else
        return 0.0;
}

float4x4 inverse(float4x4 m)
{
    float n11 = m[0][0]; float n12 = m[1][0]; float n13 = m[2][0]; float n14 = m[3][0];
    float n21 = m[0][1]; float n22 = m[1][1]; float n23 = m[2][1]; float n24 = m[3][1];
    float n31 = m[0][2]; float n32 = m[1][2]; float n33 = m[2][2]; float n34 = m[3][2];
    float n41 = m[0][3]; float n42 = m[1][3]; float n43 = m[2][3]; float n44 = m[3][3];

    float t11 = n23 * n34 * n42 - n24 * n33 * n42 + n24 * n32 * n43 - n22 * n34 * n43 - n23 * n32 * n44 + n22 * n33 * n44;
    float t12 = n14 * n33 * n42 - n13 * n34 * n42 - n14 * n32 * n43 + n12 * n34 * n43 + n13 * n32 * n44 - n12 * n33 * n44;
    float t13 = n13 * n24 * n42 - n14 * n23 * n42 + n14 * n22 * n43 - n12 * n24 * n43 - n13 * n22 * n44 + n12 * n23 * n44;
    float t14 = n14 * n23 * n32 - n13 * n24 * n32 - n14 * n22 * n33 + n12 * n24 * n33 + n13 * n22 * n34 - n12 * n23 * n34;

    float det = n11 * t11 + n21 * t12 + n31 * t13 + n41 * t14;
    float idet = 1.0 / det;

    float4x4 ret;
    ret[0][0] = t11 * idet;
    ret[0][1] = (n24 * n33 * n41 - n23 * n34 * n41 - n24 * n31 * n43 + n21 * n34 * n43 + n23 * n31 * n44 - n21 * n33 * n44) * idet;
    ret[0][2] = (n22 * n34 * n41 - n24 * n32 * n41 + n24 * n31 * n42 - n21 * n34 * n42 - n22 * n31 * n44 + n21 * n32 * n44) * idet;
    ret[0][3] = (n23 * n32 * n41 - n22 * n33 * n41 - n23 * n31 * n42 + n21 * n33 * n42 + n22 * n31 * n43 - n21 * n32 * n43) * idet;

    ret[1][0] = t12 * idet;
    ret[1][1] = (n13 * n34 * n41 - n14 * n33 * n41 + n14 * n31 * n43 - n11 * n34 * n43 - n13 * n31 * n44 + n11 * n33 * n44) * idet;
    ret[1][2] = (n14 * n32 * n41 - n12 * n34 * n41 - n14 * n31 * n42 + n11 * n34 * n42 + n12 * n31 * n44 - n11 * n32 * n44) * idet;
    ret[1][3] = (n12 * n33 * n41 - n13 * n32 * n41 + n13 * n31 * n42 - n11 * n33 * n42 - n12 * n31 * n43 + n11 * n32 * n43) * idet;

    ret[2][0] = t13 * idet;
    ret[2][1] = (n14 * n23 * n41 - n13 * n24 * n41 - n14 * n21 * n43 + n11 * n24 * n43 + n13 * n21 * n44 - n11 * n23 * n44) * idet;
    ret[2][2] = (n12 * n24 * n41 - n14 * n22 * n41 + n14 * n21 * n42 - n11 * n24 * n42 - n12 * n21 * n44 + n11 * n22 * n44) * idet;
    ret[2][3] = (n13 * n22 * n41 - n12 * n23 * n41 - n13 * n21 * n42 + n11 * n23 * n42 + n12 * n21 * n43 - n11 * n22 * n43) * idet;

    ret[3][0] = t14 * idet;
    ret[3][1] = (n13 * n24 * n31 - n14 * n23 * n31 + n14 * n21 * n33 - n11 * n24 * n33 - n13 * n21 * n34 + n11 * n23 * n34) * idet;
    ret[3][2] = (n14 * n22 * n31 - n12 * n24 * n31 - n14 * n21 * n32 + n11 * n24 * n32 + n12 * n21 * n34 - n11 * n22 * n34) * idet;
    ret[3][3] = (n12 * n23 * n31 - n13 * n22 * n31 + n13 * n21 * n32 - n11 * n23 * n32 - n12 * n21 * n33 + n11 * n22 * n33) * idet;
    return ret;
}

float mipmap_level(float2 uv, float2 texture_size)
{
    float2 s = ddx(uv) * texture_size;
    float2 t = ddy(uv) * texture_size;
    float delta = max(dot(s, s), dot(t, t));
    return 0.5 * log2(delta);
}

uint get_mip_level_size(uint length, uint level)
{
    return max(uint(1), length >> level);
}

float2 mipmap_size(float2 lod0_size, uint level)
{
    float x = float(get_mip_level_size(uint(lod0_size.x), level));
    float y = float(get_mip_level_size(uint(lod0_size.y), level));
    return float2(x, y);
}

float4x4 make_translation_matrix(float x, float y, float z)
{
    return float4x4(
        float4(1.0, 0.0, 0.0, 0.0),
        float4(0.0, 1.0, 0.0, 0.0),
        float4(0.0, 0.0, 1.0, 0.0),
        float4(x, y, z, 1.0));
}

float4x4 make_translation_matrix_from_vec3(float3 v)
{
    return float4x4(
        float4(1.0, 0.0, 0.0, 0.0),
        float4(0.0, 1.0, 0.0, 0.0),
        float4(0.0, 0.0, 1.0, 0.0),
        float4(v.x, v.y, v.z, 1.0));
}

float4x4 make_identity_matrix()
{
    return float4x4(
        float4(1.0, 0.0, 0.0, 0.0),
        float4(0.0, 1.0, 0.0, 0.0),
        float4(0.0, 0.0, 1.0, 0.0),
        float4(0.0, 0.0, 0.0, 1.0));
}

float4x4 strip_matrix_location(float4x4 mat)
{
    return float4x4(
        float4(mat[0][0], mat[1][0], mat[2][0], 0.0),
        float4(mat[0][1], mat[1][1], mat[2][1], 0.0),
        float4(mat[0][2], mat[1][2], mat[2][2], 0.0),
        float4(0.0, 0.0, 0.0, 1.0));
}

float4x4 strip_matrix_location_ant(float4x4 mat)
{
    return float4x4(
        float4(-mat[0][0], -mat[1][0], -mat[2][0], 0.0),
        float4(-mat[0][1], -mat[1][1], -mat[2][1], 0.0),
        float4(-mat[0][2], -mat[1][2], -mat[2][2], 0.0),
        float4(0.0, 0.0, 0.0, 1.0));
}

#endif
