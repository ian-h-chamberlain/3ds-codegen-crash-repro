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
    pub fn new(half_extents: Vector2<f32>) -> Cuboid {
        Cuboid { half_extents }
    }

    pub fn scaled(self, scale: &Vector2<f32>) -> Self {
        let new_hext = self.half_extents.component_mul(scale);
        Self {
            half_extents: new_hext,
        }
    }

    pub fn vertex_feature_id(vertex: Point2<f32>) -> u32 {
        ((vertex.x.to_bits() >> 31) & 0b001 | (vertex.y.to_bits() >> 30) & 0b010) as u32
    }

    pub fn support_feature(&self, local_dir: Vector2<f32>) -> PolygonalFeature {
        self.support_face(local_dir)
    }

    pub fn support_face(&self, local_dir: Vector2<f32>) -> PolygonalFeature {
        let he = self.half_extents;
        let i = local_dir.iamin();
        let j = (i + 1) % 2;
        let mut a = Point2::origin();
        a[i] = he[i];
        a[j] = he[j].copysign(local_dir[j]);

        let mut b = a;
        b[i] = -he[i];

        let vid1 = Self::vertex_feature_id(a);
        let vid2 = Self::vertex_feature_id(b);
        let fid = (vid1.max(vid2) << 2) | vid1.min(vid2) | 0b11_00_00;

        PolygonalFeature {
            vertices: [a, b],
            vids: [vid1, vid2],
            fid,
            num_vertices: 2,
        }
    }

    pub fn feature_normal(&self, feature: FeatureId) -> Option<Unit<Vector2<f32>>> {
        match feature {
            FeatureId::Face(id) => {
                let mut dir: Vector2<f32> = nalgebra::zero();

                if id < 2 {
                    dir[id as usize] = 1.0;
                } else {
                    dir[id as usize - 2] = -1.0;
                }
                Some(Unit::new_unchecked(dir))
            }
            FeatureId::Vertex(id) => {
                let mut dir: Vector2<f32> = nalgebra::zero();

                match id {
                    0b00 => {
                        dir[0] = 1.0;
                        dir[1] = 1.0;
                    }
                    0b01 => {
                        dir[1] = 1.0;
                        dir[0] = -1.0;
                    }
                    0b11 => {
                        dir[0] = -1.0;
                        dir[1] = -1.0;
                    }
                    0b10 => {
                        dir[1] = -1.0;
                        dir[0] = 1.0;
                    }
                    _ => return None,
                }

                Some(Unit::new_normalize(dir))
            }
            _ => None,
        }
    }

    #[inline]
    pub fn aabb(&self, pos: &Isometry2<f32>) -> AABB {
        let center = Point2::from(pos.translation.vector);
        let ws_half_extents = pos.absolute_transform_vector(&self.half_extents);

        AABB::from_half_extents(center, ws_half_extents)
    }

    #[inline]
    pub fn local_aabb(&self) -> AABB {
        let half_extents = Point2::from(self.half_extents);

        AABB::new(-half_extents, half_extents)
    }

    #[inline]
    pub fn bounding_sphere(&self, pos: &Isometry2<f32>) -> BoundingSphere {
        let bv: BoundingSphere = self.local_bounding_sphere();
        bv.transform_by(pos)
    }

    #[inline]
    pub fn local_bounding_sphere(&self) -> BoundingSphere {
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
