use crate::{prelude::*, private::is_finite, scalar};
use skia_bindings::SkPoint3;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

pub type Vector3 = Point3;
pub type Color3f = Point3;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Point3 {
    pub x: scalar,
    pub y: scalar,
    pub z: scalar,
}

native_transmutable!(SkPoint3, Point3);

impl From<(scalar, scalar, scalar)> for Point3 {
    fn from((x, y, z): (scalar, scalar, scalar)) -> Self {
        Self::new(x, y, z)
    }
}

impl Neg for Point3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Point3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Point3 {
    fn add_assign(&mut self, rhs: Point3) {
        *self = *self + rhs;
    }
}

impl Sub for Point3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Point3 {
    fn sub_assign(&mut self, rhs: Point3) {
        *self = *self - rhs;
    }
}

impl Mul<Point3> for scalar {
    type Output = Point3;

    fn mul(self, p: Point3) -> Self::Output {
        Point3::new(self * p.x, self * p.y, self * p.z)
    }
}

impl Point3 {
    /// Sets `x`, `y`, and `z`.
    ///
    /// - `x` x-axis value
    /// - `y` y-axis value
    /// - `z` z-axis value
    pub const fn new(x: scalar, y: scalar, z: scalar) -> Self {
        Self { x, y, z }
    }

    /// Sets `x`, `y`, and `z`.
    ///
    /// - `x` new value for `x`
    /// - `y` new value for `y`
    /// - `z` new value for `z`
    pub fn set(&mut self, x: scalar, y: scalar, z: scalar) {
        *self = Self::new(x, y, z);
    }

    /// Returns the Euclidean distance from (0, 0, 0) to (`x`, `y`, `z`).
    ///
    /// - `x` x-axis value
    /// - `y` y-axis value
    /// - `z` z-axis value
    pub fn length_xyz(x: scalar, y: scalar, z: scalar) -> scalar {
        unsafe { SkPoint3::Length(x, y, z) }
    }

    /// Returns the Euclidean distance from (0, 0, 0) to the point.
    pub fn length(&self) -> scalar {
        unsafe { SkPoint3::Length(self.x, self.y, self.z) }
    }

    /// Sets the point (vector) to be unit-length in the same direction as it already points. If
    /// the point has a degenerate length (i.e., nearly 0) then sets it to (0, 0, 0) and returns
    /// false; otherwise returns true.
    pub fn normalize(&mut self) -> bool {
        unsafe { self.native_mut().normalize() }
    }

    /// Returns a new point that is unit-length in the same direction as this one, or `None` if the
    /// point has a degenerate length (i.e., nearly 0).
    #[must_use]
    pub fn normalized(&self) -> Option<Self> {
        let mut normalized = *self;
        unsafe { normalized.native_mut().normalize() }.then_some(normalized)
    }

    // TODO: with_scale()?
    /// Returns a new point whose x, y, and z coordinates are scaled.
    ///
    /// - `scale` factor to multiply the point by
    #[must_use]
    pub fn scaled(&self, scale: scalar) -> Self {
        Self::new(scale * self.x, scale * self.y, scale * self.z)
    }

    /// Scales the point's coordinates by `value`.
    ///
    /// - `value` factor to multiply the point by
    pub fn scale(&mut self, value: scalar) {
        *self = self.scaled(value);
    }

    /// Returns true if `x`, `y`, and `z` are measurable values.
    pub fn is_finite(&self) -> bool {
        is_finite(&[self.x, self.y, self.z])
    }

    /// Returns the dot product of `a` and `b`, treating them as 3D vectors.
    ///
    /// - `a` left side of dot product
    /// - `b` right side of dot product
    pub fn dot_product(a: Self, b: Self) -> scalar {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    /// Returns the dot product of the point and `vec`, treating them as 3D vectors.
    ///
    /// - `vec` right side of dot product
    pub fn dot(&self, vec: Self) -> scalar {
        Self::dot_product(*self, vec)
    }

    /// Returns the cross product of `a` and `b`, treating them as 3D vectors.
    ///
    /// - `a` left side of cross product
    /// - `b` right side of cross product
    #[allow(clippy::many_single_char_names)]
    pub fn cross_product(a: Self, b: Self) -> Self {
        let x = a.y * b.z - a.z * b.y;
        let y = a.z * b.x - a.x * b.z;
        let z = a.x * b.y - a.y * b.x;
        Self { x, y, z }
    }

    /// Returns the cross product of the point and `vec`, treating them as 3D vectors.
    ///
    /// - `vec` right side of cross product
    #[must_use]
    pub fn cross(&self, vec: Self) -> Self {
        Self::cross_product(*self, vec)
    }
}
