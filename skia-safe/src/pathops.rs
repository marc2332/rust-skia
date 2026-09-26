//! Boolean operations on [`Path`]s, as well as a builder for performing a series of path
//! operations.
//! Wrapper for pathops/SkPathOps.h
use crate::{Path, Rect, prelude::*};
use skia_bindings::{self as sb, SkOpBuilder};
use std::fmt;

/// The logical operations that can be performed when combining two paths.
///
/// Variants:
/// - [`PathOp::Difference`]: subtract the op path from the first path.
/// - [`PathOp::Intersect`]: intersect the two paths.
/// - [`PathOp::Union`]: union (inclusive-or) the two paths.
/// - [`PathOp::XOR`]: exclusive-or the two paths.
/// - [`PathOp::ReverseDifference`]: subtract the first path from the op path.
pub type PathOp = skia_bindings::SkPathOp;
variant_name!(PathOp::XOR);

// TODO: I am not so sure if we should export these global functions.

/// Set the resulting path to the result of applying the operator to `one` and `two`:
/// this = (one op two). The resulting path will be constructed from non-overlapping contours. The
/// curve order is reduced where possible so that cubics may be turned into quadratics, and
/// quadratics maybe turned into lines.
///
/// Returns `Some` if the operation was able to produce a result; otherwise, the result is
/// unmodified.
///
/// - `one` The first operand (for difference, the minuend)
/// - `two` The second operand (for difference, the subtrahend)
/// - `op` The operator to apply.
///
/// Returns `Some` if the operation succeeded.
pub fn op(one: &Path, two: &Path, op: PathOp) -> Option<Path> {
    Path::try_construct(|p| unsafe { sb::C_SkPathOps_Op(one.native(), two.native(), op, p) })
}

/// Return a path with a set of non-overlapping contours that describe the same area as the
/// original path. The curve order is reduced where possible so that cubics may be turned into
/// quadratics, and quadratics maybe turned into lines.
///
/// - `path` The path to simplify.
///
/// Returns the simplified path, or `None` on failure.
pub fn simplify(path: &Path) -> Option<Path> {
    Path::try_construct(|p| unsafe { sb::C_SkPathOps_Simplify(path.native(), p) })
}

/// Set the resulting rectangle to the tight bounds of the path.
///
/// - `path` The path measured.
///
/// Returns the tight bounds of the path, if they could be computed.
#[deprecated(
    since = "0.83.0",
    note = "Use Path::compute_tight_bounds() and test if the resulting Rect::is_finite()"
)]
pub fn tight_bounds(path: &Path) -> Option<Rect> {
    let rect = path.compute_tight_bounds();
    rect.is_finite().then_some(rect)
}

/// Returns a path with fill type winding to area equivalent to the input. Does not detect if path
/// contains contours which contain self-crossings or cross other contours; in these cases, may
/// return a result even though it does not fill same area as the input.
///
/// If it fails to compute a result, returns `None`.
///
/// - `path` The path typically with fill type set to even odd.
pub fn as_winding(path: &Path) -> Option<Path> {
    Path::try_construct(|p| unsafe { sb::C_SkPathOps_AsWinding(path.native(), p) })
}

/// Perform a series of path operations, optimized for unioning many paths together.
pub type OpBuilder = Handle<SkOpBuilder>;
unsafe_send_sync!(OpBuilder);

impl NativeDrop for SkOpBuilder {
    fn drop(&mut self) {
        unsafe { sb::C_SkOpBuilder_destruct(self) }
    }
}

impl Default for Handle<SkOpBuilder> {
    fn default() -> Self {
        Self::construct(|opb| unsafe { sb::C_SkOpBuilder_Construct(opb) })
    }
}

impl fmt::Debug for OpBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpBuilder").finish()
    }
}

impl OpBuilder {
    /// Add one or more paths and their operand. The builder is empty before the first path is
    /// added, so the result of a single add is (emptyPath OP path).
    ///
    /// - `path` The second operand.
    /// - `operator` The operator to apply to the existing and supplied paths.
    pub fn add(&mut self, path: &Path, operator: PathOp) -> &mut Self {
        unsafe {
            self.native_mut().add(path.native(), operator);
        }
        self
    }

    /// Computes the sum of all paths and operands, and resets the builder to its initial state.
    ///
    /// Returns the product of the operands, `None` on failure.
    pub fn resolve(&mut self) -> Option<Path> {
        Path::try_construct(|p| unsafe { sb::C_SkOpBuilder_resolve(self.native_mut(), p) })
    }
}

impl Path {
    pub fn op(&self, path: &Path, path_op: PathOp) -> Option<Self> {
        op(self, path, path_op)
    }

    pub fn simplify(&self) -> Option<Self> {
        simplify(self)
    }

    #[deprecated(
        since = "0.83.0",
        note = "Use Path::compute_tight_bounds() and test if the resulting Rect::is_finite()"
    )]
    pub fn tight_bounds(&self) -> Option<Rect> {
        #[allow(deprecated)]
        tight_bounds(self)
    }

    pub fn as_winding(&self) -> Option<Path> {
        as_winding(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Path, PathBuilder, PathOp, Rect};

    #[test]
    fn test_tight_bounds() {
        let mut builder = PathBuilder::new();
        builder.add_rect(
            Rect::from_point_and_size((10.0, 10.0), (10.0, 10.0)),
            None,
            None,
        );
        builder.add_rect(
            Rect::from_point_and_size((15.0, 15.0), (10.0, 10.0)),
            None,
            None,
        );
        let path = builder.detach();

        let tight_bounds: Rect = Rect::from_point_and_size((10.0, 10.0), (15.0, 15.0));
        assert_eq!(path.compute_tight_bounds(), tight_bounds);
    }

    #[test]
    fn test_union() {
        let path = Path::rect(Rect::from_point_and_size((10.0, 10.0), (10.0, 10.0)), None);
        let path2 = Path::rect(Rect::from_point_and_size((15.0, 15.0), (10.0, 10.0)), None);
        let union = path.op(&path2, PathOp::Union).unwrap();
        let expected: Rect = Rect::from_point_and_size((10.0, 10.0), (15.0, 15.0));
        assert_eq!(union.compute_tight_bounds(), expected);
    }

    #[test]
    fn test_intersect() {
        let path = Path::rect(Rect::from_point_and_size((10.0, 10.0), (10.0, 10.0)), None);
        let path2 = Path::rect(Rect::from_point_and_size((15.0, 15.0), (10.0, 10.0)), None);
        let intersected = path.op(&path2, PathOp::Intersect).unwrap();
        let expected: Rect = Rect::from_point_and_size((15.0, 15.0), (5.0, 5.0));
        assert_eq!(intersected.compute_tight_bounds(), expected);
    }
}
