use num::Zero;
use nalgebra::{Matrix4, Point2, Point3, Point4, Vector2, Vector3, Vector4};

pub type GpuScalar = f32;

const EPS: f32 = 1e-4;

pub trait ScalarField {
    #[inline]
    fn value_at(&self, x: f32, y: f32, z: f32) -> f32;

    #[inline]
    fn gradient_at(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
        let dx = self.value_at(x + EPS, y, z) - self.value_at(x - EPS, y, z);
        let dy = self.value_at(x, y + EPS, z) - self.value_at(x, y - EPS, z);
        let dz = self.value_at(x, y, z + EPS) - self.value_at(x, y, z - EPS);
        [dx, dy, dz]
    }
}

pub struct SquareField;

impl ScalarField for SquareField {
    #[inline]
    fn value_at(&self, x: f32, y: f32, z: f32) -> f32 {
        x * x + y * y + z * z
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeFrom, NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex(usize), NewtypeIndexMut(usize),
             NewtypeAdd, NewtypeAddAssign,
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub, NewtypeSubAssign,
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul, NewtypeMulAssign,
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv, NewtypeDivAssign,
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Vec2f(Vector2<f32>);
}

impl Vec2f {
    pub fn new(x: GpuScalar, y: GpuScalar) -> Self {
        Vec2f::from(Vector2::new(x, y))
    }
}

impl Zero for Vec2f {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn zero() -> Self {
        Vec2f::from(Vector2::zero())
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeFrom, NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex(usize), NewtypeIndexMut(usize),
             NewtypeAdd, NewtypeAddAssign,
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub, NewtypeSubAssign,
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul, NewtypeMulAssign,
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv, NewtypeDivAssign,
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Vec3f(Vector3<f32>);
}

impl Vec3f {
    pub fn new(x: GpuScalar, y: GpuScalar, z: GpuScalar) -> Self {
        Vec3f::from(Vector3::new(x, y, z))
    }
}

impl Zero for Vec3f {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn zero() -> Self {
        Vec3f::from(Vector3::zero())
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeFrom, NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex(usize), NewtypeIndexMut(usize),
             NewtypeAdd, NewtypeAddAssign,
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub, NewtypeSubAssign,
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul, NewtypeMulAssign,
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv, NewtypeDivAssign,
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Vec4f(Vector4<f32>);
}

impl Vec4f {
    pub fn new(x: GpuScalar, y: GpuScalar, z: GpuScalar, w: GpuScalar) -> Self {
        Vec4f::from(Vector4::new(x, y, z, w))
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeFrom, NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex(usize), NewtypeIndexMut(usize),
             NewtypeAdd(Vector2<GpuScalar>), NewtypeAddAssign(Vector2<GpuScalar>),
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub(Vector2<GpuScalar>),
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Point2f(Point2<f32>);
}

impl Point2f {
    pub fn new(x: GpuScalar, y: GpuScalar) -> Self {
        Point2f::from(Point2::new(x, y))
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeFrom, NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex(usize), NewtypeIndexMut(usize),
             NewtypeAdd(Vector3<GpuScalar>), NewtypeAddAssign(Vector3<GpuScalar>),
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Point3f(Point3<f32>);
}

impl Point3f {
    pub fn new(x: GpuScalar, y: GpuScalar, z: GpuScalar) -> Self {
        Point3f::from(Point3::new(x, y, z))
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeFrom, NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex(usize), NewtypeIndexMut(usize),
             NewtypeAdd(Vector4<GpuScalar>), NewtypeAddAssign(Vector4<GpuScalar>),
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Point4f(Point4<f32>);
}

impl Point4f {
    pub fn new(x: GpuScalar, y: GpuScalar, z: GpuScalar, w: GpuScalar) -> Self {
        Point4f::from(Point4::new(x, y, z, w))
    }
}

custom_derive! {
    #[derive(Debug, Copy, Clone, PartialEq,
             NewtypeDeref, NewtypeDerefMut,
             NewtypeIndex((usize, usize)), NewtypeIndexMut((usize, usize)),
             NewtypeAdd, NewtypeAddAssign,
             NewtypeAdd(GpuScalar), NewtypeAddAssign(GpuScalar),
             NewtypeSub, NewtypeSubAssign,
             NewtypeSub(GpuScalar), NewtypeSubAssign(GpuScalar),
             NewtypeMul, NewtypeMulAssign,
             NewtypeMul(GpuScalar), NewtypeMulAssign(GpuScalar),
             NewtypeDiv(GpuScalar), NewtypeDivAssign(GpuScalar))]
    pub struct Matrix4f(Matrix4<f32>);
}

impl Matrix4f {
    pub fn new(m11: GpuScalar,
               m21: GpuScalar,
               m31: GpuScalar,
               m41: GpuScalar,
               m12: GpuScalar,
               m22: GpuScalar,
               m32: GpuScalar,
               m42: GpuScalar,
               m13: GpuScalar,
               m23: GpuScalar,
               m33: GpuScalar,
               m43: GpuScalar,
               m14: GpuScalar,
               m24: GpuScalar,
               m34: GpuScalar,
               m44: GpuScalar)
               -> Self {
        Matrix4f::from(Matrix4::new(m11,
                                    m21,
                                    m31,
                                    m41,
                                    m12,
                                    m22,
                                    m32,
                                    m42,
                                    m13,
                                    m23,
                                    m33,
                                    m43,
                                    m14,
                                    m24,
                                    m34,
                                    m44))
    }
}

impl<T> From<T> for Matrix4f
    where Matrix4<GpuScalar>: From<T>
{
    fn from(value: T) -> Self {
        Matrix4f(Matrix4::from(value))
    }
}
