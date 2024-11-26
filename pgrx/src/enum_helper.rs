//LICENSE Portions Copyright 2019-2021 ZomboDB, LLC.
//LICENSE
//LICENSE Portions Copyright 2021-2023 Technology Concepts & Design, Inc.
//LICENSE
//LICENSE Portions Copyright 2023-2023 PgCentral Foundation, Inc. <contact@pgcentral.org>
//LICENSE
//LICENSE All rights reserved.
//LICENSE
//LICENSE Use of this source code is governed by the MIT license that can be found in the LICENSE file.
//! Helper functions for working with Postgres `enum` types

use crate::pg_catalog::PgEnum;
use crate::{ereport, pg_sys, PgLogLevel, PgSqlErrorCode};

pub fn lookup_enum_by_oid(enumval: pg_sys::Oid) -> (String, pg_sys::Oid, f32) {
    let pg_enum = PgEnum::search_enumoid(enumval).unwrap();

    let Some(pg_enum) = pg_enum.get() else {
        ereport!(
            PgLogLevel::ERROR,
            PgSqlErrorCode::ERRCODE_INVALID_BINARY_REPRESENTATION,
            format!("invalid internal value for enum: {enumval:?}")
        );
        unreachable!()
    };

    (
        pg_enum.enumlabel().to_str().unwrap().to_string(),
        pg_enum.enumtypid(),
        pg_enum.enumsortorder() as f32,
    )
}

pub fn lookup_enum_by_label(typname: &str, label: &str) -> pg_sys::Datum {
    let enumtypoid = crate::regtypein(typname);

    if enumtypoid == pg_sys::InvalidOid {
        panic!("could not locate type oid for type: {typname}");
    }

    let label = std::ffi::CString::new(label).expect("failed to convert enum typname to a CString");

    let pg_enum = PgEnum::search_enumtypoidname(enumtypoid, &label).unwrap();

    let Some(pg_enum) = pg_enum.get() else {
        panic!("could not find heap tuple for enum: {typname}.{label:?}, typoid={enumtypoid:?}");
    };

    pg_sys::Datum::from(pg_enum.oid())
}
