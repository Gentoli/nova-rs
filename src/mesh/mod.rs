use cgmath::{Vector2, Vector3, Vector4};

pub struct FullVertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tangent: Vector3<f32>,
    pub main_uv: Vector2<f32>,
    pub secondary_uv: Vector2<f32>,
    pub virtual_texture_id: u32,
    pub additional_stuff: Vector4<f32>,
}
