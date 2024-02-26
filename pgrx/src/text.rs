use crate::datum::{BorrowDatum, Datum};
use crate::pgrx_sql_entity_graph::metadata::{
    ArgumentError, Returns, ReturnsError, SqlMapping, SqlTranslatable,
};
use crate::{pg_sys, varlena};
use core::ops::{Deref, DerefMut};
use core::ptr;

/// A more strongly-typed representation of a Postgres string, AKA `TEXT`.
/// A pointer to this points to a byte array, which includes a variable-length header
/// and an unsized data field which is... often but not always UTF-8.
#[repr(transparent)]
pub struct Text([u8]);

// TODO(0.12.0): strip this and make Text forward its impl to BStr fn instead
impl Deref for Text {
    type Target = str;
    fn deref(&self) -> &str {
        let self_ptr = self as *const Text as *const pg_sys::varlena;
        unsafe { varlena::text_to_rust_str(self_ptr).unwrap() }
    }
}

// TODO(0.12.0): strip this and make Text forward its impl to BStr fn instead
impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut str {
        let self_ptr = self as *mut Text as *mut pg_sys::varlena;
        unsafe {
            let len = varlena::varsize_any_exhdr(self_ptr);
            let data = varlena::vardata_any(self_ptr);

            &mut *(ptr::slice_from_raw_parts_mut(data as *mut u8, len) as *mut str)
        }
    }
}

unsafe impl BorrowDatum for Text {
    unsafe fn borrow_from<'dat>(datum: &'dat Datum<'_>) -> &'dat Self {
        let ptr = datum as *const Datum<'_> as *const *const pg_sys::varlena;
        unsafe {
            let ptr = *ptr;
            let len = varlena::varsize_any(ptr);
            &*(ptr::slice_from_raw_parts(ptr as *const u8, len) as *const Text)
        }
    }
    unsafe fn borrow_mut_from<'dat>(datum: &'dat mut Datum<'_>) -> &'dat mut Self {
        let ptr = datum as *mut Datum<'_> as *mut *mut pg_sys::varlena;
        unsafe {
            let ptr = *ptr;
            let len = varlena::varsize_any(ptr);
            &mut *(ptr::slice_from_raw_parts(ptr as *mut u8, len) as *mut Text)
        }
    }
}

unsafe impl<'dat> SqlTranslatable for &'dat Text {
    fn argument_sql() -> Result<SqlMapping, ArgumentError> {
        Ok(SqlMapping::literal("TEXT"))
    }
    fn return_sql() -> Result<Returns, ReturnsError> {
        Ok(Returns::One(SqlMapping::literal("TEXT")))
    }
}