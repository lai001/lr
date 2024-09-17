use russimp_sys::{aiColor3D, aiColor4D, aiMatrix4x4, aiString, aiVector3D};

use crate::AISTRING_MAXLEN;

pub(crate) trait ConvertToMat4 {
    fn to_mat4(&self) -> glam::Mat4;
}

pub(crate) trait ConvertToString {
    fn to_string(&self) -> String;
}

pub(crate) trait ConvertToAIString {
    fn to_ai_string(&self) -> aiString;
}

pub(crate) trait ConvertToVec4 {
    fn to_vec4(&self) -> glam::Vec4;
}

pub(crate) trait ConvertToVec3 {
    fn to_vec3(&self) -> glam::Vec3;
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

impl ConvertToMat4 for aiMatrix4x4 {
    fn to_mat4(&self) -> glam::Mat4 {
        glam::mat4(
            glam::vec4(self.a1, self.b1, self.c1, self.d1),
            glam::vec4(self.a2, self.b2, self.c2, self.d2),
            glam::vec4(self.a3, self.b3, self.c3, self.d3),
            glam::vec4(self.a4, self.b4, self.c4, self.d4),
        )
    }
}

impl ConvertToString for aiString {
    fn to_string(&self) -> String {
        self.into()
    }
}

impl ConvertToAIString for String {
    fn to_ai_string(&self) -> aiString {
        let len = self.as_bytes().len();
        let len = AISTRING_MAXLEN.min(len);
        let mut ai_string = aiString {
            length: len as _,
            data: [0; AISTRING_MAXLEN],
        };
        unsafe {
            let raw = std::slice::from_raw_parts(
                std::mem::transmute::<*const u8, *const i8>(self.as_ptr()),
                len,
            );
            ai_string.data.copy_from_slice(raw);
        };
        ai_string
    }
}

impl ConvertToAIString for &str {
    fn to_ai_string(&self) -> aiString {
        let len = self.as_bytes().len();
        let len = AISTRING_MAXLEN.min(len);
        let mut ai_string = aiString {
            length: len as _,
            data: [0; AISTRING_MAXLEN],
        };
        unsafe {
            let raw = std::slice::from_raw_parts(
                std::mem::transmute::<*const u8, *const i8>(self.as_ptr()),
                len,
            );
            ai_string.data.copy_from_slice(raw);
        };
        ai_string
    }
}
