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
