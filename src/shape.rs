use nalgebra::{Matrix2, Rotation2, UnitComplex, Vector2};

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub struct Cuboid {
    pub half_extents: Vector2<f32>,
}

pub struct AABB;

pub struct Isometry {
    rotation: UnitComplex<f32>,
}

pub trait Shape {
    fn compute_aabb(&self, _position: &Isometry) -> AABB {
        loop {}
    }
}

impl Shape for Cuboid {
    fn compute_aabb(&self, position: &Isometry) -> AABB {
        let v = &self.half_extents;
        let mul = {
            let this = position.rotation;
            let r = this.re.clone();
            let i = this.im.clone();

            Rotation2::from_matrix_unchecked(Matrix2::new(r.clone(), -i.clone(), i, r)).into_inner()
        };

        let mut res = mul.clone_owned();

        for e in res.iter_mut() {
            *e = e.abs();
        }

        let _ = res * *v;

        loop {}
    }
}
