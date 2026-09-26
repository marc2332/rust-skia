//! Holds a 3x3 matrix for transforming coordinates. This allows mapping [`Point`] and vectors
//! with translation, scaling, skewing, rotation, and perspective.
//!
//! Matrix elements are in row-major order. [`Matrix`] constexpr default constructs to identity.
//!
//! [`Matrix`] includes a hidden variable that classifies the type of matrix to improve
//! performance. [`Matrix`] is not thread safe unless [`Matrix::get_type()`] is called first.
//!
//! Example (C++): <https://fiddle.skia.org/c/@Matrix_063>

use std::{
    ops::{Index, IndexMut, Mul},
    slice,
};

use super::scalar_;
use crate::{Point, Point3, RSXform, Rect, Scalar, Size, Vector, prelude::*, scalar};
use skia_bindings::{self as sb, SkMatrix};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Matrix {
    mat: [scalar; 9usize],
    type_mask: u32,
}

native_transmutable!(SkMatrix, Matrix);

impl PartialEq for Matrix {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { sb::C_SkMatrix_Equals(self.native(), rhs.native()) }
    }
}

impl Mul for Matrix {
    type Output = Self;
    fn mul(self, rhs: Matrix) -> Self::Output {
        Matrix::concat(&self, &rhs)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Member {
    ScaleX = 0,
    SkewX = 1,
    TransX = 2,
    SkewY = 3,
    ScaleY = 4,
    TransY = 5,
    Persp0 = 6,
    Persp1 = 7,
    Persp2 = 8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AffineMember {
    ScaleX = 0,
    SkewY = 1,
    SkewX = 2,
    ScaleY = 3,
    TransX = 4,
    TransY = 5,
}

impl Index<Member> for Matrix {
    type Output = scalar;

    fn index(&self, index: Member) -> &Self::Output {
        &self[index as usize]
    }
}

impl Index<AffineMember> for Matrix {
    type Output = scalar;

    fn index(&self, index: AffineMember) -> &Self::Output {
        &self[index as usize]
    }
}

impl Index<usize> for Matrix {
    type Output = scalar;

    fn index(&self, index: usize) -> &Self::Output {
        &self.native().fMat[index]
    }
}

impl IndexMut<Member> for Matrix {
    fn index_mut(&mut self, index: Member) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}

impl IndexMut<AffineMember> for Matrix {
    fn index_mut(&mut self, index: AffineMember) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *sb::C_SkMatrix_SubscriptMut(self.native_mut(), index) }
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Matrix::new()
    }
}

impl Matrix {
    const fn new() -> Self {
        Self {
            mat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            type_mask: TypeMask::IDENTITY.bits() | 0x10,
        }
    }

    #[must_use]
    /// Sets the matrix to scale by (`sx`, `sy`). The returned matrix is:
    ///
    /// ```text
    /// | sx  0  0 |
    /// |  0 sy  0 |
    /// |  0  0  1 |
    /// ```
    ///
    /// - `sx` horizontal scale factor
    /// - `sy` vertical scale factor
    pub fn scale((sx, sy): (scalar, scalar)) -> Self {
        let mut m = Self::new();
        m.set_scale((sx, sy), None);
        m
    }

    #[must_use]
    /// Sets the matrix to translate by (`dx`, `dy`). The returned matrix is:
    ///
    /// ```text
    /// | 1 0 dx |
    /// | 0 1 dy |
    /// | 0 0  1 |
    /// ```
    ///
    /// - `d` translation as a vector
    pub fn translate(d: impl Into<Vector>) -> Self {
        let mut m = Self::new();
        m.set_translate(d);
        m
    }

    #[must_use]
    pub fn scale_translate((sx, sy): (scalar, scalar), t: impl Into<Vector>) -> Self {
        let t = t.into();
        Self::construct(|m| unsafe { sb::C_SkMatrix_ScaleTranslate(sx, sy, t.x, t.y, m) })
    }

    #[must_use]
    /// Sets the matrix to rotate by `deg` about a pivot point at (0, 0).
    ///
    /// - `deg` rotation angle in degrees (positive rotates clockwise)
    pub fn rotate_deg(deg: scalar) -> Self {
        let mut m = Self::new();
        m.set_rotate(deg, None);
        m
    }

    #[must_use]
    pub fn rotate_deg_pivot(deg: scalar, pivot: impl Into<Point>) -> Self {
        let mut m = Self::new();
        m.set_rotate(deg, pivot.into());
        m
    }

    #[must_use]
    pub fn rotate_rad(rad: scalar) -> Self {
        Self::rotate_deg(scalar_::radians_to_degrees(rad))
    }

    #[must_use]
    /// Sets the matrix to skew by (`kx`, `ky`) about the pivot point (0, 0).
    ///
    /// - `kx` horizontal skew factor
    /// - `ky` vertical skew factor
    pub fn skew((kx, ky): (scalar, scalar)) -> Self {
        let mut m = Self::new();
        m.set_skew((kx, ky), None);
        m
    }
}

pub type ScaleToFit = skia_bindings::SkMatrix_ScaleToFit;
variant_name!(ScaleToFit::Fill);

impl Matrix {
    #[deprecated(since = "0.89.0", note = "Use rect_2_rect")]
    #[must_use]
    pub fn rect_to_rect(
        src: impl AsRef<Rect>,
        dst: impl AsRef<Rect>,
        scale_to_fit: impl Into<Option<ScaleToFit>>,
    ) -> Option<Self> {
        Self::rect_2_rect(src, dst, scale_to_fit.into().unwrap_or(ScaleToFit::Fill))
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    /// Sets the matrix to:
    ///
    /// ```text
    /// | scaleX  skewX transX |
    /// |  skewY scaleY transY |
    /// |  pers0  pers1  pers2 |
    /// ```
    ///
    /// - `scale_x` horizontal scale factor
    /// - `skew_x` horizontal skew factor
    /// - `trans_x` horizontal translation
    /// - `skew_y` vertical skew factor
    /// - `scale_y` vertical scale factor
    /// - `trans_y` vertical translation
    /// - `pers_0` input x-axis perspective factor
    /// - `pers_1` input y-axis perspective factor
    /// - `pers_2` perspective scale factor
    pub fn new_all(
        scale_x: scalar,
        skew_x: scalar,
        trans_x: scalar,
        skew_y: scalar,
        scale_y: scalar,
        trans_y: scalar,
        pers_0: scalar,
        pers_1: scalar,
        pers_2: scalar,
    ) -> Self {
        let mut m = Self::new();
        m.set_all(
            scale_x, skew_x, trans_x, skew_y, scale_y, trans_y, pers_0, pers_1, pers_2,
        );
        m
    }
}

bitflags! {
    // m85: On Windows the SkMatrix_TypeMask is defined as i32,
    // but we stick to u32 (macOS / Linux), because there is no need to leak
    // the platform difference to the Rust side.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TypeMask: u32 {
        const IDENTITY = sb::SkMatrix_TypeMask_kIdentity_Mask as _;
        const TRANSLATE = sb::SkMatrix_TypeMask_kTranslate_Mask as _;
        const SCALE = sb::SkMatrix_TypeMask_kScale_Mask as _;
        const AFFINE = sb::SkMatrix_TypeMask_kAffine_Mask as _;
        const PERSPECTIVE = sb::SkMatrix_TypeMask_kPerspective_Mask as _;
    }
}

impl TypeMask {
    const UNKNOWN: u32 = sb::SkMatrix_kUnknown_Mask as _;
}

impl Matrix {
    /// Returns a bit field describing the transformations the matrix may perform. The bit field is
    /// computed conservatively, so it may include false positives. For example, when
    /// [`TypeMask::PERSPECTIVE`] is set, all other bits are set.
    pub fn get_type(&self) -> TypeMask {
        TypeMask::from_bits_truncate(unsafe { sb::C_SkMatrix_getType(self.native()) } as _)
    }

    /// Returns true if the matrix is identity. The identity matrix is:
    ///
    /// ```text
    /// | 1 0 0 |
    /// | 0 1 0 |
    /// | 0 0 1 |
    /// ```
    pub fn is_identity(&self) -> bool {
        self.get_type() == TypeMask::IDENTITY
    }

    /// Returns true if the matrix at most scales and translates. The matrix may be identity,
    /// contain only scale elements, only translate elements, or both. The matrix form is:
    ///
    /// ```text
    /// | scale-x    0    translate-x |
    /// |    0    scale-y translate-y |
    /// |    0       0         1      |
    /// ```
    pub fn is_scale_translate(&self) -> bool {
        (self.get_type() & !(TypeMask::SCALE | TypeMask::TRANSLATE)).is_empty()
    }

    /// Returns true if the matrix is identity, or translates. The matrix form is:
    ///
    /// ```text
    /// | 1 0 translate-x |
    /// | 0 1 translate-y |
    /// | 0 0      1      |
    /// ```
    pub fn is_translate(&self) -> bool {
        (self.get_type() & !TypeMask::TRANSLATE).is_empty()
    }

    /// Returns true if the matrix maps a [`Rect`] to another [`Rect`]. If true, the matrix is
    /// identity, or scales, or rotates a multiple of 90 degrees, or mirrors on axes. In all cases,
    /// the matrix may also have translation. The matrix form is either:
    ///
    /// ```text
    /// | scale-x    0    translate-x |
    /// |    0    scale-y translate-y |
    /// |    0       0         1      |
    /// ```
    ///
    /// or
    ///
    /// ```text
    /// |    0     rotate-x translate-x |
    /// | rotate-y    0     translate-y |
    /// |    0        0          1      |
    /// ```
    ///
    /// for non-zero values of scale-x, scale-y, rotate-x, and rotate-y.
    ///
    /// Also called [`Self::preserves_axis_alignment()`]; use the one that provides better inline
    /// documentation.
    pub fn rect_stays_rect(&self) -> bool {
        unsafe { sb::C_SkMatrix_rectStaysRect(self.native()) }
    }

    /// Returns true if the matrix maps a [`Rect`] to another [`Rect`]. If true, the matrix is
    /// identity, or scales, or rotates a multiple of 90 degrees, or mirrors on axes. In all cases,
    /// the matrix may also have translation. The matrix form is either:
    ///
    /// ```text
    /// | scale-x    0    translate-x |
    /// |    0    scale-y translate-y |
    /// |    0       0         1      |
    /// ```
    ///
    /// or
    ///
    /// ```text
    /// |    0     rotate-x translate-x |
    /// | rotate-y    0     translate-y |
    /// |    0        0          1      |
    /// ```
    ///
    /// for non-zero values of scale-x, scale-y, rotate-x, and rotate-y.
    ///
    /// Also called [`Self::rect_stays_rect()`]; use the one that provides better inline
    /// documentation.
    pub fn preserves_axis_alignment(&self) -> bool {
        self.rect_stays_rect()
    }

    /// Returns true if the matrix contains perspective elements. The matrix form is:
    ///
    /// ```text
    /// |       --            --              --          |
    /// |       --            --              --          |
    /// | perspective-x  perspective-y  perspective-scale |
    /// ```
    ///
    /// where perspective-x or perspective-y is non-zero, or perspective-scale is not one. All other
    /// elements may have any value.
    pub fn has_perspective(&self) -> bool {
        unsafe { sb::C_SkMatrix_hasPerspective(self.native()) }
    }

    /// Returns true if the matrix contains only translation, rotation, reflection, and uniform
    /// scale. Returns false if the matrix contains different scales, skewing, perspective, or
    /// degenerate forms that collapse to a line or point.
    ///
    /// Describes that the matrix makes rendering with and without the matrix visually alike; a
    /// transformed circle remains a circle. Mathematically, this is referred to as similarity of a
    /// Euclidean space, or a similarity transformation.
    ///
    /// Preserves right angles, keeping the arms of the angle equal lengths.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_isSimilarity>
    pub fn is_similarity(&self) -> bool {
        unsafe { self.native().isSimilarity(scalar::NEARLY_ZERO) }
    }

    /// Returns true if the matrix contains only translation, rotation, reflection, and scale. Scale
    /// may differ along rotated axes. Returns false if the matrix skews, has perspective, or has
    /// degenerate forms that collapse to a line or point.
    ///
    /// Preserves right angles, but does not require that the arms of the angle retain equal
    /// lengths.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_preservesRightAngles>
    pub fn preserves_right_angles(&self) -> bool {
        unsafe { self.native().preservesRightAngles(scalar::NEARLY_ZERO) }
    }

    /// Returns one matrix value from a particular row/column. Asserts if the index is out of range
    /// and `SK_DEBUG` is defined.
    ///
    /// - `r` matrix row to fetch
    /// - `c` matrix column to fetch
    pub fn rc(&self, r: usize, c: usize) -> scalar {
        assert!(r <= 2);
        assert!(c <= 2);
        self[r * 3 + c]
    }

    /// Returns the scale factor multiplied by the x-axis input, contributing to the x-axis output.
    /// With [`Self::map_points()`], scales [`Point`] along the x-axis.
    pub fn scale_x(&self) -> scalar {
        self[Member::ScaleX]
    }

    /// Returns the scale factor multiplied by the y-axis input, contributing to the y-axis output.
    /// With [`Self::map_points()`], scales [`Point`] along the y-axis.
    pub fn scale_y(&self) -> scalar {
        self[Member::ScaleY]
    }

    /// Returns the scale factor multiplied by the x-axis input, contributing to the y-axis output.
    /// With [`Self::map_points()`], skews [`Point`] along the y-axis. Skewing both axes can rotate
    /// [`Point`].
    pub fn skew_y(&self) -> scalar {
        self[Member::SkewY]
    }

    /// Returns the scale factor multiplied by the y-axis input, contributing to the x-axis output.
    /// With [`Self::map_points()`], skews [`Point`] along the x-axis. Skewing both axes can rotate
    /// [`Point`].
    pub fn skew_x(&self) -> scalar {
        self[Member::SkewX]
    }

    /// Returns the translation contributing to the x-axis output. With [`Self::map_points()`],
    /// moves [`Point`] along the x-axis.
    pub fn translate_x(&self) -> scalar {
        self[Member::TransX]
    }

    /// Returns the translation contributing to the y-axis output. With [`Self::map_points()`],
    /// moves [`Point`] along the y-axis.
    pub fn translate_y(&self) -> scalar {
        self[Member::TransY]
    }

    /// Returns the factor scaling input x-axis relative to input y-axis.
    pub fn persp_x(&self) -> scalar {
        self[Member::Persp0]
    }

    /// Returns the factor scaling input y-axis relative to input x-axis.
    pub fn persp_y(&self) -> scalar {
        self[Member::Persp1]
    }

    /// Sets the horizontal scale factor.
    ///
    /// - `v` horizontal scale factor to store
    pub fn set_scale_x(&mut self, v: scalar) -> &mut Self {
        self.set(Member::ScaleX, v)
    }

    /// Sets the vertical scale factor.
    ///
    /// - `v` vertical scale factor to store
    pub fn set_scale_y(&mut self, v: scalar) -> &mut Self {
        self.set(Member::ScaleY, v)
    }

    /// Sets the vertical skew factor.
    ///
    /// - `v` vertical skew factor to store
    pub fn set_skew_y(&mut self, v: scalar) -> &mut Self {
        self.set(Member::SkewY, v)
    }

    /// Sets the horizontal skew factor.
    ///
    /// - `v` horizontal skew factor to store
    pub fn set_skew_x(&mut self, v: scalar) -> &mut Self {
        self.set(Member::SkewX, v)
    }

    /// Sets the horizontal translation.
    ///
    /// - `v` horizontal translation to store
    pub fn set_translate_x(&mut self, v: scalar) -> &mut Self {
        self.set(Member::TransX, v)
    }

    /// Sets the vertical translation.
    ///
    /// - `v` vertical translation to store
    pub fn set_translate_y(&mut self, v: scalar) -> &mut Self {
        self.set(Member::TransY, v)
    }

    /// Sets the input x-axis perspective factor, which causes [`Self::map_points()`] to vary input
    /// x-axis values inversely proportional to input y-axis values.
    ///
    /// - `v` perspective factor
    pub fn set_persp_x(&mut self, v: scalar) -> &mut Self {
        self.set(Member::Persp0, v)
    }

    /// Sets the input y-axis perspective factor, which causes [`Self::map_points()`] to vary input
    /// y-axis values inversely proportional to input x-axis values.
    ///
    /// - `v` perspective factor
    pub fn set_persp_y(&mut self, v: scalar) -> &mut Self {
        self.set(Member::Persp1, v)
    }

    /// Sets all values from parameters. Sets the matrix to:
    ///
    /// ```text
    /// | scaleX  skewX transX |
    /// |  skewY scaleY transY |
    /// | persp0 persp1 persp2 |
    /// ```
    ///
    /// - `scale_x` horizontal scale factor to store
    /// - `skew_x` horizontal skew factor to store
    /// - `trans_x` horizontal translation to store
    /// - `skew_y` vertical skew factor to store
    /// - `scale_y` vertical scale factor to store
    /// - `trans_y` vertical translation to store
    /// - `persp_0` input x-axis values perspective factor to store
    /// - `persp_1` input y-axis values perspective factor to store
    /// - `persp_2` perspective scale factor to store
    #[allow(clippy::too_many_arguments)]
    pub fn set_all(
        &mut self,
        scale_x: scalar,
        skew_x: scalar,
        trans_x: scalar,
        skew_y: scalar,
        scale_y: scalar,
        trans_y: scalar,
        persp_0: scalar,
        persp_1: scalar,
        persp_2: scalar,
    ) -> &mut Self {
        self[Member::ScaleX] = scale_x;
        self[Member::SkewX] = skew_x;
        self[Member::TransX] = trans_x;
        self[Member::SkewY] = skew_y;
        self[Member::ScaleY] = scale_y;
        self[Member::TransY] = trans_y;
        self[Member::Persp0] = persp_0;
        self[Member::Persp1] = persp_1;
        self[Member::Persp2] = persp_2;
        self.type_mask = TypeMask::UNKNOWN;
        self
    }

    /// Copies the nine scalar values contained by the matrix into `buffer`, in member value
    /// ascending order: scale-x, skew-x, trans-x, skew-y, scale-y, trans-y, persp-0, persp-1,
    /// persp-2.
    ///
    /// - `buffer` storage for nine scalar values
    pub fn get_9(&self, buffer: &mut [scalar; 9]) {
        buffer.copy_from_slice(&self.mat)
    }

    /// Sets the matrix to the nine scalar values in `buffer`, in member value ascending order:
    /// scale-x, skew-x, trans-x, skew-y, scale-y, trans-y, persp-0, persp-1, persp-2.
    ///
    /// Sets the matrix to:
    ///
    /// ```text
    /// | buffer[0] buffer[1] buffer[2] |
    /// | buffer[3] buffer[4] buffer[5] |
    /// | buffer[6] buffer[7] buffer[8] |
    /// ```
    ///
    /// In the future, `set_9` followed by `get_9` may not return the same values. Since the matrix
    /// maps non-homogeneous coordinates, scaling all nine values produces an equivalent
    /// transformation, possibly improving precision.
    ///
    /// - `buffer` nine scalar values
    pub fn set_9(&mut self, buffer: &[scalar; 9]) -> &mut Self {
        unsafe {
            self.native_mut().set9(buffer.as_ptr());
        }
        self
    }

    /// Sets the matrix to identity, which has no effect on mapped points. Sets the matrix to:
    ///
    /// ```text
    /// | 1 0 0 |
    /// | 0 1 0 |
    /// | 0 0 1 |
    /// ```
    ///
    /// Also called [`Self::set_identity()`]; use the one that provides better inline documentation.
    pub fn reset(&mut self) -> &mut Self {
        unsafe {
            self.native_mut().reset();
        }
        self
    }

    /// Sets the matrix to identity, which has no effect on mapped points. Sets the matrix to:
    ///
    /// ```text
    /// | 1 0 0 |
    /// | 0 1 0 |
    /// | 0 0 1 |
    /// ```
    ///
    /// Also called [`Self::reset()`]; use the one that provides better inline documentation.
    pub fn set_identity(&mut self) -> &mut Self {
        self.reset();
        self
    }

    /// Sets the matrix to translate by `v`.
    ///
    /// - `v` vector containing horizontal and vertical translation
    pub fn set_translate(&mut self, v: impl Into<Vector>) -> &mut Self {
        let v = v.into();
        unsafe {
            self.native_mut().setTranslate(v.x, v.y);
        }
        self
    }

    /// Sets the matrix to scale by (`sx`, `sy`), about a pivot point at `pivot`. The pivot point is
    /// unchanged when mapped with the matrix.
    ///
    /// - `sx` horizontal scale factor
    /// - `sy` vertical scale factor
    /// - `pivot` pivot on x- and y-axes
    pub fn set_scale(
        &mut self,
        (sx, sy): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().setScale(sx, sy, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to rotate by `degrees` about a pivot point at `pivot`. The pivot point is
    /// unchanged when mapped with the matrix.
    ///
    /// Positive degrees rotates clockwise.
    ///
    /// - `degrees` angle of axes relative to upright axes
    /// - `pivot` pivot on x- and y-axes
    pub fn set_rotate(&mut self, degrees: scalar, pivot: impl Into<Option<Point>>) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().setRotate(degrees, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to rotate by `sin_value` and `cos_value`, about a pivot point at `pivot`.
    /// The pivot point is unchanged when mapped with the matrix.
    ///
    /// Vector (`sin_value`, `cos_value`) describes the angle of rotation relative to (0, 1). Vector
    /// length specifies scale.
    ///
    /// - `sin_value` rotation vector x-axis component
    /// - `cos_value` rotation vector y-axis component
    /// - `pivot` pivot on x- and y-axes
    pub fn set_sin_cos(
        &mut self,
        (sin_value, cos_value): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut()
                .setSinCos(sin_value, cos_value, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to rotate, scale, and translate using a compressed matrix form.
    ///
    /// Vector (`rsxform.scos`, `rsxform.ssin`) describes the angle of rotation relative to (0, 1).
    /// Vector length specifies scale. The mapped point is rotated and scaled by the vector, then
    /// translated by (`rsxform.tx`, `rsxform.ty`).
    ///
    /// - `rsxform` compressed [`RSXform`] matrix
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_setRSXform>
    pub fn set_rsxform(&mut self, rsxform: &RSXform) -> &mut Self {
        unsafe {
            self.native_mut().setRSXform(rsxform.native());
        }
        self
    }

    /// Sets the matrix to skew by (`kx`, `ky`), about a pivot point at `pivot`. The pivot point is
    /// unchanged when mapped with the matrix.
    ///
    /// - `kx` horizontal skew factor
    /// - `ky` vertical skew factor
    /// - `pivot` pivot on x- and y-axes
    pub fn set_skew(
        &mut self,
        (kx, ky): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().setSkew(kx, ky, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to matrix `a` multiplied by matrix `b`. Either `a` or `b` may be this.
    ///
    /// Given:
    ///
    /// ```text
    ///         | A B C |      | J K L |
    ///     a = | D E F |, b = | M N O |
    ///         | G H I |      | P Q R |
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///             | A B C |   | J K L |   | AJ+BM+CP AK+BN+CQ AL+BO+CR |
    ///     a * b = | D E F | * | M N O | = | DJ+EM+FP DK+EN+FQ DL+EO+FR |
    ///             | G H I |   | P Q R |   | GJ+HM+IP GK+HN+IQ GL+HO+IR |
    /// ```
    ///
    /// - `a` matrix on left side of multiply expression
    /// - `b` matrix on right side of multiply expression
    pub fn set_concat(&mut self, a: &Self, b: &Self) -> &mut Self {
        unsafe {
            self.native_mut().setConcat(a.native(), b.native());
        }
        self
    }

    /// Sets the matrix to the matrix multiplied by a matrix constructed from translation `delta`.
    /// This can be thought of as moving the point to be mapped before applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | A B C |               | 1 0 dx |
    ///     Matrix = | D E F |,  T(dx, dy) = | 0 1 dy |
    ///              | G H I |               | 0 0  1 |
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                          | A B C | | 1 0 dx |   | A B A*dx+B*dy+C |
    ///     Matrix * T(dx, dy) = | D E F | | 0 1 dy | = | D E D*dx+E*dy+F |
    ///                          | G H I | | 0 0  1 |   | G H G*dx+H*dy+I |
    /// ```
    ///
    /// - `delta` x- and y-axis translation before applying the matrix
    pub fn pre_translate(&mut self, delta: impl Into<Vector>) -> &mut Self {
        let delta = delta.into();
        unsafe {
            self.native_mut().preTranslate(delta.x, delta.y);
        }
        self
    }

    /// Sets the matrix to the matrix multiplied by a matrix constructed from scaling by (`sx`, `sy`)
    /// about the pivot point `pivot`. This can be thought of as scaling about a pivot point before
    /// applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | A B C |                       | sx  0 dx |
    ///     Matrix = | D E F |,  S(sx, sy, px, py) = |  0 sy dy |
    ///              | G H I |                       |  0  0  1 |
    /// ```
    ///
    /// where
    ///
    /// ```text
    ///     dx = px - sx * px
    ///     dy = py - sy * py
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                                      | A B C | | sx  0 dx |   | A*sx B*sy A*dx+B*dy+C |
    ///     Matrix * S(sx, sy, px, py) =     | D E F | |  0 sy dy | = | D*sx E*sy D*dx+E*dy+F |
    ///                                      | G H I | |  0  0  1 |   | G*sx H*sy G*dx+H*dy+I |
    /// ```
    ///
    /// - `sx` horizontal scale factor
    /// - `sy` vertical scale factor
    /// - `pivot` pivot on x- and y-axes
    pub fn pre_scale(
        &mut self,
        (sx, sy): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().preScale(sx, sy, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to the matrix multiplied by a matrix constructed from rotating by `degrees`
    /// about the pivot point `pivot`. This can be thought of as rotating about a pivot point before
    /// applying the matrix.
    ///
    /// Positive degrees rotates clockwise.
    ///
    /// Given:
    ///
    /// ```text
    ///              | A B C |                        | c -s dx |
    ///     Matrix = | D E F |,  R(degrees, px, py) = | s  c dy |
    ///              | G H I |                        | 0  0  1 |
    /// ```
    ///
    /// where
    ///
    /// ```text
    ///     c  = cos(degrees)
    ///     s  = sin(degrees)
    ///     dx =  s * py + (1 - c) * px
    ///     dy = -s * px + (1 - c) * py
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                                       | A B C | | c -s dx |   | Ac+Bs -As+Bc A*dx+B*dy+C |
    ///     Matrix * R(degrees, px, py) =     | D E F | | s  c dy | = | Dc+Es -Ds+Ec D*dx+E*dy+F |
    ///                                       | G H I | | 0  0  1 |   | Gc+Hs -Gs+Hc G*dx+H*dy+I |
    /// ```
    ///
    /// - `degrees` angle of axes relative to upright axes
    /// - `pivot` pivot on x- and y-axes
    pub fn pre_rotate(&mut self, degrees: scalar, pivot: impl Into<Option<Point>>) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().preRotate(degrees, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to the matrix multiplied by a matrix constructed from skewing by (`kx`, `ky`)
    /// about the pivot point `pivot`. This can be thought of as skewing about a pivot point before
    /// applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | A B C |                       |  1 kx dx |
    ///     Matrix = | D E F |,  K(kx, ky, px, py) = | ky  1 dy |
    ///              | G H I |                       |  0  0  1 |
    /// ```
    ///
    /// where
    ///
    /// ```text
    ///     dx = -kx * py
    ///     dy = -ky * px
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                                      | A B C | |  1 kx dx |   | A+B*ky A*kx+B A*dx+B*dy+C |
    ///     Matrix * K(kx, ky, px, py) =     | D E F | | ky  1 dy | = | D+E*ky D*kx+E D*dx+E*dy+F |
    ///                                      | G H I | |  0  0  1 |   | G+H*ky G*kx+H G*dx+H*dy+I |
    /// ```
    ///
    /// - `kx` horizontal skew factor
    /// - `ky` vertical skew factor
    /// - `pivot` pivot on x- and y-axes
    pub fn pre_skew(
        &mut self,
        (kx, ky): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().preSkew(kx, ky, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to the matrix multiplied by the matrix `other`. This can be thought of as
    /// mapping by `other` before applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | A B C |          | J K L |
    ///     Matrix = | D E F |, other = | M N O |
    ///              | G H I |          | P Q R |
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                      | A B C |   | J K L |   | AJ+BM+CP AK+BN+CQ AL+BO+CR |
    ///     Matrix * other = | D E F | * | M N O | = | DJ+EM+FP DK+EN+FQ DL+EO+FR |
    ///                      | G H I |   | P Q R |   | GJ+HM+IP GK+HN+IQ GL+HO+IR |
    /// ```
    ///
    /// - `other` matrix on the right side of the multiply expression
    pub fn pre_concat(&mut self, other: &Self) -> &mut Self {
        unsafe {
            self.native_mut().preConcat(other.native());
        }
        self
    }

    /// Sets the matrix to a matrix constructed from translation `delta` multiplied by the matrix.
    /// This can be thought of as moving the point to be mapped after applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | J K L |               | 1 0 dx |
    ///     Matrix = | M N O |,  T(dx, dy) = | 0 1 dy |
    ///              | P Q R |               | 0 0  1 |
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                          | 1 0 dx | | J K L |   | J+dx*P K+dx*Q L+dx*R |
    ///     T(dx, dy) * Matrix = | 0 1 dy | | M N O | = | M+dy*P N+dy*Q O+dy*R |
    ///                          | 0 0  1 | | P Q R |   |      P      Q      R |
    /// ```
    ///
    /// - `delta` x- and y-axis translation after applying the matrix
    pub fn post_translate(&mut self, delta: impl Into<Vector>) -> &mut Self {
        let delta = delta.into();
        unsafe {
            self.native_mut().postTranslate(delta.x, delta.y);
        }
        self
    }

    /// Sets the matrix to a matrix constructed from scaling by (`sx`, `sy`) about the pivot point
    /// `pivot`, multiplied by the matrix. This can be thought of as scaling about a pivot point
    /// after applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | J K L |                       | sx  0 dx |
    ///     Matrix = | M N O |,  S(sx, sy, px, py) = |  0 sy dy |
    ///              | P Q R |                       |  0  0  1 |
    /// ```
    ///
    /// where
    ///
    /// ```text
    ///     dx = px - sx * px
    ///     dy = py - sy * py
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                                      | sx  0 dx | | J K L |   | sx*J+dx*P sx*K+dx*Q sx*L+dx*R |
    ///     S(sx, sy, px, py) * Matrix =     |  0 sy dy | | M N O | = | sy*M+dy*P sy*N+dy*Q sy*O+dy*R |
    ///                                      |  0  0  1 | | P Q R |   |         P         Q         R |
    /// ```
    ///
    /// - `sx` horizontal scale factor
    /// - `sy` vertical scale factor
    /// - `pivot` pivot on x- and y-axes
    pub fn post_scale(
        &mut self,
        (sx, sy): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().postScale(sx, sy, pivot.x, pivot.y);
        }
        self
    }

    #[deprecated(
        since = "0.27.0",
        note = "use post_scale((1.0 / x as scalar, 1.0 / y as scalar), None)"
    )]
    pub fn post_idiv(&mut self, (div_x, div_y): (i32, i32)) -> bool {
        if div_x == 0 || div_y == 0 {
            return false;
        }
        self.post_scale((1.0 / div_x as scalar, 1.0 / div_y as scalar), None);
        true
    }

    /// Sets the matrix to a matrix constructed from rotating by `degrees` about the pivot point
    /// `pivot`, multiplied by the matrix. This can be thought of as rotating about a pivot point
    /// after applying the matrix.
    ///
    /// Positive degrees rotates clockwise.
    ///
    /// Given:
    ///
    /// ```text
    ///              | J K L |                        | c -s dx |
    ///     Matrix = | M N O |,  R(degrees, px, py) = | s  c dy |
    ///              | P Q R |                        | 0  0  1 |
    /// ```
    ///
    /// where
    ///
    /// ```text
    ///     c  = cos(degrees)
    ///     s  = sin(degrees)
    ///     dx =  s * py + (1 - c) * px
    ///     dy = -s * px + (1 - c) * py
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                                       | c -s dx | | J K L |   | cJ-sM+dx*P cK-sN+dx*Q cL-sO+dx*R |
    ///     R(degrees, px, py) * Matrix =     | s  c dy | | M N O | = | sJ+cM+dy*P sK+cN+dy*Q sL+cO+dy*R |
    ///                                       | 0  0  1 | | P Q R |   |         P          Q          R |
    /// ```
    ///
    /// - `degrees` angle of axes relative to upright axes
    /// - `pivot` pivot on x- and y-axes
    pub fn post_rotate(&mut self, degrees: scalar, pivot: impl Into<Option<Point>>) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().postRotate(degrees, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to a matrix constructed from skewing by (`kx`, `ky`) about the pivot point
    /// `pivot`, multiplied by the matrix. This can be thought of as skewing about a pivot point
    /// after applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | J K L |                       |  1 kx dx |
    ///     Matrix = | M N O |,  K(kx, ky, px, py) = | ky  1 dy |
    ///              | P Q R |                       |  0  0  1 |
    /// ```
    ///
    /// where
    ///
    /// ```text
    ///     dx = -kx * py
    ///     dy = -ky * px
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                                      | 1 kx dx | | J K L |   | J+kx*M+dx*P K+kx*N+dx*Q L+kx*O+dx*R |
    ///     K(kx, ky, px, py) * Matrix =     | ky  1 dy | | M N O | = | ky*J+M+dy*P ky*K+N+dy*Q ky*L+O+dy*R |
    ///                                      | 0  0  1 | | P Q R |   |          P           Q           R |
    /// ```
    ///
    /// - `kx` horizontal skew factor
    /// - `ky` vertical skew factor
    /// - `pivot` pivot on x- and y-axes
    pub fn post_skew(
        &mut self,
        (kx, ky): (scalar, scalar),
        pivot: impl Into<Option<Point>>,
    ) -> &mut Self {
        let pivot = pivot.into().unwrap_or_default();
        unsafe {
            self.native_mut().postSkew(kx, ky, pivot.x, pivot.y);
        }
        self
    }

    /// Sets the matrix to the matrix `other` multiplied by the matrix. This can be thought of as
    /// mapping by `other` after applying the matrix.
    ///
    /// Given:
    ///
    /// ```text
    ///              | J K L |           | A B C |
    ///     Matrix = | M N O |,  other = | D E F |
    ///              | P Q R |           | G H I |
    /// ```
    ///
    /// sets the matrix to:
    ///
    /// ```text
    ///                      | A B C |   | J K L |   | AJ+BM+CP AK+BN+CQ AL+BO+CR |
    ///     other * Matrix = | D E F | * | M N O | = | DJ+EM+FP DK+EN+FQ DL+EO+FR |
    ///                      | G H I |   | P Q R |   | GJ+HM+IP GK+HN+IQ GL+HO+IR |
    /// ```
    ///
    /// - `other` matrix on the left side of the multiply expression
    pub fn post_concat(&mut self, other: &Matrix) -> &mut Self {
        unsafe {
            self.native_mut().postConcat(other.native());
        }
        self
    }

    #[deprecated(since = "0.88.0", note = "Use rect_2_rect")]
    pub fn from_rect_to_rect(
        src: impl AsRef<Rect>,
        dst: impl AsRef<Rect>,
        stf: ScaleToFit,
    ) -> Option<Self> {
        Self::rect_2_rect(src, dst, stf)
    }

    /// If possible, returns a matrix that will transform the `src` rect to the `dst` rect. If `src`
    /// is empty, this will return `None`. If `dst` is empty, this will return the zero matrix
    /// (degenerate).
    ///
    /// - `src` source rectangle
    /// - `dst` destination rectangle
    /// - `stf` scale-to-fit mode, defaults to [`ScaleToFit::Fill`]
    pub fn rect_2_rect(
        src: impl AsRef<Rect>,
        dst: impl AsRef<Rect>,
        stf: impl Into<Option<ScaleToFit>>,
    ) -> Option<Self> {
        let mut m = Self::new_identity();
        unsafe {
            sb::C_SkMatrix_Rect2Rect(
                src.as_ref().native(),
                dst.as_ref().native(),
                stf.into().unwrap_or(ScaleToFit::Fill),
                m.native_mut(),
            )
        }
        .then_some(m)
    }

    /// Returns the matrix that transforms the `src` rect to the `dst` rect, or the identity matrix
    /// if such a matrix does not exist (e.g. if `src` is empty).
    ///
    /// - `src` source rectangle
    /// - `dst` destination rectangle
    /// - `stf` scale-to-fit mode, defaults to [`ScaleToFit::Fill`]
    pub fn rect_to_rect_or_identity(
        src: impl AsRef<Rect>,
        dst: impl AsRef<Rect>,
        stf: impl Into<Option<ScaleToFit>>,
    ) -> Self {
        Self::rect_2_rect(src, dst, stf).unwrap_or(Matrix::new_identity())
    }

    /// Computes a matrix from two polygons, such that if the matrix was applied to the `src`
    /// polygon, it would produce the `dst` polygon.
    ///
    /// If the size of the two spans are not equal, or if they are greater than 4, returns `None`.
    /// If the resulting matrix is non-invertible, returns `None`.
    ///
    /// - `src` source polygon points
    /// - `dst` destination polygon points
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_setPolyToPoly>
    pub fn poly_to_poly(src: &[Point], dst: &[Point]) -> Option<Matrix> {
        let mut m = Matrix::new();
        m.set_poly_to_poly(src, dst).then_some(m)
    }

    /// Sets the matrix to a matrix computed from two polygons, such that if the matrix was applied
    /// to the `src` polygon, it would produce the `dst` polygon.
    ///
    /// If the size of the two spans are not equal, or if they are greater than 4, returns `false`.
    /// If the resulting matrix is non-invertible, returns `false`.
    ///
    /// - `src` source polygon points
    /// - `dst` destination polygon points
    pub fn set_poly_to_poly(&mut self, src: &[Point], dst: &[Point]) -> bool {
        unsafe {
            sb::C_SkMatrix_setPolyToPoly(
                self.native_mut(),
                src.native().as_ptr(),
                src.len(),
                dst.native().as_ptr(),
                dst.len(),
            )
        }
    }

    /// Computes a matrix from two polygons, such that if the matrix was applied to the `src`
    /// polygon, it would produce the `dst` polygon.
    ///
    /// If the size of the two spans are not equal, or if they are greater than 4, returns `None`.
    /// If the resulting matrix is non-invertible, returns `None`.
    ///
    /// - `src` source polygon points
    /// - `dst` destination polygon points
    pub fn from_poly_to_poly(src: &[Point], dst: &[Point]) -> Option<Matrix> {
        let mut m = Matrix::new_identity();
        m.set_poly_to_poly(src, dst).then_some(m)
    }

    /// If this matrix is invertible, returns its inverse; otherwise, returns `None`.
    pub fn invert(&self) -> Option<Matrix> {
        let mut m = Matrix::new_identity();
        unsafe { sb::C_SkMatrix_invert(self.native(), m.native_mut()) }.then_some(m)
    }

    /// Fills `affine` with identity values in column major order. Sets `affine` to:
    ///
    /// ```text
    /// | 1 0 0 |
    /// | 0 1 0 |
    /// ```
    ///
    /// Affine 3 by 2 matrices in column major order are used by OpenGL and XPS.
    ///
    /// - `affine` storage for the 3 by 2 affine matrix
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_SetAffineIdentity>
    pub fn set_affine_identity(affine: &mut [scalar; 6]) {
        unsafe { SkMatrix::SetAffineIdentity(affine.as_mut_ptr()) }
    }

    /// Fills `affine` in column major order. Sets `affine` to:
    ///
    /// ```text
    /// | scale-x  skew-x translate-x |
    /// |  skew-y scale-y translate-y |
    /// ```
    ///
    /// If the matrix contains perspective, returns `None` and leaves `affine` unchanged.
    #[must_use]
    pub fn to_affine(self) -> Option<[scalar; 6]> {
        let mut affine = [scalar::default(); 6];
        unsafe { self.native().asAffine(affine.as_mut_ptr()) }.then_some(affine)
    }

    /// Sets the matrix to affine values, passed in column major order. Given `affine`, column, then
    /// row, as:
    ///
    /// ```text
    /// | scale-x  skew-x translate-x |
    /// |  skew-y scale-y translate-y |
    /// ```
    ///
    /// the matrix is set, row, then column, to:
    ///
    /// ```text
    /// | scale-x  skew-x translate-x |
    /// |  skew-y scale-y translate-y |
    /// |       0       0           1 |
    /// ```
    ///
    /// - `affine` 3 by 2 affine matrix
    pub fn set_affine(&mut self, affine: &[scalar; 6]) -> &mut Self {
        unsafe { self.native_mut().setAffine(affine.as_ptr()) };
        self
    }

    /// Creates a matrix from affine values, passed in column major order. Given `affine`, column,
    /// then row, as:
    ///
    /// ```text
    /// | scale-x  skew-x translate-x |
    /// |  skew-y scale-y translate-y |
    /// ```
    ///
    /// the matrix is set, row, then column, to:
    ///
    /// ```text
    /// | scale-x  skew-x translate-x |
    /// |  skew-y scale-y translate-y |
    /// |       0       0           1 |
    /// ```
    ///
    /// - `affine` 3 by 2 affine matrix
    pub fn from_affine(affine: &[scalar; 6]) -> Matrix {
        let mut m = Matrix::new_identity();
        unsafe {
            m.native_mut().setAffine(affine.as_ptr());
        }
        m
    }

    /// A matrix is categorized as 'perspective' if the bottom row is not `[0, 0, 1]`. However, for
    /// most uses (e.g. [`Self::map_points()`]) a bottom row of `[0, 0, X]` behaves like a
    /// non-perspective matrix, though it will be categorized as perspective. Calling
    /// `normalize_perspective()` will change the matrix such that, if its bottom row was `[0, 0, X]`,
    /// it will be changed to `[0, 0, 1]` by scaling the rest of the matrix by `1/X`.
    ///
    /// ```text
    /// | A B C |    | A/X B/X C/X |
    /// | D E F | -> | D/X E/X F/X |   for X != 0
    /// | 0 0 X |    |  0   0   1  |
    /// ```
    pub fn normalize_perspective(&mut self) {
        unsafe { sb::C_SkMatrix_normalizePerspective(self.native_mut()) }
    }

    /// Maps the `src` point array to the `dst` point array. Points are mapped by multiplying each
    /// point by the matrix. Given:
    ///
    /// ```text
    ///              | A B C |        | x |
    ///     Matrix = | D E F |,  pt = | y |
    ///              | G H I |        | 1 |
    /// ```
    ///
    /// each `dst` point is computed as:
    ///
    /// ```text
    ///                   |A B C| |x|                               Ax+By+C   Dx+Ey+F
    ///     Matrix * pt = |D E F| |y| = |Ax+By+C Dx+Ey+F Gx+Hy+I| = ------- , -------
    ///                   |G H I| |1|                               Gx+Hy+I   Gx+Hy+I
    /// ```
    ///
    /// `src` and `dst` may point to the same storage.
    ///
    /// - `dst` storage where the transformed points are written
    /// - `src` storage where the points are read from
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_mapPoints>
    pub fn map_points(&self, dst: &mut [Point], src: &[Point]) {
        assert!(dst.len() >= src.len());

        unsafe {
            sb::C_SkMatrix_mapPoints(
                self.native(),
                dst.native_mut().as_mut_ptr(),
                src.native().as_ptr(),
                src.len(),
            )
        };
    }

    /// Maps the point array in place. Points are mapped by multiplying each point by the matrix.
    /// Given:
    ///
    /// ```text
    ///              | A B C |        | x |
    ///     Matrix = | D E F |,  pt = | y |
    ///              | G H I |        | 1 |
    /// ```
    ///
    /// each resulting point is computed as:
    ///
    /// ```text
    ///                   |A B C| |x|                               Ax+By+C   Dx+Ey+F
    ///     Matrix * pt = |D E F| |y| = |Ax+By+C Dx+Ey+F Gx+Hy+I| = ------- , -------
    ///                   |G H I| |1|                               Gx+Hy+I   Gx+Hy+I
    /// ```
    ///
    /// - `pts` points to be transformed in place
    pub fn map_points_inplace(&self, pts: &mut [Point]) {
        let ptr = pts.native_mut().as_mut_ptr();
        unsafe { sb::C_SkMatrix_mapPoints(self.native(), ptr, ptr, pts.len()) };
    }

    /// Maps the `src` point array to the `dst` point array. Points are mapped by multiplying each
    /// point by the matrix. Given:
    ///
    /// ```text
    ///              | A B C |         | x |
    ///     Matrix = | D E F |,  src = | y |
    ///              | G H I |         | z |
    /// ```
    ///
    /// each resulting point is computed as:
    ///
    /// ```text
    ///                |A B C| |x|
    ///     Matrix * src = |D E F| |y| = |Ax+By+Cz Dx+Ey+Fz Gx+Hy+Iz|
    ///                |G H I| |z|
    /// ```
    ///
    /// - `dst` storage where the transformed points are written
    /// - `src` storage where the points are read from
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_mapHomogeneousPoints>
    pub fn map_homogeneous_points(&self, dst: &mut [Point3], src: &[Point3]) {
        assert!(dst.len() >= src.len());

        unsafe {
            sb::C_SkMatrix_mapHomogeneousPoints(
                self.native(),
                dst.native_mut().as_mut_ptr(),
                src.native().as_ptr(),
                src.len(),
            )
        };
    }

    /// Maps `src` point by the matrix.
    ///
    /// - `src` point to map
    pub fn map_homogeneous_point(&self, src: Point3) -> Point3 {
        let mut dst = Point3::default();
        self.map_homogeneous_points(slice::from_mut(&mut dst), &[src]);
        dst
    }

    #[deprecated(since = "0.88.0", note = "use map_points_to_homogeneous()")]
    pub fn map_homogeneous_points_2d(&self, dst: &mut [Point3], src: &[Point]) {
        self.map_points_to_homogeneous(dst, src);
    }

    /// Returns homogeneous points, starting with the 2D `src` points (with implied `w = 1`).
    ///
    /// - `dst` storage where the transformed points are written
    /// - `src` storage where the points are read from
    pub fn map_points_to_homogeneous(&self, dst: &mut [Point3], src: &[Point]) {
        assert!(dst.len() >= src.len());

        unsafe {
            sb::C_SkMatrix_mapPointsToHomogeneous(
                self.native(),
                dst.native_mut().as_mut_ptr(),
                src.native().as_ptr(),
                src.len(),
            )
        };
    }

    /// Returns `src` mapped to a homogeneous point.
    ///
    /// - `src` point to map
    pub fn map_point_to_homogeneous(&self, src: Point) -> Point3 {
        let mut dst = Point3::default();
        self.map_points_to_homogeneous(slice::from_mut(&mut dst), &[src]);
        dst
    }

    #[deprecated(since = "0.88.0", note = "use map_point((x, y))")]
    pub fn map_xy(&self, x: scalar, y: scalar) -> Point {
        self.map_point((x, y))
    }

    /// Returns the point multiplied by the matrix. Given:
    ///
    /// ```text
    ///              | A B C |        | x |
    ///     Matrix = | D E F |,  pt = | y |
    ///              | G H I |        | 1 |
    /// ```
    ///
    /// the result is computed as:
    ///
    /// ```text
    ///                   |A B C| |x|                               Ax+By+C   Dx+Ey+F
    ///     Matrix * pt = |D E F| |y| = |Ax+By+C Dx+Ey+F Gx+Hy+I| = ------- , -------
    ///                   |G H I| |1|                               Gx+Hy+I   Gx+Hy+I
    /// ```
    ///
    /// - `point` point to map
    pub fn map_point(&self, point: impl Into<Point>) -> Point {
        Point::from_native_c(unsafe {
            sb::C_SkMatrix_mapPoint(self.native(), point.into().into_native())
        })
    }

    /// If the caller knows the matrix has no perspective, this will inline the math, making it more
    /// efficient than calling [`Self::map_point()`].
    ///
    /// - `point` point to map
    pub fn map_point_affine(&self, point: impl Into<Point>) -> Point {
        Point::from_native_c(unsafe {
            sb::C_SkMatrix_mapPointAffine(self.native(), point.into().into_native())
        })
    }

    /// Returns `(0, 0)` multiplied by the matrix. Given:
    ///
    /// ```text
    ///              | A B C |        | 0 |
    ///     Matrix = | D E F |,  pt = | 0 |
    ///              | G H I |        | 1 |
    /// ```
    ///
    /// the result is computed as:
    ///
    /// ```text
    ///                   |A B C| |0|             C    F
    ///     Matrix * pt = |D E F| |0| = |C F I| = -  , -
    ///                   |G H I| |1|             I    I
    /// ```
    pub fn map_origin(&self) -> Point {
        let mut x = self.translate_x();
        let mut y = self.translate_y();
        if self.has_perspective() {
            let mut w = self[Member::Persp2];
            if w != 0.0 {
                w = 1.0 / w;
            }
            x *= w;
            y *= w;
        }
        Point::new(x, y)
    }

    /// Maps the `src` vector array to the `dst` vector array. Vectors are mapped by multiplying
    /// each vector by the matrix, treating the matrix translation as zero. Given:
    ///
    /// ```text
    ///              | A B 0 |         | x |
    ///     Matrix = | D E 0 |,  src = | y |
    ///              | G H I |         | 1 |
    /// ```
    ///
    /// each `dst` vector is computed as:
    ///
    /// ```text
    ///                |A B 0| |x|                            Ax+By     Dx+Ey
    ///     Matrix * src = |D E 0| |y| = |Ax+By Dx+Ey Gx+Hy+I| = ------- , -------
    ///                |G H I| |1|                           Gx+Hy+I   Gx+Hy+I
    /// ```
    ///
    /// `src` and `dst` may point to the same storage.
    ///
    /// - `dst` storage where the transformed vectors are written
    /// - `src` storage where the vectors are read from
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_mapVectors>
    pub fn map_vectors(&self, dst: &mut [Vector], src: &[Vector]) {
        assert!(dst.len() >= src.len());
        unsafe {
            sb::C_SkMatrix_mapVectors(
                self.native(),
                dst.native_mut().as_mut_ptr(),
                src.native().as_ptr(),
                src.len(),
            )
        }
    }

    /// Maps the vector array in place, multiplying each vector by the matrix, treating the matrix
    /// translation as zero.
    ///
    /// - `vecs` vectors to transform, and storage for the mapped vectors
    pub fn map_vectors_inplace(&self, vecs: &mut [Vector]) {
        let ptr = vecs.native_mut().as_mut_ptr();
        unsafe { sb::C_SkMatrix_mapVectors(self.native(), ptr, ptr, vecs.len()) }
    }

    /// Returns the vector multiplied by the matrix, treating the matrix translation as zero. Given:
    ///
    /// ```text
    ///              | A B 0 |         | dx |
    ///     Matrix = | D E 0 |,  vec = | dy |
    ///              | G H I |         |  1 |
    /// ```
    ///
    /// the result vector is computed as:
    ///
    /// ```text
    ///                |A B 0| |dx|                                        A*dx+B*dy     D*dx+E*dy
    ///     Matrix * vec = |D E 0| |dy| = |A*dx+B*dy D*dx+E*dy G*dx+H*dy+I| = ----------- , -----------
    ///                |G H I| | 1|                                       G*dx+H*dy+I   G*dx+H*dy+I
    /// ```
    ///
    /// - `vec` vector to map
    pub fn map_vector(&self, vec: impl Into<Vector>) -> Vector {
        let mut vec = vec.into();
        self.map_vectors_inplace(slice::from_mut(&mut vec));
        vec
    }

    /// Sets `dst` to the bounds of the `src` corners mapped by the matrix. Returns true if the
    /// mapped corners are the `dst` corners.
    ///
    /// The returned value is the same as calling [`Self::rect_stays_rect()`].
    ///
    /// - `src` rectangle to map
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_mapRect>
    pub fn map_rect(&self, src: impl AsRef<Rect>) -> (Rect, bool) {
        let mut dst = Rect::default();
        let rect_stays_rect = unsafe {
            self.native()
                .mapRect(dst.native_mut(), src.as_ref().native())
        };
        (dst, rect_stays_rect)
    }

    /// Maps the four corners of `rect` to a quad. Points are mapped by multiplying each rect corner
    /// by the matrix. The rect corner is processed in this order: (`rect.left`, `rect.top`),
    /// (`rect.right`, `rect.top`), (`rect.right`, `rect.bottom`), (`rect.left`, `rect.bottom`).
    ///
    /// `rect` may be empty: `rect.left` may be greater than or equal to `rect.right`; `rect.top` may
    /// be greater than or equal to `rect.bottom`.
    ///
    /// This does not perform perspective clipping (as that might result in more than 4 points, so
    /// results are suspect if the matrix contains perspective).
    ///
    /// - `rect` rectangle to map
    pub fn map_rect_to_quad(&self, rect: impl AsRef<Rect>) -> [Point; 4] {
        let mut quad = rect.as_ref().to_quad(None);
        self.map_points_inplace(quad.as_mut());
        quad
    }

    /// Sets the result to the bounds of the `src` corners mapped by the matrix. If the matrix
    /// contains elements other than scale or translate, returns `None` (asserts if `SK_DEBUG` is
    /// defined; otherwise, results are undefined).
    ///
    /// - `src` rectangle to map
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_mapRectScaleTranslate>
    pub fn map_rect_scale_translate(&self, src: impl AsRef<Rect>) -> Option<Rect> {
        if self.is_scale_translate() {
            let mut rect = Rect::default();
            unsafe {
                self.native()
                    .mapRectScaleTranslate(rect.native_mut(), src.as_ref().native())
            };
            Some(rect)
        } else {
            None
        }
    }

    /// Returns the geometric mean radius of the ellipse formed by constructing a circle of size
    /// `radius`, and mapping the constructed circle with the matrix. The result squared is equal to
    /// the major axis length times the minor axis length.
    ///
    /// The result is not meaningful if the matrix contains perspective elements.
    ///
    /// - `radius` circle size to map
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_mapRadius>
    pub fn map_radius(&self, radius: scalar) -> Option<scalar> {
        if !self.has_perspective() {
            Some(unsafe { self.native().mapRadius(radius) })
        } else {
            None
        }
    }

    #[deprecated(since = "0.27.0", note = "removed without replacement")]
    pub fn is_fixed_step_in_x(&self) -> ! {
        unimplemented!("removed without replacement")
    }

    #[deprecated(since = "0.27.0", note = "removed without replacement")]
    pub fn fixed_step_in_x(&self, _y: scalar) -> ! {
        unimplemented!("removed without replacement")
    }

    #[deprecated(since = "0.27.0", note = "removed without replacement")]
    pub fn cheap_equal_to(&self, _other: &Matrix) -> ! {
        unimplemented!("removed without replacement")
    }

    /// Writes a text representation of the matrix to standard output. Floating point values are
    /// written with limited precision; it may not be possible to reconstruct the original matrix
    /// from the output.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_dump>
    pub fn dump(&self) {
        unsafe { self.native().dump() }
    }

    /// Returns the minimum scaling factor of the matrix by decomposing the scaling and skewing
    /// elements.
    ///
    /// Returns -1 if the scale factor overflows or the matrix contains perspective.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_getMinScale>
    pub fn min_scale(&self) -> scalar {
        unsafe { self.native().getMinScale() }
    }

    /// Returns the maximum scaling factor of the matrix by decomposing the scaling and skewing
    /// elements.
    ///
    /// Returns -1 if the scale factor overflows or the matrix contains perspective.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_getMaxScale>
    pub fn max_scale(&self) -> scalar {
        unsafe { self.native().getMaxScale() }
    }

    /// Sets the first element of the result to the minimum scaling factor, and the second to the
    /// maximum scaling factor. Scaling factors are computed by decomposing the matrix scaling and
    /// skewing elements.
    ///
    /// Returns a tuple of the minimum and maximum scale factors if they are found; otherwise,
    /// returns undefined values.
    #[must_use]
    pub fn min_max_scales(&self) -> (scalar, scalar) {
        let mut r: [scalar; 2] = Default::default();
        unsafe { self.native().getMinMaxScales(r.as_mut_ptr()) };
        #[allow(clippy::tuple_array_conversions)]
        (r[0], r[1])
    }

    /// Decomposes the matrix into scale components and whatever remains. Returns `None` if the
    /// matrix could not be decomposed.
    ///
    /// Sets `scale` to the portion of the matrix that scales axes. Sets `remaining` to the matrix
    /// with scaling factored out. `remaining` may be passed as `None` to determine if the matrix
    /// can be decomposed without computing the remainder.
    ///
    /// Returns the scale components if they are found. The scale and `remaining` are unchanged if
    /// the matrix contains perspective, the scale factors are not finite, or are nearly zero.
    ///
    /// On success: `Matrix = remaining * scale`.
    ///
    /// - `remaining` matrix without scaling; may be `None`
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_decomposeScale>
    pub fn decompose_scale(&self, mut remaining: Option<&mut Matrix>) -> Option<Size> {
        let mut size = Size::default();
        unsafe {
            self.native()
                .decomposeScale(size.native_mut(), remaining.native_ptr_or_null_mut())
        }
        .then_some(size)
    }

    /// Returns a reference to the const identity matrix. The returned matrix is set to:
    ///
    /// ```text
    /// | 1 0 0 |
    /// | 0 1 0 |
    /// | 0 0 1 |
    /// ```
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_I>
    pub fn i() -> &'static Matrix {
        &IDENTITY
    }

    /// Returns a reference to a const matrix with invalid values. The returned matrix is set to:
    ///
    /// ```text
    /// | SK_ScalarMax SK_ScalarMax SK_ScalarMax |
    /// | SK_ScalarMax SK_ScalarMax SK_ScalarMax |
    /// | SK_ScalarMax SK_ScalarMax SK_ScalarMax |
    /// ```
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Matrix_InvalidMatrix>
    pub fn invalid_matrix() -> &'static Matrix {
        Self::from_native_ref(unsafe { &*sb::C_SkMatrix_InvalidMatrix() })
    }

    /// Returns matrix `a` multiplied by matrix `b`.
    ///
    /// Given:
    ///
    /// ```text
    ///         | A B C |      | J K L |
    ///     a = | D E F |, b = | M N O |
    ///         | G H I |      | P Q R |
    /// ```
    ///
    /// the result is:
    ///
    /// ```text
    ///             | A B C |   | J K L |   | AJ+BM+CP AK+BN+CQ AL+BO+CR |
    ///     a * b = | D E F | * | M N O | = | DJ+EM+FP DK+EN+FQ DL+EO+FR |
    ///             | G H I |   | P Q R |   | GJ+HM+IP GK+HN+IQ GL+HO+IR |
    /// ```
    ///
    /// - `a` matrix on the left side of the multiply expression
    /// - `b` matrix on the right side of the multiply expression
    pub fn concat(a: &Matrix, b: &Matrix) -> Matrix {
        let mut m = Matrix::new_identity();
        unsafe { m.native_mut().setConcat(a.native(), b.native()) };
        m
    }

    /// Sets the internal cache to the unknown state. Use to force an update after repeated
    /// modifications to matrix element references returned by indexing.
    pub fn dirty_matrix_type_cache(&mut self) {
        self.native_mut().fTypeMask = 0x80;
    }

    /// Initializes the matrix with scale and translate elements.
    ///
    /// ```text
    /// | sx  0 tx |
    /// |  0 sy ty |
    /// |  0  0  1 |
    /// ```
    ///
    /// - `s` horizontal and vertical scale factors to store
    /// - `t` horizontal and vertical translation to store
    pub fn set_scale_translate(&mut self, s: (scalar, scalar), t: impl Into<Vector>) -> &mut Self {
        *self = Self::scale_translate(s, t);
        self
    }

    /// Returns true if all elements of the matrix are finite. Returns false if any element is
    /// infinity, or NaN.
    pub fn is_finite(&self) -> bool {
        unsafe { sb::C_SkMatrix_isFinite(self.native()) }
    }

    pub const fn new_identity() -> Self {
        Self::new()
    }
}

impl IndexGet for Matrix {}
impl IndexSet for Matrix {}

pub const IDENTITY: Matrix = Matrix::new_identity();

#[cfg(test)]
mod tests {
    use super::{AffineMember, Matrix, TypeMask};
    use crate::prelude::*;

    #[test]
    fn test_get_set_trait_compilation() {
        let mut m = Matrix::new_identity();
        let _x = m.get(AffineMember::ScaleX);
        m.set(AffineMember::ScaleX, 1.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_tuple_to_vector() {
        let mut m = Matrix::new_identity();
        m.set_translate((10.0, 11.0));
        assert_eq!(10.0, m.translate_x());
        assert_eq!(11.0, m.translate_y());
    }

    #[test]
    fn setting_a_matrix_component_recomputes_typemask() {
        let mut m = Matrix::default();
        assert_eq!(TypeMask::IDENTITY, m.get_type());
        m.set_persp_x(0.1);
        assert_eq!(
            TypeMask::TRANSLATE | TypeMask::SCALE | TypeMask::AFFINE | TypeMask::PERSPECTIVE,
            m.get_type()
        );
    }
}
