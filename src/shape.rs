use nalgebra::{Isometry2, Point2, Vector2};

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub struct Cuboid {
    pub half_extents: Vector2<f32>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct AABB {
    pub mins: Point2<f32>,
    pub maxs: Point2<f32>,
}

pub trait IsometryOps<T> {
    /// Transform a vector by the absolute value of the homogeneous matrix
    /// equivalent to `self`.
    fn absolute_transform_vector(&self, v: &Vector2<T>) -> Vector2<T>;
}

impl IsometryOps<f32> for Isometry2<f32> {
    #[inline]
    fn absolute_transform_vector(&self, v: &Vector2<f32>) -> Vector2<f32> {
        self.rotation.to_rotation_matrix().into_inner().abs() * *v
    }
}

impl AABB {
    #[inline]
    fn transform_by(&self, m: &Isometry2<f32>) -> Self {
        let ls_center = self.center();
        let center = m * ls_center;
        let ws_half_extents = m.absolute_transform_vector(&self.half_extents());

        Self {
            mins: center + (-ws_half_extents),
            maxs: center + ws_half_extents,
        }
    }

    #[inline]
    fn center(&self) -> Point2<f32> {
        nalgebra::center(&self.mins, &self.maxs)
    }

    /// The half extents of this AABB.
    #[inline]
    fn half_extents(&self) -> Vector2<f32> {
        let half: f32 = nalgebra::convert::<f64, f32>(0.5);
        (self.maxs - self.mins) * half
    }
}

impl Cuboid {
    #[inline]
    fn aabb(&self, pos: &Isometry2<f32>) -> AABB {
        let center = Point2::from(pos.translation.vector);
        let ws_half_extents = pos.absolute_transform_vector(&self.half_extents);

        {
            let mins = center - ws_half_extents;
            let maxs = center + ws_half_extents;
            AABB { mins, maxs }
        }
    }

    #[inline]
    fn local_aabb(&self) -> AABB {
        let half_extents = Point2::from(self.half_extents);

        {
            let mins = -half_extents;
            AABB {
                mins,
                maxs: half_extents,
            }
        }
    }
}

pub trait Shape {
    fn compute_local_aabb(&self) -> AABB;

    fn compute_aabb(&self, position: &Isometry2<f32>) -> AABB {
        self.compute_local_aabb().transform_by(position)
    }
}

impl Shape for Cuboid {
    fn compute_local_aabb(&self) -> AABB {
        self.local_aabb()
    }

    fn compute_aabb(&self, position: &Isometry2<f32>) -> AABB {
        self.aabb(position)
    }
}
