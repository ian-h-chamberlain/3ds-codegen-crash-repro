use nalgebra::{Isometry2, Point2, RealField, Unit, Vector2};
use parry2d::bounding_volume::{BoundingSphere, AABB};
use parry2d::mass_properties::MassProperties;
use parry2d::shape::{FeatureId, PolygonalFeature, ShapeType, TypedShape};
use parry2d::utils::IsometryOps;

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub struct Cuboid {
    pub half_extents: Vector2<f32>,
}

impl Cuboid {
    #[inline]
    fn aabb(&self, pos: &Isometry2<f32>) -> AABB {
        let center = Point2::from(pos.translation.vector);
        let ws_half_extents = pos.absolute_transform_vector(&self.half_extents);

        AABB::from_half_extents(center, ws_half_extents)
    }

    #[inline]
    fn local_aabb(&self) -> AABB {
        let half_extents = Point2::from(self.half_extents);

        AABB::new(-half_extents, half_extents)
    }

    #[inline]
    fn local_bounding_sphere(&self) -> BoundingSphere {
        let radius = self.half_extents.norm();
        BoundingSphere::new(Point2::origin(), radius)
    }
}

pub trait Shape {
    fn compute_local_aabb(&self) -> AABB;
    fn compute_local_bounding_sphere(&self) -> BoundingSphere;

    fn compute_aabb(&self, position: &Isometry2<f32>) -> AABB {
        self.compute_local_aabb().transform_by(position)
    }
    fn compute_bounding_sphere(&self, position: &Isometry2<f32>) -> BoundingSphere {
        self.compute_local_bounding_sphere().transform_by(position)
    }

    fn mass_properties(&self, density: f32) -> MassProperties;

    fn shape_type(&self) -> ShapeType;

    fn as_typed_shape(&self) -> TypedShape;

    fn ccd_thickness(&self) -> f32;

    fn ccd_angular_thickness(&self) -> f32;
}

impl Shape for Cuboid {
    fn compute_local_aabb(&self) -> AABB {
        self.local_aabb()
    }

    fn compute_local_bounding_sphere(&self) -> BoundingSphere {
        self.local_bounding_sphere()
    }

    fn compute_aabb(&self, position: &Isometry2<f32>) -> AABB {
        self.aabb(position)
    }

    fn mass_properties(&self, density: f32) -> MassProperties {
        MassProperties::from_cuboid(density, self.half_extents)
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Cuboid
    }

    fn as_typed_shape(&self) -> TypedShape {
        TypedShape::Cuboid(loop {})
    }

    fn ccd_thickness(&self) -> f32 {
        self.half_extents.min()
    }

    fn ccd_angular_thickness(&self) -> f32 {
        f32::frac_pi_2()
    }
}
