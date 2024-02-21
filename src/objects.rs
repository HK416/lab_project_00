use std::mem;
use crate::interfaces::{
    GameObject, 
    ShaderResource, 
};

/// #### 한국어 </br>
/// 쉐이더에 전달되는 색상된 오브젝트의 유니폼 데이터 레이아웃 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is the uniform data layout of the colored object passed to the shader. </br>
/// 
#[repr(C, align(16))]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ColoredObjectUniformLayout {
    world_matrix: glam::Mat4, 
    color: glam::Vec4, 
}

/// #### 한국어 </br>
/// 색상 오브젝트를 생성하는 빌더 입니다. </br>
/// 
/// #### English (Translation) </br>
/// A builder that creates a colored object. </br>
/// 
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ColordObjectBuilder {
    pub translation: glam::Vec3, 
    pub rotation: glam::Quat, 
    pub scale: glam::Vec3, 
    pub color: glam::Vec4, 
}

#[allow(dead_code)]
impl ColordObjectBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn set_color(mut self, color: glam::Vec4) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub fn set_scale(mut self, scale: glam::Vec3) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub fn set_translation(mut self, translation: glam::Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub fn translate_local(self, distance: glam::Vec3) -> Self {
        let mat = glam::Mat3::from_quat(self.rotation);
        let right = mat.x_axis.normalize_or_zero() * distance.x;
        let up = mat.y_axis.normalize_or_zero() * distance.y;
        let look = mat.z_axis.normalize_or_zero() * distance.z;
        self.translate_world(right + up + look)
    }

    #[inline]
    pub fn translate_world(mut self, distance: glam::Vec3) -> Self {
        self.translation += distance;
        self
    }

    #[inline]
    pub fn set_rotation(mut self, rotation: glam::Quat) -> Self {
        self.rotation = rotation.normalize();
        self
    }
    
    pub fn look_at_point(mut self, point: glam::Vec3) -> Self {
        let mat = glam::Mat3::from_quat(self.rotation);
        let up = mat.y_axis.normalize_or_zero();
        let look = (point - self.translation).normalize_or_zero();
        let right = up.cross(look).normalize_or_zero();
        let up = look.cross(right).normalize_or_zero();
        self.rotation = glam::Quat::from_mat3(&glam::Mat3::from_cols(right, up, look));
        self
    }

    #[inline]
    pub fn rotate(mut self, rotation: glam::Quat) -> Self {
        self.rotation *= rotation.normalize();
        self
    }

    pub fn build(self, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> ColoredObject {
        let buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("UniformBuffer(ColoredObject)"), 
                mapped_at_creation: false, 
                size: mem::size_of::<ColoredObjectUniformLayout>() as wgpu::BufferAddress, 
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("BindGroup(ColoredObject)"), 
                layout: &bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0, 
                        resource: wgpu::BindingResource::Buffer(
                            buffer.as_entire_buffer_binding()
                        ),
                    },
                ],
            },
        );

        ColoredObject { 
            color: self.color, 
            transform: glam::Mat4::from_scale_rotation_translation(
                self.scale, 
                self.rotation.normalize(), 
                self.translation
            ), 
            buffer, 
            bind_group, 
        }
    }
}

/// #### 한국어 </br>
/// 게임 월드 좌표계에 존재하는 색상 오브젝트 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is a colored object that exists in the game world coordinate system. </br>
/// 
#[derive(Debug)]
pub struct ColoredObject {
    color: glam::Vec4, 
    transform: glam::Mat4, 
    buffer: wgpu::Buffer, 
    bind_group: wgpu::BindGroup, 
}

impl GameObject for ColoredObject {
    #[inline]
    fn ref_world_transform(&self) -> &glam::Mat4 {
        &self.transform
    }

    #[inline]
    fn mut_world_transform(&mut self) -> &mut glam::Mat4 {
        &mut self.transform
    }
}

impl ShaderResource for ColoredObject {
    fn update_shader_resource(&self, queue: &wgpu::Queue) {
        let data = ColoredObjectUniformLayout {
            world_matrix: self.transform, 
            color: self.color, 
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&data));
    }

    #[inline]
    fn ref_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
