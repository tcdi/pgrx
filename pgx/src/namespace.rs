/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

//! A helper struct for creating a Postgres `List` of `String`s to qualify an object name

use crate::list::PgList;
use crate::pg_sys;
use pgx_pg_sys::AsPgCStr;

/// A helper struct for creating a Postgres `List` of `String`s to qualify an object name
pub struct PgQualifiedNameBuilder {
    #[cfg(feature = "pg15")]
    list: PgList<pg_sys::String>,
    #[cfg(not(feature = "pg15"))]
    list: PgList<pg_sys::Value>,
}

impl Default for PgQualifiedNameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PgQualifiedNameBuilder {
    pub fn new() -> PgQualifiedNameBuilder {
        PgQualifiedNameBuilder {
            #[cfg(feature = "pg15")]
            list: PgList::<pg_sys::String>::new(),
            #[cfg(not(feature = "pg15"))]
            list: PgList::<pg_sys::Value>::new(),
        }
    }

    pub fn push(mut self, value: &str) -> PgQualifiedNameBuilder {
        unsafe {
            // SAFETY:  the result of pg_sys::makeString is always a valid pointer
            self.list.push(pg_sys::makeString(value.as_pg_cstr()));
        }
        self
    }

    pub fn get_operator_oid(self, lhs_type: pg_sys::Oid, rhs_type: pg_sys::Oid) -> pg_sys::Oid {
        unsafe { pg_sys::OpernameGetOprid(self.list.into_pg(), lhs_type, rhs_type) }
    }
}
