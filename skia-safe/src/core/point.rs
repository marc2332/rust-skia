use crate::{ISize, Size, prelude::*, scalar};
use skia_bindings::{self as sb, SkIPoint, SkPoint};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub use IPoint as IVector;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct IPoint {
    pub x: i32,
    pub y: i32,
}

native_transmutable!(SkIPoint, IPoint);

impl Neg for IPoint {
    type Output = IPoint;
    fn neg(self) -> Self::Output {
        IPoint::new(-self.x, -self.y)
    }
}

impl Add<IVector> for IPoint {
    type Output = IPoint;
    fn add(self, rhs: IVector) -> Self {
        IPoint::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<IVector> for IPoint {
    fn add_assign(&mut self, rhs: IVector) {
        self.x += rhs.x;
        self.y += self.y;
    }
}

impl Add<ISize> for IPoint {
    type Output = IPoint;
    fn add(self, rhs: ISize) -> Self::Output {
        IPoint::new(self.x + rhs.width, self.y + rhs.height)
    }
}

impl AddAssign<ISize> for IPoint {
    fn add_assign(&mut self, rhs: ISize) {
        self.x += rhs.width;
        self.y += rhs.height;
    }
}

impl Sub for IPoint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        IPoint::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<IVector> for IPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub<ISize> for IPoint {
    type Output = IPoint;
    fn sub(self, rhs: ISize) -> Self::Output {
        IPoint::new(self.x - rhs.width, self.y - rhs.height)
    }
}

impl SubAssign<ISize> for IPoint {
    fn sub_assign(&mut self, rhs: ISize) {
        self.x -= rhs.width;
        self.y -= rhs.height;
    }
}

impl IPoint {
    /// Sets `x` and `y`.
    ///
    /// - `x` integer x-axis value
    /// - `y` integer y-axis value
    pub const fn new(x: i32, y: i32) -> Self {
        IPoint { x, y }
    }

    /// Returns true if `x` and `y` are both zero.
    pub fn is_zero(self) -> bool {
        (self.x | self.y) == 0
    }

    /// Sets `x` and `y`.
    ///
    /// - `x` new value for `x`
    /// - `y` new value for `y`
    pub fn set(&mut self, x: i32, y: i32) {
        *self = IPoint::new(x, y);
    }

    /// Returns true if the point is equivalent to (`x`, `y`).
    ///
    /// - `x` value compared with `x`
    /// - `y` value compared with `y`
    pub fn equals(self, x: i32, y: i32) -> bool {
        self == IPoint::new(x, y)
    }
}

pub type Vector = Point;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Point {
    pub x: scalar,
    pub y: scalar,
}

native_transmutable!(SkPoint, Point);

impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Self::Output {
        Point::new(-self.x, -self.y)
    }
}

impl Add<Vector> for Point {
    type Output = Self;
    fn add(self, rhs: Vector) -> Self {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<Size> for Point {
    type Output = Self;
    fn add(self, rhs: Size) -> Self {
        Point::new(self.x + rhs.width, self.y + rhs.height)
    }
}

impl AddAssign<Size> for Point {
    fn add_assign(&mut self, rhs: Size) {
        self.x += rhs.width;
        self.y += rhs.height;
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Self) -> Self {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<Vector> for Point {
    fn sub_assign(&mut self, rhs: Vector) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub<Size> for Point {
    type Output = Self;
    fn sub(self, rhs: Size) -> Self {
        Point::new(self.x - rhs.width, self.y - rhs.height)
    }
}

impl SubAssign<Size> for Point {
    fn sub_assign(&mut self, rhs: Size) {
        self.x -= rhs.width;
        self.y -= rhs.height;
    }
}

impl Mul<scalar> for Point {
    type Output = Self;
    fn mul(self, rhs: scalar) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<scalar> for Point {
    fn mul_assign(&mut self, rhs: scalar) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

// `SkPoint.h` does not define a `/` operator, but we add it to complement Mul<>.

impl Div<scalar> for Point {
    type Output = Self;
    fn div(self, rhs: scalar) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign<scalar> for Point {
    fn div_assign(&mut self, rhs: scalar) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Point {
    /// Sets `x` and `y`. Used both to set a point and a vector.
    ///
    /// - `x` float x-axis value
    /// - `y` float y-axis value
    pub const fn new(x: scalar, y: scalar) -> Self {
        Self { x, y }
    }

    /// Returns true if `x` and `y` are both zero.
    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    /// Sets `x` and `y`.
    ///
    /// - `x` new value for `x`
    /// - `y` new value for `y`
    pub fn set(&mut self, x: scalar, y: scalar) {
        *self = Self::new(x, y);
    }

    /// Sets `x` and `y`, promoting integers to float values.
    ///
    /// Assigning a large integer value directly to `x` or `y` may cause a compiler error,
    /// triggered by narrowing conversion of int to float. This safely casts `p` to avoid the
    /// error.
    ///
    /// - `p` integer point members promoted to float
    pub fn iset(&mut self, p: impl Into<IPoint>) {
        let p = p.into();
        self.x = p.x as scalar;
        self.y = p.y as scalar;
    }

    /// Sets `x` to the absolute value of `p.x`, and `y` to the absolute value of `p.y`.
    ///
    /// - `p` members providing magnitude for `x` and `y`
    pub fn set_abs(&mut self, p: impl Into<Point>) {
        let p = p.into();
        self.x = p.x.abs();
        self.y = p.y.abs();
    }

    /// Adds `offset` to each point in `points`.
    ///
    /// - `points` point array
    /// - `offset` vector added to points
    pub fn offset_points(points: &mut [Point], offset: impl Into<Vector>) {
        let offset = offset.into();
        points.iter_mut().for_each(|p| p.offset(offset));
    }

    /// Adds `d` to the point.
    ///
    /// - `d` vector to add
    pub fn offset(&mut self, d: impl Into<Vector>) {
        *self += d.into();
    }

    /// Returns the Euclidean distance from origin, computed as `sqrt(x * x + y * y)`.
    pub fn length(self) -> scalar {
        unsafe { SkPoint::Length(self.x, self.y) }
    }

    /// Returns the Euclidean distance from origin, computed as `sqrt(x * x + y * y)`.
    pub fn distance_to_origin(self) -> scalar {
        self.length()
    }

    /// Scales so that [`Self::length()`] returns one, while preserving the ratio of `x` to `y`,
    /// if possible. If the prior length is nearly zero, sets the vector to (0, 0) and returns
    /// false; otherwise returns true.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_normalize_2>
    pub fn normalize(&mut self) -> bool {
        unsafe { self.native_mut().normalize() }
    }

    /// Sets the vector to (`x`, `y`) scaled so that [`Self::length()`] returns one, and so that
    /// the vector is proportional to (`x`, `y`). If the (`x`, `y`) length is nearly zero, sets the
    /// vector to (0, 0) and returns false; otherwise returns true.
    ///
    /// - `x` proportional value for `x`
    /// - `y` proportional value for `y`
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_setNormalize>
    pub fn set_normalize(&mut self, x: scalar, y: scalar) -> bool {
        unsafe { self.native_mut().setNormalize(x, y) }
    }

    /// Scales the vector so that [`Self::distance_to_origin()`] returns `length`, if possible. If
    /// the former length is nearly zero, sets the vector to (0, 0) and returns false; otherwise
    /// returns true.
    ///
    /// - `length` straight-line distance to origin
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_setLength>
    pub fn set_length(&mut self, length: scalar) -> bool {
        unsafe { self.native_mut().setLength(length) }
    }

    /// Sets the vector to (`x`, `y`) scaled to `length`, if possible. If the former length is
    /// nearly zero, sets the vector to (0, 0) and returns false; otherwise returns true.
    ///
    /// - `x` proportional value for `x`
    /// - `y` proportional value for `y`
    /// - `length` straight-line distance to origin
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_setLength_2>
    pub fn set_length_xy(&mut self, x: scalar, y: scalar, length: scalar) -> bool {
        unsafe { self.native_mut().setLength1(x, y, length) }
    }

    /// Returns the point times `scale`.
    ///
    /// - `scale` factor to multiply the point by
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_scale>
    #[must_use]
    pub fn scaled(self, scale: scalar) -> Self {
        let mut p = Point::default();
        unsafe { self.native().scale(scale, p.native_mut()) }
        p
    }

    /// Scales the point in place by `scale`.
    ///
    /// - `scale` factor to multiply the point by
    pub fn scale(&mut self, scale: scalar) {
        *self = self.scaled(scale);
    }

    /// Changes the sign of `x` and `y`.
    pub fn negate(&mut self) {
        *self = -*self;
    }

    /// Returns true if both `x` and `y` are measurable values.
    pub fn is_finite(self) -> bool {
        unsafe { sb::C_SkPoint_isFinite(self.native()) }
    }

    /// Returns true if the point is equivalent to (`x`, `y`).
    ///
    /// - `x` value compared with `x`
    /// - `y` value compared with `y`
    pub fn equals(self, x: scalar, y: scalar) -> bool {
        self == Point::new(x, y)
    }

    /// Returns the Euclidean distance from origin, computed as `sqrt(x * x + y * y)`.
    ///
    /// - `x` component of length
    /// - `y` component of length
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_Length>
    pub fn length_xy(x: scalar, y: scalar) -> scalar {
        unsafe { SkPoint::Length(x, y) }
    }

    /// Scales `v` so that [`Self::length()`] returns one, while preserving the ratio of `v.x` to
    /// `v.y`, if possible. If the original length is nearly zero, sets `v` to (0, 0) and returns
    /// zero; otherwise, returns the length of `v` before it is scaled.
    ///
    /// The returned prior length may be `INFINITY` if it cannot be represented by a float.
    ///
    /// Note that [`Self::normalize()`] is faster if the prior length is not required.
    ///
    /// - `v` normalized to unit length
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Point_Normalize>
    pub fn normalize_vector(v: &mut Vector) -> scalar {
        unsafe { SkPoint::Normalize(v.native_mut()) }
    }

    /// Returns the Euclidean distance between `a` and `b`.
    ///
    /// - `a` line end point
    /// - `b` line end point
    pub fn distance(a: Self, b: Self) -> scalar {
        unsafe { SkPoint::Length(a.x - b.x, a.y - b.y) }
    }

    /// Returns the dot product of `a` and `b`.
    ///
    /// - `a` left side of dot product
    /// - `b` right side of dot product
    pub fn dot_product(a: Self, b: Self) -> scalar {
        a.x * b.x + a.y * b.y
    }

    /// Returns the cross product of `a` and `b`.
    ///
    /// `a` and `b` form three-dimensional vectors with z-axis value equal to zero. The cross
    /// product is a three-dimensional vector with x-axis and y-axis values equal to zero. The
    /// cross product z-axis component is returned.
    ///
    /// - `a` left side of cross product
    /// - `b` right side of cross product
    pub fn cross_product(a: Self, b: Self) -> scalar {
        a.x * b.y - a.y * b.x
    }

    /// Returns the cross product of the point and `vec`.
    ///
    /// The point and `vec` form three-dimensional vectors with z-axis value equal to zero. The
    /// cross product is a three-dimensional vector with x-axis and y-axis values equal to zero.
    /// The cross product z-axis component is returned.
    ///
    /// - `vec` right side of cross product
    pub fn cross(self, vec: Vector) -> scalar {
        Self::cross_product(self, vec)
    }

    /// Returns the dot product of the point and `vec`.
    ///
    /// - `vec` right side of dot product
    pub fn dot(self, vec: Vector) -> scalar {
        Self::dot_product(self, vec)
    }
}

impl From<(i32, i32)> for IPoint {
    fn from(source: (i32, i32)) -> Self {
        IPoint::new(source.0, source.1)
    }
}

impl From<(scalar, scalar)> for Point {
    fn from(source: (scalar, scalar)) -> Self {
        Point::new(source.0, source.1)
    }
}

impl From<IPoint> for Point {
    fn from(source: IPoint) -> Self {
        Self::new(source.x as _, source.y as _)
    }
}

impl From<(i32, i32)> for Point {
    fn from(source: (i32, i32)) -> Self {
        (source.0 as scalar, source.1 as scalar).into()
    }
}
