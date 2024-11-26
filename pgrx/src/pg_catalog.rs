//LICENSE Portions Copyright 2019-2021 ZomboDB, LLC.
//LICENSE
//LICENSE Portions Copyright 2021-2023 Technology Concepts & Design, Inc.
//LICENSE
//LICENSE Portions Copyright 2023-2023 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.

use crate as pgrx;
use pg_sys::Oid as Regproc;
use pg_sys::Oid;
use pgrx::datum::Array;
use pgrx::pg_sys;
use std::ffi::{c_char, CStr};
use std::ptr::NonNull;

trait GetStruct<T> {
    /// # Safety
    ///
    /// `raw` must be a refernece to a struct member in `FormData_*`.
    unsafe fn get_struct(raw: T) -> Self;
}

impl<'a, T: Copy> GetStruct<&'a T> for T {
    unsafe fn get_struct(raw: &'a T) -> Self {
        *raw
    }
}

// PostgreSQL comments:
// Representation of a Name: effectively just a C string, but null-padded to exactly NAMEDATALEN bytes.
// The use of a struct is historical.
impl<'a> GetStruct<&'a pg_sys::nameData> for &'a CStr {
    unsafe fn get_struct(raw: &pg_sys::nameData) -> Self {
        assert_eq!(raw.data[63], 0);
        unsafe { CStr::from_ptr(raw.data.as_ptr()) }
    }
}

// A variable-sized type that is always the last struct member in `FormData_*`.
impl<'a> GetStruct<&'a pg_sys::int2vector> for &'a [i16] {
    unsafe fn get_struct(raw: &'a pg_sys::int2vector) -> Self {
        let len = raw.dim1;
        // SAFETY: we trust `len` since it's passed from PostgreSQL and we cannot check it anyway
        unsafe { raw.values.as_slice(len as usize) }
    }
}

// A variable-sized type that is always the last struct member in `FormData_*`.
impl<'a> GetStruct<&'a pg_sys::oidvector> for &'a [Oid] {
    unsafe fn get_struct(raw: &'a pg_sys::oidvector) -> Self {
        let len = raw.dim1;
        // SAFETY: we trust `len` since it's passed from PostgreSQL and we cannot check it anyway
        unsafe { raw.values.as_slice(len as usize) }
    }
}

#[inline]
unsafe fn get_struct<T>(inner: &pg_sys::HeapTupleData) -> &T {
    debug_assert!(std::any::type_name::<T>().contains("FormData_"));
    unsafe {
        // PostgreSQL macro `GETSTRUCT(tup)` expands to `((char *)((tup)->t_data) + (tup)->t_data->t_hoff)`
        let start = inner.t_data.cast::<u8>();
        let offset: u8 = (*inner.t_data).t_hoff;
        // So `s` is deferenced `GETSTRUCT(tup)`.
        &*start.add(offset as usize).cast::<T>()
    }
}

#[inline]
unsafe fn get_attr<T: pgrx::datum::FromDatum>(
    inner: *const pg_sys::HeapTupleData,
    cache_id: i32,
    attribute: u32,
) -> Option<T> {
    unsafe {
        let arg_tup = inner.cast_mut();
        let arg_att = attribute as i16;
        let mut is_null = true;
        let datum = pg_sys::SysCacheGetAttr(cache_id, arg_tup, arg_att, &mut is_null);
        T::from_datum(datum, is_null)
    }
}

#[inline]
unsafe fn syscache_len(inner: NonNull<pg_sys::CatCList>) -> usize {
    unsafe {
        use pg_sys::CatCList;
        let inner: &CatCList = inner.as_ref();
        let n: i32 = inner.n_members;
        n as usize
    }
}

#[inline]
unsafe fn syscache_get<'a>(
    inner: NonNull<pg_sys::CatCList>,
    i: usize,
) -> Option<&'a pg_sys::HeapTupleData> {
    unsafe {
        use pg_sys::{CatCList, CatCTup, HeapTupleData};
        let inner: &CatCList = inner.as_ref();
        let n: i32 = inner.n_members;
        let slice: &[*mut CatCTup] = inner.members.as_slice(n as usize);
        let member: *mut CatCTup = *slice.get(i)?;
        let tuple: &HeapTupleData = &(*member).tuple;
        Some(tuple)
    }
}

macro_rules! define_character {
    {
        $(#[$character_meta:meta])* character ($table:ident, $column:ident) { $($(#[$variant_meta:meta])* ($variant:ident, $variant_value:literal))* }
    } => {
        paste::paste! {
            #[doc = concat!("Enum for pg_catalog.", stringify!($table), ".", stringify!($column), ".")]
            $(#[$character_meta])*
            #[non_exhaustive]
            #[repr(u8)]
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
            pub enum [<$table:camel $column:camel>] {
                $($(#[$variant_meta])* $variant = $variant_value),*
            }

            impl [<$table:camel $column:camel>] {
                pub fn new(value: c_char) -> Option<Self> {
                    match value as u8 {
                        $($variant_value => Some(Self::$variant),)*
                        _ => None,
                    }
                }
                pub fn as_c_char(self) -> c_char {
                    self as u8 as c_char
                }
            }

            impl pgrx::datum::FromDatum for [<$table:camel $column:camel>] {
                unsafe fn from_polymorphic_datum(
                    datum: pg_sys::Datum,
                    is_null: bool,
                    _: Oid,
                ) -> Option<Self>
                where
                    Self: Sized,
                {
                    if is_null {
                        None
                    } else {
                        Some(Self::new(datum.value() as _).expect("unrecognized value"))
                    }
                }
            }

            impl pgrx::datum::IntoDatum for [<$table:camel $column:camel>] {
                fn into_datum(self) -> std::option::Option<pg_sys::Datum> {
                    Some(pg_sys::Datum::from(self as i8))
                }
                fn type_oid() -> pg_sys::Oid {
                    pg_sys::CHAROID
                }
            }

            unsafe impl pgrx::datum::UnboxDatum for [<$table:camel $column:camel>] {
                type As<'src> = Self;
                #[inline]
                unsafe fn unbox<'src>(datum: pgrx::datum::Datum<'src>) -> Self::As<'src>
                where
                    Self: 'src,
                {
                    Self::new(datum.sans_lifetime().value() as c_char).expect("unrecognized value")
                }
            }

            impl<'a> GetStruct<&'a c_char> for [<$table:camel $column:camel>] {
                unsafe fn get_struct(raw: &c_char) -> Self {
                    Self::new(*raw).expect("unrecognized value")
                }
            }
        }
    };
}

macro_rules! define_column {
    {
        $(#[$column_meta:meta])* column ($table:ident, $column:ident, $column_type:ty, $(#[$character_meta:meta])* character { $($(#[$variant_meta:meta])* ($variant:ident, $variant_value:literal))* } $($x:tt)*)
    } => {
        paste::paste! {
            define_character! { $(#[$character_meta])* character ($table, $column) { $($(#[$variant_meta])* ($variant, $variant_value))* } }
            define_column! { $(#[$column_meta])* column ($table, $column, $column_type $($x)*) }
        }
    };
    {
        $(#[$column_meta:meta])* column ($table:ident, $column:ident, $column_type:ty, get_struct)
    } => {
        paste::paste! {
            impl [<$table:camel>]<'_> {
                $(#[$column_meta])*
                pub fn $column(&self) -> $column_type {
                    unsafe {
                        let s = get_struct::<pg_sys::[<FormData_ $table>]>(self.inner);
                        // SAFETY: we get `s` from PostgreSQL functions
                        GetStruct::get_struct(&s.$column)
                    }
                }
            }
        }
    };
    {
        $(#[$column_meta:meta])* column ($table:ident, $column:ident, $column_type:ty, get_attr)
    } => {
        paste::paste! {
            impl [<$table:camel>]<'_> {
                $(#[$column_meta])*
                /// # Panics
                ///
                /// This function panics if the catalog wrapper is created by `Self::new` but there is no `cache_id` provided.
                pub fn $column(&self) -> Option<$column_type> {
                    if let Some(cache_id) = self.cache_id {
                        unsafe { get_attr::<$column_type>(self.inner, cache_id, pg_sys::[<Anum_ $table _ $column>]) }
                    } else {
                        panic!("`cache_id` is not provided")
                    }
                }
            }
        }
    };
    {
        $(#[$column_meta:meta])* column ($table:ident, $column:ident, $column_type:ty, get_attr_notnull)
    } => {
        paste::paste! {
            impl [<$table:camel>]<'_> {
                $(#[$column_meta])*
                /// # Panics
                ///
                /// This function panics if the catalog wrapper is created by `Self::new` but there is no `cache_id` provided.
                pub fn $column(&self) -> $column_type {
                    if let Some(cache_id) = self.cache_id {
                        unsafe { get_attr::<$column_type>(self.inner, cache_id, pg_sys::[<Anum_ $table _ $column>]).unwrap() }
                    } else {
                        panic!("`cache_id` is not provided")
                    }
                }
            }
        }
    };
}

macro_rules! define_catalog {
    {
        $(#[$table_meta:meta])* catalog ($table:ident) { $($(#[$column_meta:meta])* ($column:ident, $column_type:ty, $($x:tt)*))* }
    } => {
        paste::paste!{
            #[doc = concat!("Safe wrapper for pg_catalog.", stringify!($table), ".")]
            $(#[$table_meta])*
            pub struct [<$table:camel>]<'a> {
                inner: &'a pg_sys::HeapTupleData,
                #[allow(dead_code)]
                cache_id: Option<i32>,
            }

            impl<'a> [<$table:camel>]<'a> {
                /// Create a catalog wrapper by a heap tuple.
                ///
                /// # Safety
                ///
                /// If `cache_id` is provided, `cache_id` must be an ID for a `syscache` for this catalog.
                pub unsafe fn new(inner: &'a pg_sys::HeapTupleData, cache_id: Option<i32>) -> Self {
                    Self {
                        inner,
                        cache_id,
                    }
                }
            }

            #[doc = concat!("The search result for pg_catalog.", stringify!($table), ".")]
            pub struct [<$table:camel Search>] {
                inner: Option<NonNull<pg_sys::HeapTupleData>>,
                cache_id: i32,
            }

            impl [<$table:camel Search>] {
                /// Returns true if the search result is empty.
                pub fn is_empty(&self) -> bool {
                    self.inner.is_none()
                }
                /// Get the search result.
                pub fn get(&self) -> Option<[<$table:camel>]> {
                    unsafe {
                        Some([<$table:camel>] {
                            inner: self.inner?.as_ref(),
                            cache_id: Some(self.cache_id),
                        })
                    }
                }
            }

            impl Drop for [<$table:camel Search>] {
                fn drop(&mut self) {
                    unsafe {
                        if let Some(inner) = self.inner {
                            pg_sys::ReleaseSysCache(inner.as_ptr());
                        }
                    }
                }
            }

            #[doc = concat!("The search results for pg_catalog.", stringify!($table), ".")]
            pub struct [<$table:camel SearchList>] {
                inner: NonNull<pg_sys::CatCList>,
                cache_id: i32,
            }

            impl [<$table:camel SearchList>] {
                /// Returns the number of the search results.
                pub fn len(&self) -> usize {
                    unsafe {
                        syscache_len(self.inner)
                    }
                }
                /// Returns true if the search result is empty.
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
                /// Get the `i`-th search result.
                pub fn get(&self, i: usize) -> Option<[<$table:camel>]> {
                    unsafe {
                        Some([<$table:camel>] {
                            inner: syscache_get(self.inner, i)?,
                            cache_id: Some(self.cache_id)
                        })
                    }
                }
            }

            impl Drop for [<$table:camel SearchList>] {
                fn drop(&mut self) {
                    unsafe { pg_sys::ReleaseCatCacheList(self.inner.as_ptr()) }
                }
            }

            $(define_column! { $(#[$column_meta])* column ($table, $column, $column_type, $($x)*) })*
        }
    };
}

macro_rules! define_cache {
    {
        cache ($cache:ident, $table:ident) {
            ($_0:ident, $_0_type:ty)
        }
    } => {
        paste::paste!{
            impl<'a> [<$table:camel>]<'a> {
                /// Search for a row by providing arguments needed by cache.
                pub fn [<search _ $cache>]($_0: $_0_type) -> Option<[<$table:camel Search>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe { pg_sys::SearchSysCache1(cache_id, $_0.into_datum()?) };
                    let inner = NonNull::new(entry);
                    Some([<$table:camel Search>] { inner, cache_id })
                }
            }
        }
    };
    {
        cache ($cache:ident, $table:ident) {
            ($_0:ident, $_0_type:ty)
            ($_1:ident, $_1_type:ty)
        }
    } => {
        paste::paste!{
            impl<'a> [<$table:camel>]<'a> {
                /// Search for a row by providing arguments needed by cache.
                pub fn [<search _ $cache>]($_0: $_0_type, $_1: $_1_type) -> Option<[<$table:camel Search>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCache2(
                            cache_id,
                            $_0.into_datum()?,
                            $_1.into_datum()?,
                        )
                    };
                    let inner = NonNull::new(entry);
                    Some([<$table:camel Search>] { inner, cache_id })
                }
                /// Search for rows by providing first 1 argument needed by cache.
                pub fn [<search _list_ $cache _1>]($_0: $_0_type) -> Option<[<$table:camel SearchList>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCacheList(
                            cache_id,
                            1,
                            $_0.into_datum()?,
                            0.into(),
                            0.into(),
                        )
                    };
                    let inner = NonNull::new(entry).unwrap();
                    Some([<$table:camel SearchList>] { inner, cache_id })
                }
            }
        }
    };
    {
        cache ($cache:ident, $table:ident) {
            ($_0:ident, $_0_type:ty)
            ($_1:ident, $_1_type:ty)
            ($_2:ident, $_2_type:ty)
        }
    } => {
        paste::paste!{
            impl<'a> [<$table:camel>]<'a> {
                /// Search for a row by providing arguments needed by cache.
                pub fn [<search _ $cache>]($_0: $_0_type, $_1: $_1_type, $_2: $_2_type) -> Option<[<$table:camel Search>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCache3(
                            cache_id,
                            $_0.into_datum()?,
                            $_1.into_datum()?,
                            $_2.into_datum()?,
                        )
                    };
                    let inner = NonNull::new(entry);
                    Some([<$table:camel Search>] { inner, cache_id })
                }
                /// Search for rows by providing first 1 argument needed by cache.
                pub fn [<search _list_ $cache _1>]($_0: $_0_type) -> Option<[<$table:camel SearchList>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCacheList(
                            cache_id,
                            1,
                            $_0.into_datum()?,
                            0.into(),
                            0.into(),
                        )
                    };
                    let inner = NonNull::new(entry).unwrap();
                    Some([<$table:camel SearchList>] { inner, cache_id })
                }
                /// Search for rows by providing first 2 arguments needed by cache.
                pub fn [<search _list_ $cache _2>]($_0: $_0_type, $_1: $_1_type) -> Option<[<$table:camel SearchList>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCacheList(
                            cache_id,
                            2,
                            $_0.into_datum()?,
                            $_1.into_datum()?,
                            0.into(),
                        )
                    };
                    let inner = NonNull::new(entry).unwrap();
                    Some([<$table:camel SearchList>] { inner, cache_id })
                }
            }
        }
    };
    {
        cache ($cache:ident, $table:ident) {
            ($_0:ident, $_0_type:ty)
            ($_1:ident, $_1_type:ty)
            ($_2:ident, $_2_type:ty)
            ($_3:ident, $_3_type:ty)
        }
    } => {
        paste::paste!{
            impl<'a> [<$table:camel>]<'a> {
                /// Search for a row by providing arguments needed by cache.
                pub fn [<search _ $cache>]($_0: $_0_type, $_1: $_1_type, $_2: $_2_type, $_3: $_3_type) -> Option<[<$table:camel Search>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCache4(
                            cache_id,
                            $_0.into_datum()?,
                            $_1.into_datum()?,
                            $_2.into_datum()?,
                            $_3.into_datum()?,
                        )
                    };
                    let inner = NonNull::new(entry);
                    Some([<$table:camel Search>] { inner, cache_id })
                }
                /// Search for rows by providing first 1 argument needed by cache.
                pub fn [<search _list_ $cache _1>]($_0: $_0_type) -> Option<[<$table:camel SearchList>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCacheList(
                            cache_id,
                            1,
                            $_0.into_datum()?,
                            0.into(),
                            0.into(),
                        )
                    };
                    let inner = NonNull::new(entry).unwrap();
                    Some([<$table:camel SearchList>] { inner, cache_id })
                }
                /// Search for rows by providing first 2 arguments needed by cache.
                pub fn [<search _list_ $cache _2>]($_0: $_0_type, $_1: $_1_type) -> Option<[<$table:camel SearchList>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCacheList(
                            cache_id,
                            2,
                            $_0.into_datum()?,
                            $_1.into_datum()?,
                            0.into(),
                        )
                    };
                    let inner = NonNull::new(entry).unwrap();
                    Some([<$table:camel SearchList>] { inner, cache_id })
                }
                /// Search for rows by providing first 3 arguments needed by cache.
                pub fn [<search _list_ $cache _3>]($_0: $_0_type, $_1: $_1_type, $_2: $_2_type) -> Option<[<$table:camel SearchList>]> {
                    use pgrx::datum::IntoDatum;
                    let cache_id = pg_sys::SysCacheIdentifier::[<$cache:upper>] as i32;
                    let entry = unsafe {
                        pg_sys::SearchSysCacheList(
                            cache_id,
                            3,
                            $_0.into_datum()?,
                            $_1.into_datum()?,
                            $_2.into_datum()?,
                        )
                    };
                    let inner = NonNull::new(entry).unwrap();
                    Some([<$table:camel SearchList>] { inner, cache_id })
                }
            }
        }
    };
}

define_catalog! {
    /// The catalog pg_am stores information about relation access methods. There is one row for each access method supported by the system. Currently, only tables and indexes have access methods.
    catalog (pg_am) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Name of the access method
        (amname, &CStr, get_struct)
        /// OID of a handler function that is responsible for supplying information about the access method
        (amhandler, Regproc, get_struct)
        (amtype, PgAmAmtype, character {
            /// t = table (including materialized views)
            (Table, b't')
            /// i = index
            (Index, b'i')
        }, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_amop stores information about operators associated with access method operator families. There is one row for each operator that is a member of an operator family. A family member can be either a search operator or an ordering operator. An operator can appear in more than one family, but cannot appear in more than one search position nor more than one ordering position within a family. (It is allowed, though unlikely, for an operator to be used for both search and ordering purposes.)
    catalog (pg_amop) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// The operator family this entry is for
        (amopfamily, Oid, get_struct)
        /// Left-hand input data type of operator
        (amoplefttype, Oid, get_struct)
        /// Right-hand input data type of operator
        (amoprighttype, Oid, get_struct)
        /// Operator strategy number
        (amopstrategy, i16, get_struct)
        /// Operator purpose
        (amoppurpose, PgAmopAmoppurpose, character {
            /// s for search
            (Search, b's')
            /// o for ordering
            (Order, b'o')
        }, get_struct)
        /// OID of the operator
        (amopopr, Oid, get_struct)
        /// Index access method operator family is for
        (amopmethod, Oid, get_struct)
        /// The B-tree operator family this entry sorts according to, if an ordering operator; zero if a search operator
        (amopsortfamily, Oid, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_amproc stores information about support functions associated with access method operator families. There is one row for each support function belonging to an operator family.
    catalog (pg_amproc) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// The operator family this entry is for
        (amprocfamily, Oid, get_struct)
        /// Left-hand input data type of associated operator
        (amproclefttype, Oid, get_struct)
        /// Right-hand input data type of associated operator
        (amprocrighttype, Oid, get_struct)
        /// Support function number
        (amprocnum, i16, get_struct)
        /// OID of the function
        (amproc, Regproc, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_class describes tables and other objects that have columns or are otherwise similar to a table. This includes indexes, sequences, views, materialized views, composite types, and TOAST tables; see relkind. Below, when we mean all of these kinds of objects we speak of “relations”. Not all of pg_class's columns are meaningful for all relation kinds.
    catalog (pg_class) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Name of the table, index, view, etc.
        (relname, &CStr, get_struct)
        /// The OID of the namespace that contains this relation
        (relnamespace, Oid, get_struct)
        /// The OID of the data type that corresponds to this table's row type, if any; zero for indexes, sequences, and toast tables, which have no pg_type entry
        (reltype, Oid, get_struct)
        /// For typed tables, the OID of the underlying composite type; zero for all other relations
        (reloftype, Oid, get_struct)
        /// Owner of the relation
        (relowner, Oid, get_struct)
        /// If this is a table or an index, the access method used (heap, B-tree, hash, etc.); otherwise zero (zero occurs for sequences, as well as relations without storage, such as views)
        (relam, Oid, get_struct)
        /// Name of the on-disk file of this relation; zero means this is a “mapped” relation whose disk file name is determined by low-level state
        (relfilenode, Oid, get_struct)
        /// The tablespace in which this relation is stored. If zero, the database's default tablespace is implied. (Not meaningful if the relation has no on-disk file.)
        (reltablespace, Oid, get_struct)
        /// Size of the on-disk representation of this table in pages (of size BLCKSZ). This is only an estimate used by the planner. It is updated by VACUUM, ANALYZE, and a few DDL commands such as CREATE INDEX.
        (relpages, i32, get_struct)
        /// Number of live rows in the table. This is only an estimate used by the planner. It is updated by VACUUM, ANALYZE, and a few DDL commands such as CREATE INDEX. If the table has never yet been vacuumed or analyzed, reltuples contains -1 indicating that the row count is unknown.
        (reltuples, f32, get_struct)
        /// Number of pages that are marked all-visible in the table's visibility map. This is only an estimate used by the planner. It is updated by VACUUM, ANALYZE, and a few DDL commands such as CREATE INDEX.
        (relallvisible, i32, get_struct)
        /// OID of the TOAST table associated with this table, zero if none. The TOAST table stores large attributes “out of line” in a secondary table.
        (reltoastrelid, Oid, get_struct)
        /// True if this is a table and it has (or recently had) any indexes
        (relhasindex, bool, get_struct)
        /// True if this table is shared across all databases in the cluster. Only certain system catalogs (such as pg_database) are shared.
        (relisshared, bool, get_struct)
        (relpersistence, PgClassRelpersistence, character {
            /// p = permanent table/sequence
            (Permanent, b'p')
            /// u = unlogged table/sequence
            (Unlogged, b'u')
            /// t = temporary table/sequence
            (Temp, b't')
        }, get_struct)
        (relkind, PgClassRelkind, character {
            /// r = ordinary table
            (Relation, b'r')
            /// i = index
            (Index, b'i')
            /// S = sequence
            (Sequence, b'S')
            /// t = TOAST table
            (Toastvalue, b't')
            /// v = view
            (View, b'v')
            /// m = materialized view
            (Matview, b'm')
            /// c = composite type
            (CompositeType, b'c')
            /// f = foreign table
            (ForeignTable, b'f')
            /// p = partitioned table
            (PartitionedTable, b'p')
            /// I = partitioned index
            (PartitionedIndex, b'I')
        }, get_struct)
        /// Number of user columns in the relation (system columns not counted). There must be this many corresponding entries in pg_attribute.
        (relnatts, i16, get_struct)
        /// Number of CHECK constraints on the table
        (relchecks, i16, get_struct)
        /// True if table has (or once had) rules
        (relhasrules, bool, get_struct)
        /// True if table has (or once had) triggers
        (relhastriggers, bool, get_struct)
        /// True if table or index has (or once had) any inheritance children or partitions
        (relhassubclass, bool, get_struct)
        /// True if table has row-level security enabled
        (relrowsecurity, bool, get_struct)
        /// True if row-level security (when enabled) will also apply to table owner
        (relforcerowsecurity, bool, get_struct)
        /// True if relation is populated (this is true for all relations other than some materialized views)
        (relispopulated, bool, get_struct)
        /// Columns used to form “replica identity” for rows
        (relreplident, PgClassRelreplident, character {
            /// d = default (primary key, if any)
            (Default, b'd')
            /// n = nothing
            (Nothing, b'n')
            /// f = all columns
            (Full, b'f')
            /// i = index with indisreplident set (same as nothing if the index used has been dropped)
            (Index, b'i')
        }, get_struct)
        /// True if table or index is a partition
        (relispartition, bool, get_struct)
        /// For new relations being written during a DDL operation that requires a table rewrite, this contains the OID of the original relation; otherwise zero. That state is only visible internally; this field should never contain anything other than zero for a user-visible relation.
        (relrewrite, Oid, get_struct)
        /// All transaction IDs before this one have been replaced with a permanent (“frozen”) transaction ID in this table. This is used to track whether the table needs to be vacuumed in order to prevent transaction ID wraparound or to allow pg_xact to be shrunk. Zero (InvalidTransactionId) if the relation is not a table.
        (relfrozenxid, u32, get_struct)
        /// All multixact IDs before this one have been replaced by a transaction ID in this table. This is used to track whether the table needs to be vacuumed in order to prevent multixact ID wraparound or to allow pg_multixact to be shrunk. Zero (InvalidMultiXactId) if the relation is not a table.
        (relminmxid, u32, get_struct)
        // (relacl, aclitem[], get_attr)
        /// Access-method-specific options, as “keyword=value” strings
        (reloptions, Array<String>, get_attr)
        // (relpartbound, pg_node_tree, get_attr)
    }
}

define_catalog! {
    /// The pg_enum catalog contains entries showing the values and labels for each enum type. The internal representation of a given enum value is actually the OID of its associated row in pg_enum.
    catalog (pg_enum) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// The OID of the pg_type entry owning this enum value
        (enumtypid, Oid, get_struct)
        /// The sort position of this enum value within its enum type
        (enumsortorder, f32, get_struct)
        /// The textual label for this enum value
        (enumlabel, &CStr, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_index contains part of the information about indexes. The rest is mostly in pg_class.
    catalog (pg_index) {
        /// The OID of the pg_class entry for this index
        (indexrelid, Oid, get_struct)
        /// The OID of the pg_class entry for the table this index is for
        (indrelid, Oid, get_struct)
        /// The total number of columns in the index (duplicates pg_class.relnatts); this number includes both key and included attributes
        (indnatts, i16, get_struct)
        /// The number of key columns in the index, not counting any included columns, which are merely stored and do not participate in the index semantics
        (indnkeyatts, i16, get_struct)
        /// If true, this is a unique index
        (indisunique, bool, get_struct)
        #[cfg(not(any(feature = "pg12", feature = "pg13", feature = "pg14")))]
        /// This value is only used for unique indexes. If false, this unique index will consider null values distinct (so the index can contain multiple null values in a column, the default PostgreSQL behavior). If it is true, it will consider null values to be equal (so the index can only contain one null value in a column).
        (indnullsnotdistinct, bool, get_struct)
        /// If true, this index represents the primary key of the table (indisunique should always be true when this is true)
        (indisprimary, bool, get_struct)
        /// If true, this index supports an exclusion constraint
        (indisexclusion, bool, get_struct)
        /// If true, the uniqueness check is enforced immediately on insertion (irrelevant if indisunique is not true)
        (indimmediate, bool, get_struct)
        /// If true, the table was last clustered on this index
        (indisclustered, bool, get_struct)
        /// If true, the index is currently valid for queries. False means the index is possibly incomplete: it must still be modified by INSERT/UPDATE operations, but it cannot safely be used for queries. If it is unique, the uniqueness property is not guaranteed true either.
        (indisvalid, bool, get_struct)
        /// If true, queries must not use the index until the xmin of this pg_index row is below their TransactionXmin event horizon, because the table may contain broken HOT chains with incompatible rows that they can see
        (indcheckxmin, bool, get_struct)
        /// If true, the index is currently ready for inserts. False means the index must be ignored by INSERT/UPDATE operations.
        (indisready, bool, get_struct)
        /// If false, the index is in process of being dropped, and should be ignored for all purposes (including HOT-safety decisions)
        (indislive, bool, get_struct)
        /// If true this index has been chosen as “replica identity” using ALTER TABLE ... REPLICA IDENTITY USING INDEX ...
        (indisreplident, bool, get_struct)
        /// This is an array of indnatts values that indicate which table columns this index indexes. For example, a value of 1 3 would mean that the first and the third table columns make up the index entries. Key columns come before non-key (included) columns. A zero in this array indicates that the corresponding index attribute is an expression over the table columns, rather than a simple column reference.
        (indkey, &[i16], get_struct)
        /// For each column in the index key (indnkeyatts values), this contains the OID of the collation to use for the index, or zero if the column is not of a collatable data type.
        (indcollation, Array<Oid>, get_attr_notnull)
        /// For each column in the index key (indnkeyatts values), this contains the OID of the operator class to use. See pg_opclass for details.
        (indclass, Array<Oid>, get_attr_notnull)
        /// This is an array of indnkeyatts values that store per-column flag bits. The meaning of the bits is defined by the index's access method.
        (indoption, Array<i16>, get_attr_notnull)
        // (indexprs, pg_node_tree, get_attr)
        // (indpred, pg_node_tree, get_attr)
    }
}

define_catalog! {
    /// The catalog pg_namespace stores namespaces. A namespace is the structure underlying SQL schemas: each namespace can have a separate collection of relations, types, etc. without name conflicts.
    catalog (pg_namespace) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Name of the namespace
        (nspname, &CStr, get_struct)
        /// Owner of the namespace
        (nspowner, Oid, get_struct)
        // (nspacl, aclitem[], get_attr)
    }
}

define_catalog! {
    /// The catalog pg_opclass defines index access method operator classes. Each operator class defines semantics for index columns of a particular data type and a particular index access method. An operator class essentially specifies that a particular operator family is applicable to a particular indexable column data type. The set of operators from the family that are actually usable with the indexed column are whichever ones accept the column's data type as their left-hand input.
    catalog (pg_opclass) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Index access method operator class is for
        (opcmethod, Oid, get_struct)
        /// Name of this operator class
        (opcname, &CStr, get_struct)
        /// Namespace of this operator class
        (opcnamespace, Oid, get_struct)
        /// Owner of the operator class
        (opcowner, Oid, get_struct)
        /// Operator family containing the operator class
        (opcfamily, Oid, get_struct)
        /// Data type that the operator class indexes
        (opcintype, Oid, get_struct)
        /// True if this operator class is the default for opcintype
        (opcdefault, bool, get_struct)
        /// Type of data stored in index, or zero if same as opcintype
        (opckeytype, Oid, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_operator stores information about operators.
    catalog (pg_operator) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Name of the operator
        (oprname, &CStr, get_struct)
        /// The OID of the namespace that contains this operator
        (oprnamespace, Oid, get_struct)
        /// Owner of the operator
        (oprowner, Oid, get_struct)
        (oprkind, PgOperatorOprkind, character {
            /// b = infix operator (“both”)
            (Infix, b'b')
            /// l = prefix operator (“left”)
            (Prefix, b'l')
        }, get_struct)
        /// This operator supports merge joins
        (oprcanmerge, bool, get_struct)
        /// This operator supports hash joins
        (oprcanhash, bool, get_struct)
        /// Type of the left operand (zero for a prefix operator)
        (oprleft, Oid, get_struct)
        /// Type of the right operand
        (oprright, Oid, get_struct)
        /// Type of the result (zero for a not-yet-defined “shell” operator)
        (oprresult, Oid, get_struct)
        /// Commutator of this operator (zero if none)
        (oprcom, Oid, get_struct)
        /// Negator of this operator (zero if none)
        (oprnegate, Oid, get_struct)
        /// Function that implements this operator (zero for a not-yet-defined “shell” operator)
        (oprcode, Regproc, get_struct)
        /// Restriction selectivity estimation function for this operator (zero if none)
        (oprrest, Regproc, get_struct)
        /// Join selectivity estimation function for this operator (zero if none)
        (oprjoin, Regproc, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_opfamily defines operator families. Each operator family is a collection of operators and associated support routines that implement the semantics specified for a particular index access method. Furthermore, the operators in a family are all “compatible”, in a way that is specified by the access method. The operator family concept allows cross-data-type operators to be used with indexes and to be reasoned about using knowledge of access method semantics.
    catalog (pg_opfamily) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Index access method operator family is for
        (opfmethod, Oid, get_struct)
        /// Name of this operator family
        (opfname, &CStr, get_struct)
        /// Namespace of this operator family
        (opfnamespace, Oid, get_struct)
        /// Owner of the operator family
        (opfowner, Oid, get_struct)
    }
}

define_catalog! {
    /// The catalog pg_proc stores information about functions, procedures, aggregate functions, and window functions (collectively also known as routines).
    catalog (pg_proc) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Name of the function
        (proname, &CStr, get_struct)
        /// The OID of the namespace that contains this function
        (pronamespace, Oid, get_struct)
        /// Owner of the function
        (proowner, Oid, get_struct)
        /// Implementation language or call interface of this function
        (prolang, Oid, get_struct)
        /// Estimated execution cost (in units of cpu_operator_cost); if proretset, this is cost per row returned
        (procost, f32, get_struct)
        /// Estimated number of result rows (zero if not proretset)
        (prorows, f32, get_struct)
        /// Data type of the variadic array parameter's elements, or zero if the function does not have a variadic parameter
        (provariadic, Oid, get_struct)
        /// Planner support function for this function, or zero if none
        (prosupport, Regproc, get_struct)
        (prokind, PgProcProkind, character {
            /// f for a normal function
            (Function, b'f')
            /// p for a procedure
            (Procedure, b'p')
            /// a for an aggregate function
            (Aggregate, b'a')
            /// w for a window function
            (Window, b'w')
        }, get_struct)
        /// Function is a security definer (i.e., a “setuid” function)
        (prosecdef, bool, get_struct)
        /// The function has no side effects. No information about the arguments is conveyed except via the return value. Any function that might throw an error depending on the values of its arguments is not leak-proof.
        (proleakproof, bool, get_struct)
        /// Function returns null if any call argument is null. In that case the function won't actually be called at all. Functions that are not “strict” must be prepared to handle null inputs.
        (proisstrict, bool, get_struct)
        /// Function returns a set (i.e., multiple values of the specified data type)
        (proretset, bool, get_struct)
        /// provolatile tells whether the function's result depends only on its input arguments, or is affected by outside factors.
        (provolatile, PgProcProvolatile, character {
            /// i for “immutable” functions, which always deliver the same result for the same inputs
            (Immutable, b'i')
            /// s for “stable” functions, whose results (for fixed inputs) do not change within a scan
            (Stable, b's')
            /// v for “volatile” functions, whose results might change at any time (Use v also for functions with side-effects, so that calls to them cannot get optimized away.)
            (Volatile, b'v')
        }, get_struct)
        /// proparallel tells whether the function can be safely run in parallel mode.
        (proparallel, PgProcProparallel, character {
            /// s for functions which are safe to run in parallel mode without restriction
            (Safe, b's')
            /// r for functions which can be run in parallel mode, but their execution is restricted to the parallel group leader; parallel worker processes cannot invoke these functions
            (Restricted, b'r')
            /// u for functions which are unsafe in parallel mode; the presence of such a function forces a serial execution plan
            (Unsafe, b'u')
        }, get_struct)
        /// Number of input arguments
        (pronargs, i16, get_struct)
        /// Number of arguments that have defaults
        (pronargdefaults, i16, get_struct)
        /// Data type of the return value
        (prorettype, Oid, get_struct)
        /// An array of the data types of the function arguments. This includes only input arguments (including INOUT and VARIADIC arguments), and thus represents the call signature of the function.
        (proargtypes, &[Oid], get_struct)
        /// An array of the data types of the function arguments. This includes all arguments (including OUT and INOUT arguments); however, if all the arguments are IN arguments, this field will be null. Note that subscripting is 1-based, whereas for historical reasons proargtypes is subscripted from 0.
        (proallargtypes, Array<Oid>, get_attr)
        /// An array of the modes of the function arguments. If all the arguments are IN arguments, this field will be null. Note that subscripts correspond to positions of proallargtypes not proargtypes.
        (proargmodes, Array<PgProcProargmodes>, character {
            /// i for IN arguments
            (In, b'i')
            /// o for OUT arguments
            (Out, b'o')
            /// b for INOUT arguments
            (Inout, b'b')
            /// v for VARIADIC arguments
            (Variadic, b'v')
            /// t for TABLE arguments
            (Table, b't')
        }, get_attr)
        /// An array of the names of the function arguments. Arguments without a name are set to empty strings in the array. If none of the arguments have a name, this field will be null. Note that subscripts correspond to positions of proallargtypes not proargtypes.
        (proargnames, Array<String>, get_attr)
        // (proargdefaults, pg_node_tree, get_attr)
        /// An array of the argument/result data type(s) for which to apply transforms (from the function's TRANSFORM clause). Null if none.
        (protrftypes, Array<Oid>, get_attr)
        /// This tells the function handler how to invoke the function. It might be the actual source code of the function for interpreted languages, a link symbol, a file name, or just about anything else, depending on the implementation language/call convention.
        (prosrc, &str, get_attr_notnull)
        /// Additional information about how to invoke the function. Again, the interpretation is language-specific.
        (probin, &str, get_attr)
        // (prosqlbody, pg_node_tree, get_attr)
        /// Function's local settings for run-time configuration variables
        (proconfig, Array<String>, get_attr)
        // (proacl, aclitem[], get_attr)
    }
}

define_catalog! {
    /// The catalog pg_type stores information about data types. Base types and enum types (scalar types) are created with CREATE TYPE, and domains with CREATE DOMAIN. A composite type is automatically created for each table in the database, to represent the row structure of the table. It is also possible to create composite types with CREATE TYPE AS.
    catalog (pg_type) {
        /// Row identifier
        (oid, Oid, get_struct)
        /// Data type name
        (typname, &CStr, get_struct)
        /// The OID of the namespace that contains this type
        (typnamespace, Oid, get_struct)
        /// Owner of the type
        (typowner, Oid, get_struct)
        /// For a fixed-size type, typlen is the number of bytes in the internal representation of the type. But for a variable-length type, typlen is negative. -1 indicates a “varlena” type (one that has a length word), -2 indicates a null-terminated C string.
        (typlen, i16, get_struct)
        /// typbyval determines whether internal routines pass a value of this type by value or by reference. typbyval had better be false if typlen is not 1, 2, or 4 (or 8 on machines where Datum is 8 bytes). Variable-length types are always passed by reference. Note that typbyval can be false even if the length would allow pass-by-value.
        (typbyval, bool, get_struct)
        /// See also typrelid and typbasetype.
        (typtype, PgTypeTyptype, character {
            /// b for a base type
            (Base, b'b')
            /// c for a composite type (e.g., a table's row type)
            (Composite, b'c')
            /// d for a domain
            (Domain, b'd')
            /// e for an enum type
            (Enum, b'e')
            /// m for a multirange type
            (Multirange, b'm')
            /// p for a pseudo-type
            (Pseudo, b'p')
            /// r for a range type
            (Range, b'r')
        }, get_struct)
        /// typcategory is an arbitrary classification of data types that is used by the parser to determine which implicit casts should be “preferred”.
        (typcategory, PgTypeTypcategory, character {
            /// Array types
            (Array, b'A')
            /// Boolean types
            (Boolean, b'B')
            /// Composite types
            (Composite, b'C')
            /// Date/time types
            (DateTime, b'D')
            /// Enum types
            (Enum, b'E')
            /// Geometric types
            (Geometric, b'G')
            /// Network address types
            (Network, b'I')
            /// Numeric types
            (Numeric, b'N')
            /// Pseudo-types
            (PseudoType, b'P')
            /// Range types
            (Range, b'R')
            /// String types
            (String, b'S')
            /// Timespan types
            (TimeSpan, b'T')
            /// User-defined types
            (User, b'U')
            /// Bit-string types
            (BitString, b'V')
            /// unknown type
            (Unknown, b'X')
            /// Internal-use types
            (Internal, b'Z')
        }, get_struct)
        /// True if the type is a preferred cast target within its typcategory
        (typispreferred, bool, get_struct)
        /// True if the type is defined, false if this is a placeholder entry for a not-yet-defined type. When typisdefined is false, nothing except the type name, namespace, and OID can be relied on.
        (typisdefined, bool, get_struct)
        /// Character that separates two values of this type when parsing array input. Note that the delimiter is associated with the array element data type, not the array data type.
        (typdelim, c_char, get_struct)
        /// If this is a composite type (see typtype), then this column points to the pg_class entry that defines the corresponding table. (For a free-standing composite type, the pg_class entry doesn't really represent a table, but it is needed anyway for the type's pg_attribute entries to link to.) Zero for non-composite types.
        (typrelid, Oid, get_struct)
        #[cfg(not(any(feature = "pg12", feature = "pg13")))]
        /// Subscripting handler function's OID, or zero if this type doesn't support subscripting. Types that are “true” array types have typsubscript = array_subscript_handler, but other types may have other handler functions to implement specialized subscripting behavior.
        (typsubscript, Regproc, get_struct)
        /// If typelem is not zero then it identifies another row in pg_type, defining the type yielded by subscripting. This should be zero if typsubscript is zero. However, it can be zero when typsubscript isn't zero, if the handler doesn't need typelem to determine the subscripting result type. Note that a typelem dependency is considered to imply physical containment of the element type in this type; so DDL changes on the element type might be restricted by the presence of this type.
        (typelem, Oid, get_struct)
        /// If typarray is not zero then it identifies another row in pg_type, which is the “true” array type having this type as element
        (typarray, Oid, get_struct)
        /// Input conversion function (text format)
        (typinput, Regproc, get_struct)
        /// Output conversion function (text format)
        (typoutput, Regproc, get_struct)
        /// Input conversion function (binary format), or zero if none
        (typreceive, Regproc, get_struct)
        /// Output conversion function (binary format), or zero if none
        (typsend, Regproc, get_struct)
        /// Type modifier input function, or zero if type does not support modifiers
        (typmodin, Regproc, get_struct)
        /// Type modifier output function, or zero to use the standard format
        (typmodout, Regproc, get_struct)
        /// Custom ANALYZE function, or zero to use the standard function
        (typanalyze, Regproc, get_struct)
        /// typalign is the alignment required when storing a value of this type. It applies to storage on disk as well as most representations of the value inside PostgreSQL. When multiple values are stored consecutively, such as in the representation of a complete row on disk, padding is inserted before a datum of this type so that it begins on the specified boundary. The alignment reference is the beginning of the first datum in the sequence.
        (typalign, PgTypeTypalign, character {
            /// c = char alignment, i.e., no alignment needed.
            (Char, b'c')
            /// s = short alignment (2 bytes on most machines).
            (Short, b's')
            /// i = int alignment (4 bytes on most machines).
            (Int, b'i')
            /// d = double alignment (8 bytes on many machines, but by no means all).
            (Double, b'd')
        }, get_struct)
        /// typstorage tells for varlena types (those with typlen = -1) if the type is prepared for toasting and what the default strategy for attributes of this type should be.
        /// x is the usual choice for toast-able types. Note that m values can also be moved out to secondary storage, but only as a last resort (e and x values are moved first).
        (typstorage, PgTypeTypstorage, character {
            /// p (plain): Values must always be stored plain (non-varlena types always use this value).
            (Plain, b'p')
            /// e (external): Values can be stored in a secondary “TOAST” relation (if relation has one, see pg_class.reltoastrelid).
            (External, b'e')
            /// x (extended): Values can be compressed and/or moved to a secondary relation.
            (Extended, b'x')
            /// m (main): Values can be compressed and stored inline.
            (Main, b'm')
        }, get_struct)
        /// typnotnull represents a not-null constraint on a type. Used for domains only.
        (typnotnull, bool, get_struct)
        /// If this is a domain (see typtype), then typbasetype identifies the type that this one is based on. Zero if this type is not a domain.
        (typbasetype, Oid, get_struct)
        /// Domains use typtypmod to record the typmod to be applied to their base type (-1 if base type does not use a typmod). -1 if this type is not a domain.
        (typtypmod, i32, get_struct)
        /// typndims is the number of array dimensions for a domain over an array (that is, typbasetype is an array type). Zero for types other than domains over array types.
        (typndims, i32, get_struct)
        /// typcollation specifies the collation of the type. If the type does not support collations, this will be zero. A base type that supports collations will have a nonzero value here, typically DEFAULT_COLLATION_OID. A domain over a collatable type can have a collation OID different from its base type's, if one was specified for the domain.
        (typcollation, Oid, get_struct)
        // (typdefaultbin, pg_node_tree, get_attr)
        /// typdefault is null if the type has no associated default value. If typdefaultbin is not null, typdefault must contain a human-readable version of the default expression represented by typdefaultbin. If typdefaultbin is null and typdefault is not, then typdefault is the external representation of the type's default value, which can be fed to the type's input converter to produce a constant.
        (typdefault, String, get_attr)
        // (typacl, aclitem[], get_attr)
    }
}

define_cache! {
    cache (amname, pg_am) {
        (amname, &CStr)
    }
}

define_cache! {
    cache (amoid, pg_am) {
        (oid, Oid)
    }
}

define_cache! {
    cache (amopopid, pg_amop) {
        (amopopr, Oid)
        (amoppurpose, PgAmopAmoppurpose)
        (amopfamily, Oid)
    }
}

define_cache! {
    cache (amopstrategy, pg_amop) {
        (amopfamily, Oid)
        (amoplefttype, Oid)
        (amoprighttype, Oid)
        (amopstrategy, i16)
    }
}

define_cache! {
    cache (amprocnum, pg_amproc) {
        (amprocfamily, Oid)
        (amproclefttype, Oid)
        (amprocrighttype, Oid)
        (amprocnum, i16)
    }
}

define_cache! {
    cache (claamnamensp, pg_opclass) {
        (opcmethod, Oid)
        (opcname, &CStr)
        (opcnamespace, Oid)
    }
}

define_cache! {
    cache (claoid, pg_opclass) {
        (oid, Oid)
    }
}

define_cache! {
    cache (enumoid, pg_enum) {
        (oid, Oid)
    }
}

define_cache! {
    cache (enumtypoidname, pg_enum) {
        (enumtypid, Oid)
        (enumlabel, &CStr)
    }
}

define_cache! {
    cache (indexrelid, pg_index) {
        (indexrelid, Oid)
    }
}

define_cache! {
    cache (namespacename, pg_namespace) {
        (nspname, &CStr)
    }
}

define_cache! {
    cache (namespaceoid, pg_namespace) {
        (oid, Oid)
    }
}

define_cache! {
    cache (opernamensp, pg_operator) {
        (oprname, &CStr)
        (oprleft, Oid)
        (oprright, Oid)
        (oprnamespace, Oid)
    }
}

define_cache! {
    cache (operoid, pg_operator) {
        (oid, Oid)
    }
}

define_cache! {
    cache (opfamilyamnamensp, pg_opfamily) {
        (opfmethod, Oid)
        (opfname, &CStr)
        (opfnamespace, Oid)
    }
}

define_cache! {
    cache (opfamilyoid, pg_opfamily) {
        (oid, Oid)
    }
}

define_cache! {
    cache (procnameargsnsp, pg_proc) {
        (proname, &CStr)
        (proargtypes, &[Oid])
        (pronamespace, Oid)
    }
}

define_cache! {
    cache (procoid, pg_proc) {
        (oid, Oid)
    }
}

define_cache! {
    cache (relnamensp, pg_class) {
        (relname, &CStr)
        (relnamespace, Oid)
    }
}

define_cache! {
    cache (reloid, pg_class) {
        (oid, Oid)
    }
}

define_cache! {
    cache (typenamensp, pg_type) {
        (typname, &CStr)
        (typnamespace, Oid)
    }
}

define_cache! {
    cache (typeoid, pg_type) {
        (oid, Oid)
    }
}

use pgrx::list::List;

impl PgProc<'_> {
    /// Expression trees for default values. This is a [`List`] with `pronargdefaults` elements,
    /// corresponding to the last N input arguments (i.e., the last N proargtypes positions).
    ///
    /// If none of the arguments have defaults, this function returns [`Option::None`].
    pub fn proargdefaults<'cx>(
        &self,
        mcx: &'cx pgrx::memcx::MemCx<'_>,
    ) -> Option<pgrx::list::List<'cx, *mut std::ffi::c_void>> {
        unsafe {
            use crate::FromDatum;
            use pg_sys::AsPgCStr;

            let mut is_null = false;
            let proargdefaults = pg_sys::SysCacheGetAttr(
                pg_sys::SysCacheIdentifier::PROCOID as _,
                std::ptr::addr_of!(*self.inner).cast_mut(),
                pg_sys::Anum_pg_proc_proargdefaults as _,
                &mut is_null,
            );
            let proargdefaults = <&str>::from_datum(proargdefaults, is_null)?;

            let str = proargdefaults.as_pg_cstr();
            let argdefaults = mcx.exec_in(|| pg_sys::stringToNode(str)).cast::<pg_sys::List>();
            pg_sys::pfree(str.cast());
            List::downcast_ptr_in_memcx(argdefaults, mcx)
        }
    }
}
