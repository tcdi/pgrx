use pgrx::prelude::*;

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::pg_sys::Oid;
    use std::ffi::CString;

    #[allow(unused_imports)]
    use crate as pgrx_tests;
    use pgrx::prelude::*;

    #[pg_test]
    fn test_pg_catalog_pg_proc_boolin() {
        use pgrx::pg_catalog::*;
        let proname = CString::new("boolin").unwrap();
        let proargtypes = /* cstring */ [pgrx::wrappers::regtypein("cstring")];
        let pronamespace = /* pg_catalog */ Oid::from(11);
        // search
        let pg_proc = PgProc::search_procnameargsnsp(&proname, &proargtypes, pronamespace).unwrap();
        let pg_proc = pg_proc.get().unwrap();
        // getstruct, name
        assert_eq!(pg_proc.proname(), proname.as_c_str());
        // getstruct, primitive types
        assert_eq!(pg_proc.pronamespace(), pronamespace);
        assert_eq!(pg_proc.procost(), 1.0);
        assert_eq!(pg_proc.prorows(), 0.0);
        assert_eq!(pg_proc.provariadic(), Oid::INVALID);
        // getstruct, regproc
        assert_eq!(pg_proc.prosupport(), Oid::INVALID);
        // getstruct, char
        assert_eq!(pg_proc.prokind(), PgProcProkind::Function);
        assert_eq!(pg_proc.prosecdef(), false);
        assert_eq!(pg_proc.proleakproof(), false);
        assert_eq!(pg_proc.proisstrict(), true);
        assert_eq!(pg_proc.proretset(), false);
        assert_eq!(pg_proc.provolatile(), PgProcProvolatile::Immutable);
        assert_eq!(pg_proc.proparallel(), PgProcProparallel::Safe);
        assert_eq!(pg_proc.pronargs(), 1);
        assert_eq!(pg_proc.pronargdefaults(), 0);
        assert_eq!(pg_proc.prorettype(), pgrx::pg_sys::BOOLOID);
        // getstruct, oidvector
        assert_eq!(pg_proc.proargtypes(), &proargtypes);
        // getattr, null
        assert!(pg_proc.proallargtypes().is_none());
        assert!(pg_proc.proargmodes().is_none());
        assert!(pg_proc.proargnames().is_none());
        assert!(pg_proc.protrftypes().is_none());
        // getattr, text
        assert_eq!(pg_proc.prosrc(), "boolin");
        assert!(pg_proc.probin().is_none());
        assert!(pg_proc.proconfig().is_none());
    }

    #[pg_test]
    fn test_pg_catalog_pg_proc_num_nulls() {
        use pgrx::pg_catalog::*;
        let proname = CString::new("num_nulls").unwrap();
        let proargtypes = [pgrx::pg_sys::ANYOID];
        let pronamespace = /* pg_catalog */ pgrx::pg_sys::Oid::from(11);
        let pg_proc = PgProc::search_procnameargsnsp(&proname, &proargtypes, pronamespace).unwrap();
        let pg_proc = pg_proc.get().unwrap();
        assert_eq!(pg_proc.proname(), proname.as_c_str());
        assert_eq!(pg_proc.pronamespace(), pronamespace);
        assert_eq!(pg_proc.procost(), 1.0);
        assert_eq!(pg_proc.prorows(), 0.0);
        assert_eq!(pg_proc.provariadic(), pgrx::pg_sys::ANYOID);
        assert_eq!(pg_proc.prosupport(), Oid::INVALID);
        assert_eq!(pg_proc.prokind(), PgProcProkind::Function);
        assert_eq!(pg_proc.prosecdef(), false);
        assert_eq!(pg_proc.proleakproof(), false);
        assert_eq!(pg_proc.proisstrict(), false);
        assert_eq!(pg_proc.proretset(), false);
        assert_eq!(pg_proc.provolatile(), PgProcProvolatile::Immutable);
        assert_eq!(pg_proc.proparallel(), PgProcProparallel::Safe);
        assert_eq!(pg_proc.pronargs(), 1);
        assert_eq!(pg_proc.pronargdefaults(), 0);
        assert_eq!(pg_proc.prorettype(), pgrx::pg_sys::INT4OID);
        assert_eq!(pg_proc.proargtypes(), &proargtypes);
        // getattr, oid[]
        assert_eq!(
            pg_proc.proallargtypes().map(|v| v.iter().collect()),
            Some(vec![Some(pgrx::pg_sys::ANYOID)])
        );
        // getattr, char[]
        assert_eq!(
            pg_proc.proargmodes().map(|v| v.iter().collect()),
            Some(vec![Some(PgProcProargmodes::Variadic)])
        );
        assert!(pg_proc.proargnames().is_none());
        assert!(pg_proc.protrftypes().is_none());
        assert_eq!(pg_proc.prosrc(), "pg_num_nulls");
        assert!(pg_proc.probin().is_none());
        assert!(pg_proc.proconfig().is_none());
    }

    #[pg_test]
    fn test_pg_catalog_pg_proc_gcd() {
        // skip this test for pg12
        if pgrx::pg_sys::PG_VERSION_NUM < 130000 {
            return;
        }
        use pgrx::pg_catalog::*;
        let proname = CString::new("gcd").unwrap();
        // search_list
        let pg_proc = PgProc::search_list_procnameargsnsp_1(&proname).unwrap();
        let mut int4gcd = false;
        let mut int8gcd = false;
        for i in 0..pg_proc.len() {
            let pg_proc = pg_proc.get(i).unwrap();
            if pg_proc.prosrc() == "int4gcd" {
                int4gcd = true;
            }
            if pg_proc.prosrc() == "int8gcd" {
                int8gcd = true;
            }
        }
        assert!(int4gcd);
        assert!(int8gcd);
    }

    #[pg_test]
    fn test_pg_catalog_pg_class_pg_stats() {
        use pgrx::pg_catalog::*;
        let relname = CString::new("pg_stats").unwrap();
        let relnamespace = /* pg_catalog */ pgrx::pg_sys::Oid::from(11);
        let pg_class = PgClass::search_relnamensp(&relname, relnamespace).unwrap();
        let pg_class = pg_class.get().unwrap();
        // getattr, text[]
        assert_eq!(
            pg_class.reloptions().map(|v| v.iter().collect()),
            Some(vec![Some("security_barrier=true".to_string())])
        );
    }
}
