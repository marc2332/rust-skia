use crate::prelude::*;
use skia_bindings::{self as sb, SkDataTable, SkRefCntBase};
use std::{
    ffi::{CStr, c_void},
    fmt, mem,
    ops::Index,
};

/// Like [`crate::Data`], [`DataTable`] holds an immutable data buffer. The data buffer is
/// organized into a table of entries, each with a length, so the entries are not required to all
/// be the same size.
pub type DataTable = RCHandle<SkDataTable>;
unsafe_send_sync!(DataTable);
require_type_equality!(sb::SkDataTable_INHERITED, sb::SkRefCnt);

impl NativeRefCountedBase for SkDataTable {
    type Base = SkRefCntBase;
}

impl Index<usize> for DataTable {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.at(index)
    }
}

impl fmt::Debug for DataTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataTable")
            .field("count", &self.count())
            .finish()
    }
}

impl DataTable {
    /// Returns true if the table is empty (i.e. has no entries).
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns the number of entries in the table. 0 for an empty table.
    pub fn count(&self) -> usize {
        unsafe { sb::C_SkDataTable_count(self.native()) }
            .try_into()
            .unwrap()
    }

    /// Returns the size of the `index`'th entry in the table. The caller must ensure that `index`
    /// is valid for this table.
    ///
    /// - `index` entry index
    pub fn at_size(&self, index: usize) -> usize {
        assert!(index < self.count());
        unsafe { self.native().atSize(index.try_into().unwrap()) }
    }

    /// Returns the data of the `index`'th entry in the table. The caller must ensure that `index`
    /// is valid for this table.
    ///
    /// - `index` entry index
    pub fn at(&self, index: usize) -> &[u8] {
        unsafe { self.at_t(index) }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn at_t<T: Copy>(&self, index: usize) -> &[T] {
        unsafe {
            assert!(index < self.count());
            let mut size = usize::default();
            let ptr = self.native().at(index.try_into().unwrap(), &mut size);
            let element_size = mem::size_of::<T>();
            assert_eq!(size % element_size, 0);
            let elements = size / element_size;
            safer::from_raw_parts(ptr as _, elements)
        }
    }

    /// Returns the `index`'th entry as a c-string, and assumes that the trailing null byte had
    /// been copied into the table as well.
    ///
    /// - `index` entry index
    pub fn at_str(&self, index: usize) -> &CStr {
        let bytes = self.at(index);
        CStr::from_bytes_with_nul(bytes).unwrap()
    }

    /// Returns a new empty table.
    pub fn new_empty() -> Self {
        DataTable::from_ptr(unsafe { sb::C_SkDataTable_MakeEmpty() }).unwrap()
    }

    /// Returns a new table that contains a copy of the data stored in each slice.
    ///
    /// - `slices` slices to copy into the table
    pub fn from_slices(slices: &[&[u8]]) -> Self {
        let ptrs: Vec<*const c_void> = slices.iter().map(|s| s.as_ptr() as _).collect();
        let sizes: Vec<usize> = slices.iter().map(|s| s.len()).collect();
        unsafe {
            DataTable::from_ptr(sb::C_SkDataTable_MakeCopyArrays(
                ptrs.as_ptr(),
                sizes.as_ptr(),
                slices.len().try_into().unwrap(),
            ))
            .unwrap()
        }
    }

    /// Returns a new table that contains a copy of the data in `slice`.
    ///
    /// - `slice` contiguous array of data for all elements to be copied
    pub fn from_slice<T: Copy>(slice: &[T]) -> Self {
        unsafe {
            DataTable::from_ptr(sb::C_SkDataTable_MakeCopyArray(
                slice.as_ptr() as _,
                mem::size_of::<T>(),
                slice.len().try_into().unwrap(),
            ))
            .unwrap()
        }
    }

    // TODO: wrap MakeArrayProc()

    pub fn iter(&self) -> Iter {
        Iter {
            table: self,
            count: self.count(),
            current: 0,
        }
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    table: &'a DataTable,
    count: usize,
    current: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.count {
            let r = Some(self.table.at(self.current));
            self.current += 1;
            r
        } else {
            None
        }
    }
}
