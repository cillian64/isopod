//! This file defines various types, constants, and functions related to 3d
//! geometry which are expected to be useful for designing patterns

#![allow(unused)]

use lazy_static::lazy_static;

/// General purpose 3d vector type
#[derive(Clone, Debug, PartialEq)]
pub struct Vector3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3d {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculate the magnitude of this vector
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    /// Return a new vector3d scaled by the desired amount
    pub fn scale(&self, scale_amount: f32) -> Vector3d {
        Vector3d {
            x: self.x * scale_amount,
            y: self.y * scale_amount,
            z: self.z * scale_amount,
        }
    }
}

/// unit 3d vector describing a direction from the origin
#[derive(Clone, Debug, PartialEq)]
pub struct UnitVector3d {
    // In reality it's just a wrapped 3d vector but with constructors which
    // ensure it is actually a well-behaved unit vector
    inner: Vector3d,
}

impl UnitVector3d {
    /// Take the provided vector and scale it to unit magnitude.  Might panic
    /// if `source` has a magnitude near zero.
    pub fn from_vector3d(source: &Vector3d) -> Self {
        Self {
            inner: source.scale(1.0 / source.magnitude()),
        }
    }

    /// Convenience wrapper of from_vector3d
    pub fn from_vector3d_components(x: f32, y: f32, z: f32) -> Self {
        let source = Vector3d { x, y, z };
        Self::from_vector3d(&source)
    }

    /// Rotate a unit vector (1, 0, 0) by the provided Euler angles
    pub fn from_angles(a: f32, b: f32, c: f32) -> Self {
        let result = UnitVector3d {
            inner: Vector3d {
                // This comes from the matrix multiplication representation of
                // a 3d rotation by 3 Euler angles, applied to the unit vector
                // (1, 0, 0) and simplified.
                x: b.cos() * c.cos(),
                y: b.cos() * c.sin(),
                z: -b.sin(),
            },
        };
        // println!("angles {} {} {}, vec {:?} mag {}",
        //         a, b, c,
        //        result.as_vector3d(),
        //     result.as_vector3d().magnitude());

        // Sanity-check the result:
        assert!(result.as_vector3d().magnitude() > 0.95);
        assert!(result.as_vector3d().magnitude() < 1.15);
        result
    }

    /// Return the Vector3d representation of this unit vector
    pub fn as_vector3d(&self) -> &Vector3d {
        &self.inner
    }
}

/// Find the dot product of two vectors
pub fn dot(a: &Vector3d, b: &Vector3d) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Find the angle between two vectors in radians
pub fn vector_angle(a: &Vector3d, b: &Vector3d) -> f32 {
    let cos_theta = dot(a, b) / (a.magnitude() * b.magnitude());
    cos_theta.acos()
}

/// Find the angle between two unit vectors in radians
pub fn unit_vector_angle(a: &UnitVector3d, b: &UnitVector3d) -> f32 {
    dot(a.as_vector3d(), b.as_vector3d()).acos()
}

/// Find the angle between two unit vectors in radians, but with direction
/// applied as sign - if the lines are closer to antiparallel than parallel
/// then return a negative angle, otherwise return a positive angle
pub fn unit_vector_angle_with_dir(a: &UnitVector3d, b: &UnitVector3d) -> f32 {
    let dot = dot(a.as_vector3d(), b.as_vector3d());
    let angle = dot.acos();
    if dot < 0.0 {
        angle * -1.0
    } else {
        angle
    }
}

lazy_static! {
    /// For each of the spines, a unit vector describing its direction from
    /// the origin.
    pub static ref SPINE_DIRECTIONS: [UnitVector3d; 12] = [
        UnitVector3d::from_vector3d_components(0.0, 1.0, 1.618),
        UnitVector3d::from_vector3d_components(0.0, 1.0, -1.618),
        UnitVector3d::from_vector3d_components(0.0, -1.0, 1.618),
        UnitVector3d::from_vector3d_components(0.0, -1.0, -1.618),
        UnitVector3d::from_vector3d_components(1.0, 1.618, 0.0),
        UnitVector3d::from_vector3d_components(1.0, -1.618, 0.0),
        UnitVector3d::from_vector3d_components(-1.0, 1.618, 0.0),
        UnitVector3d::from_vector3d_components(-1.0, -1.618, 0.0),
        UnitVector3d::from_vector3d_components(1.618, 0.0, 1.0),
        UnitVector3d::from_vector3d_components(1.618, 0.0, -1.0),
        UnitVector3d::from_vector3d_components(-1.618, 0.0, 1.0),
        UnitVector3d::from_vector3d_components(-1.618, 0.0, -1.0),
    ];
}
