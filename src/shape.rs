use downcast_rs::{impl_downcast, DowncastSync};
use nalgebra::{Isometry2, Point2, RealField, Unit, Vector2};
use parry2d::bounding_volume::{BoundingSphere, BoundingVolume, AABB};
use parry2d::mass_properties::MassProperties;
use parry2d::query::{PointProjection, PointQuery, Ray, RayCast, RayIntersection};
use parry2d::shape::{
    FeatureId, PolygonalFeature, PolygonalFeatureMap, ShapeType, SupportMap, TypedShape,
};
use parry2d::utils::IsometryOps;

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub struct Cuboid {
    pub half_extents: Vector2<f32>,
}

impl Cuboid {
    /// Creates a new box from its half-extents. Half-extents are the box half-width along each
    /// axis. Each half-extent must be positive.
    #[inline]
    pub fn new(half_extents: Vector2<f32>) -> Cuboid {
        Cuboid { half_extents }
    }

    /// Computes a scaled version of this cuboid.
    pub fn scaled(self, scale: &Vector2<f32>) -> Self {
        let new_hext = self.half_extents.component_mul(scale);
        Self {
            half_extents: new_hext,
        }
    }

    /// Return the id of the vertex of this cuboid with a normal that maximizes
    /// the dot product with `dir`.
    pub fn vertex_feature_id(vertex: Point2<f32>) -> u32 {
        ((vertex.x.to_bits() >> 31) & 0b001 | (vertex.y.to_bits() >> 30) & 0b010) as u32
    }

    /// Return the feature of this cuboid with a normal that maximizes
    /// the dot product with `dir`.
    pub fn support_feature(&self, local_dir: Vector2<f32>) -> PolygonalFeature {
        // In 2D, it is best for stability to always return a face.
        // It won't have any notable impact on performances anyway.
        self.support_face(local_dir)
    }

    /// Return the face of this cuboid with a normal that maximizes
    /// the dot product with `local_dir`.
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

    /// The normal of the given feature of this shape.
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

    /// Computes the world-space AABB of this cuboid, transformed by `pos`.
    #[inline]
    pub fn aabb(&self, pos: &Isometry2<f32>) -> AABB {
        let center = Point2::from(pos.translation.vector);
        let ws_half_extents = pos.absolute_transform_vector(&self.half_extents);

        AABB::from_half_extents(center, ws_half_extents)
    }

    /// Computes the local-space AABB of this cuboid.
    #[inline]
    pub fn local_aabb(&self) -> AABB {
        let half_extents = Point2::from(self.half_extents);

        AABB::new(-half_extents, half_extents)
    }

    /// Computes the world-space bounding sphere of this cuboid, transformed by `pos`.
    #[inline]
    pub fn bounding_sphere(&self, pos: &Isometry2<f32>) -> BoundingSphere {
        let bv: BoundingSphere = self.local_bounding_sphere();
        bv.transform_by(pos)
    }

    /// Computes the local-space bounding sphere of this cuboid.
    #[inline]
    pub fn local_bounding_sphere(&self) -> BoundingSphere {
        let radius = self.half_extents.norm();
        BoundingSphere::new(Point2::origin(), radius)
    }
}

pub trait Shape: RayCast + PointQuery + DowncastSync {
    /// Computes the AABB of this shape.
    fn compute_local_aabb(&self) -> AABB;
    /// Computes the bounding-sphere of this shape.
    fn compute_local_bounding_sphere(&self) -> BoundingSphere;

    /// Clones this shape into a boxed trait-object.
    #[cfg(feature = "std")]
    fn clone_box(&self) -> Box<dyn Shape>;

    /// Computes the AABB of this shape with the given position.
    fn compute_aabb(&self, position: &Isometry2<f32>) -> AABB {
        self.compute_local_aabb().transform_by(position)
    }
    /// Computes the bounding-sphere of this shape with the given position.
    fn compute_bounding_sphere(&self, position: &Isometry2<f32>) -> BoundingSphere {
        self.compute_local_bounding_sphere().transform_by(position)
    }

    /// Compute the mass-properties of this shape given its uniform density.
    fn mass_properties(&self, density: f32) -> MassProperties;

    /// Gets the type tag of this shape.
    fn shape_type(&self) -> ShapeType;

    /// Gets the underlying shape as an enum.
    fn as_typed_shape(&self) -> TypedShape;

    fn ccd_thickness(&self) -> f32;

    // TODO: document this.
    // This should probably be the largest sharp edge angle (in radians) in [0; PI].
    // Though this isn't a very good description considering this is PI / 2
    // for capsule (which doesn't have any sharp angle). I guess a better way
    // to phrase this is: "the smallest angle such that rotating the shape by
    // that angle may result in different contact points".
    fn ccd_angular_thickness(&self) -> f32;

    /// Is this shape known to be convex?
    ///
    /// If this returns `true` then `self` is known to be convex.
    /// If this returns `false` then it is not known whether or
    /// not `self` is convex.
    fn is_convex(&self) -> bool {
        false
    }

    /// Convents this shape into its support mapping, if it has one.
    fn as_support_map(&self) -> Option<&dyn SupportMap> {
        None
    }

    #[cfg(feature = "std")]
    fn as_composite_shape(&self) -> Option<&dyn SimdCompositeShape> {
        None
    }

    /// Converts this shape to a polygonal feature-map, if it is one.
    fn as_polygonal_feature_map(&self) -> Option<(&dyn PolygonalFeatureMap, f32)> {
        None
    }

    // fn as_rounded(&self) -> Option<&Rounded<Box<AnyShape>>> {
    //     None
    // }

    /// The shape's normal at the given point located on a specific feature.
    fn feature_normal_at_point(
        &self,
        _feature: FeatureId,
        _point: &Point2<f32>,
    ) -> Option<Unit<Vector2<f32>>> {
        None
    }

    /// Computes the swept AABB of this shape, i.e., the space it would occupy by moving from
    /// the given start position to the given end position.
    fn compute_swept_aabb(&self, start_pos: &Isometry2<f32>, end_pos: &Isometry2<f32>) -> AABB {
        let aabb1 = self.compute_aabb(start_pos);
        let aabb2 = self.compute_aabb(end_pos);
        aabb1.merged(&aabb2)
    }
}

impl_downcast!(sync Shape);

impl dyn Shape {
    /// Converts this abstract shape to the given shape, if it is one.
    pub fn as_shape<T: Shape>(&self) -> Option<&T> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to the given mutable shape, if it is one.
    pub fn as_shape_mut<T: Shape>(&mut self) -> Option<&mut T> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a cuboid, if it is one.
    pub fn as_cuboid(&self) -> Option<&Cuboid> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable cuboid, if it is one.
    pub fn as_cuboid_mut(&mut self) -> Option<&mut Cuboid> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a compound shape, if it is one.
    #[cfg(feature = "std")]
    pub fn as_compound(&self) -> Option<&Compound> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable compound shape, if it is one.
    #[cfg(feature = "std")]
    pub fn as_compound_mut(&mut self) -> Option<&mut Compound> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a triangle mesh, if it is one.
    #[cfg(feature = "std")]
    pub fn as_trimesh(&self) -> Option<&TriMesh> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable triangle mesh, if it is one.
    #[cfg(feature = "std")]
    pub fn as_trimesh_mut(&mut self) -> Option<&mut TriMesh> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a polyline, if it is one.
    #[cfg(feature = "std")]
    pub fn as_polyline(&self) -> Option<&Polyline> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable polyline, if it is one.
    #[cfg(feature = "std")]
    pub fn as_polyline_mut(&mut self) -> Option<&mut Polyline> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a heightfield, if it is one.
    #[cfg(feature = "std")]
    pub fn as_heightfield(&self) -> Option<&HeightField> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable heightfield, if it is one.
    #[cfg(feature = "std")]
    pub fn as_heightfield_mut(&mut self) -> Option<&mut HeightField> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a convex polygon, if it is one.
    #[cfg(feature = "dim2")]
    #[cfg(feature = "std")]
    pub fn as_convex_polygon(&self) -> Option<&ConvexPolygon> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable convex polygon, if it is one.
    #[cfg(feature = "dim2")]
    #[cfg(feature = "std")]
    pub fn as_convex_polygon_mut(&mut self) -> Option<&mut ConvexPolygon> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a round convex polygon, if it is one.
    #[cfg(feature = "dim2")]
    #[cfg(feature = "std")]
    pub fn as_round_convex_polygon(&self) -> Option<&RoundConvexPolygon> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable round convex polygon, if it is one.
    #[cfg(feature = "dim2")]
    #[cfg(feature = "std")]
    pub fn as_round_convex_polygon_mut(&mut self) -> Option<&mut RoundConvexPolygon> {
        self.downcast_mut()
    }

    #[cfg(feature = "dim3")]
    #[cfg(feature = "std")]
    pub fn as_convex_polyhedron(&self) -> Option<&ConvexPolyhedron> {
        self.downcast_ref()
    }
    #[cfg(feature = "dim3")]
    #[cfg(feature = "std")]
    pub fn as_convex_polyhedron_mut(&mut self) -> Option<&mut ConvexPolyhedron> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a cylinder, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_cylinder(&self) -> Option<&Cylinder> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable cylinder, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_cylinder_mut(&mut self) -> Option<&mut Cylinder> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a cone, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_cone(&self) -> Option<&Cone> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable cone, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_cone_mut(&mut self) -> Option<&mut Cone> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a round cylinder, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_round_cylinder(&self) -> Option<&RoundCylinder> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable round cylinder, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_round_cylinder_mut(&mut self) -> Option<&mut RoundCylinder> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a round cone, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_round_cone(&self) -> Option<&RoundCone> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable round cone, if it is one.
    #[cfg(feature = "dim3")]
    pub fn as_round_cone_mut(&mut self) -> Option<&mut RoundCone> {
        self.downcast_mut()
    }

    /// Converts this abstract shape to a round convex polyhedron, if it is one.
    #[cfg(feature = "dim3")]
    #[cfg(feature = "std")]
    pub fn as_round_convex_polyhedron(&self) -> Option<&RoundConvexPolyhedron> {
        self.downcast_ref()
    }
    /// Converts this abstract shape to a mutable round convex polyhedron, if it is one.
    #[cfg(feature = "dim3")]
    #[cfg(feature = "std")]
    pub fn as_round_convex_polyhedron_mut(&mut self) -> Option<&mut RoundConvexPolyhedron> {
        self.downcast_mut()
    }
}

impl PointQuery for Cuboid {
    #[inline]
    fn project_local_point(&self, pt: &Point2<f32>, solid: bool) -> PointProjection {
        loop {}
    }

    #[inline]
    fn project_local_point_and_get_feature(
        &self,
        pt: &Point2<f32>,
    ) -> (PointProjection, FeatureId) {
        loop {}
    }

    #[inline]
    fn distance_to_local_point(&self, pt: &Point2<f32>, solid: bool) -> f32 {
        loop {}
    }

    #[inline]
    fn contains_local_point(&self, pt: &Point2<f32>) -> bool {
        loop {}
    }
}

impl RayCast for Cuboid {
    #[inline]
    fn cast_local_ray(&self, ray: &Ray, max_toi: f32, solid: bool) -> Option<f32> {
        let dl = Point2::from(-self.half_extents);
        let ur = Point2::from(self.half_extents);
        AABB::new(dl, ur).cast_local_ray(ray, max_toi, solid)
    }

    #[inline]
    fn cast_local_ray_and_get_normal(
        &self,
        ray: &Ray,
        max_toi: f32,
        solid: bool,
    ) -> Option<RayIntersection> {
        let dl = Point2::from(-self.half_extents);
        let ur = Point2::from(self.half_extents);
        AABB::new(dl, ur).cast_local_ray_and_get_normal(ray, max_toi, solid)
    }
}

impl Shape for Cuboid {
    // fn clone_box(&self) -> Box<dyn Shape> {
    //     Box::new(self.clone())
    // }

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

    fn is_convex(&self) -> bool {
        true
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

    fn as_support_map(&self) -> Option<&dyn SupportMap> {
        Some(self as &dyn SupportMap)
    }

    fn as_polygonal_feature_map(&self) -> Option<(&dyn PolygonalFeatureMap, f32)> {
        Some((self as &dyn PolygonalFeatureMap, 0.0))
    }

    fn feature_normal_at_point(
        &self,
        feature: FeatureId,
        _point: &Point2<f32>,
    ) -> Option<Unit<Vector2<f32>>> {
        self.feature_normal(feature)
    }
}

impl PolygonalFeatureMap for Cuboid {
    fn local_support_feature(&self, dir: &Unit<Vector2<f32>>, out_feature: &mut PolygonalFeature) {
        loop {}
    }
}

impl SupportMap for Cuboid {
    #[inline]
    fn local_support_point(&self, dir: &Vector2<f32>) -> Point2<f32> {
        loop {}
    }
}

fn copy_sign_to(from: f32, to: f32) -> f32 {
    loop {}
}
