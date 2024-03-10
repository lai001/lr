use russimp_sys::{aiColor3D, aiColor4D, aiMatrix4x4, aiString, aiTexel, aiVector2D, aiVector3D};

pub(crate) trait ConvertToMat4 {
    fn to_mat4(&self) -> glam::Mat4;
}

pub(crate) trait ConvertToString {
    fn to_string(&self) -> String;
}

pub(crate) trait ConvertToVec4 {
    fn to_vec4(&self) -> glam::Vec4;
}

pub(crate) trait ConvertToVec3 {
    fn to_vec3(&self) -> glam::Vec3;
}

pub(crate) trait ConvertToVec2 {
    fn to_vec2(&self) -> glam::Vec2;
}

pub(crate) trait ConvertToUVec4 {
    fn to_uvec4(&self) -> glam::UVec4;
}

impl ConvertToVec4 for aiColor4D {
    fn to_vec4(&self) -> glam::Vec4 {
        glam::vec4(self.r, self.g, self.b, self.a)
    }
}

impl ConvertToVec3 for aiVector3D {
    fn to_vec3(&self) -> glam::Vec3 {
        glam::vec3(self.x, self.y, self.z)
    }
}

impl ConvertToVec3 for aiColor3D {
    fn to_vec3(&self) -> glam::Vec3 {
        glam::vec3(self.r, self.g, self.b)
    }
}

impl ConvertToVec2 for aiVector2D {
    fn to_vec2(&self) -> glam::Vec2 {
        glam::vec2(self.x, self.y)
    }
}

impl ConvertToMat4 for aiMatrix4x4 {
    fn to_mat4(&self) -> glam::Mat4 {
        glam::mat4(
            glam::vec4(self.a1, self.a2, self.a3, self.a4),
            glam::vec4(self.b1, self.b2, self.b3, self.b4),
            glam::vec4(self.c1, self.c2, self.c3, self.c4),
            glam::vec4(self.d1, self.d2, self.d3, self.d4),
        )
    }
}

impl ConvertToString for aiString {
    fn to_string(&self) -> String {
        self.into()
    }
}

impl ConvertToUVec4 for aiTexel {
    fn to_uvec4(&self) -> glam::UVec4 {
        glam::uvec4(self.b as _, self.g as _, self.r as _, self.a as _)
    }
}
