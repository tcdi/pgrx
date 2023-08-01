use crate::NotBuiltinOid;
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub enum BuiltinOid {
    ABSTIMEARRAYOID = 1023,
    ABSTIMEOID = 702,
    ACLITEMARRAYOID = 1034,
    ACLITEMOID = 1033,
    ANYARRAYOID = 2277,
    ANYELEMENTOID = 2283,
    ANYENUMOID = 3500,
    ANYNONARRAYOID = 2776,
    ANYOID = 2276,
    ANYRANGEOID = 3831,
    AT_REWRITE_ALTER_OID = 8,
    AccessMethodOperatorRelationId = 2602,
    AttributeRelationId = 1249,
    AuthIdRelationId = 1260,
    BITARRAYOID = 1561,
    BITOID = 1560,
    BOOLARRAYOID = 1000,
    BOOLOID = 16,
    BOOL_BTREE_FAM_OID = 424,
    BOOL_HASH_FAM_OID = 2222,
    BOXARRAYOID = 1020,
    BOXOID = 603,
    BPCHARARRAYOID = 1014,
    BPCHAROID = 1042,
    BPCHAR_BTREE_FAM_OID = 426,
    BPCHAR_PATTERN_BTREE_FAM_OID = 2097,
    BYTEAARRAYOID = 1001,
    BYTEAOID = 17,
    BYTEA_BTREE_FAM_OID = 428,
    CASHOID = 790,
    CHARARRAYOID = 1002,
    CHAROID = 18,
    CIDARRAYOID = 1012,
    CIDOID = 29,
    CIDRARRAYOID = 651,
    CIDROID = 650,
    CIRCLEARRAYOID = 719,
    CIRCLEOID = 718,
    CSTRINGARRAYOID = 1263,
    CSTRINGOID = 2275,
    DATEARRAYOID = 1182,
    DATEOID = 1082,
    DATERANGEARRAYOID = 3913,
    DATERANGEOID = 3912,
    DATE_BTREE_OPS_OID = 3122,
    DEFAULTTABLESPACE_OID = 1663,
    DatabaseRelationId = 1262,
    EVTTRIGGEROID = 3838,
    EnumRelationId = 3501,
    EventTriggerRelationId = 3466,
    ExtensionRelationId = 3079,
    FDW_HANDLEROID = 3115,
    FLOAT4ARRAYOID = 1021,
    FLOAT4OID = 700,
    FLOAT8ARRAYOID = 1022,
    FLOAT8OID = 701,
    FLOAT8_BTREE_OPS_OID = 3123,
    ForeignServerRelationId = 1417,
    ForeignTableRelationId = 3118,
    GLOBALTABLESPACE_OID = 1664,
    GTSVECTORARRAYOID = 3644,
    GTSVECTOROID = 3642,
    INDEX_AM_HANDLEROID = 325,
    INETARRAYOID = 1041,
    INETOID = 869,
    INT2ARRAYOID = 1005,
    INT2OID = 21,
    INT2VECTORARRAYOID = 1006,
    INT2VECTOROID = 22,
    INT2_BTREE_OPS_OID = 1979,
    INT4ARRAYOID = 1007,
    INT4OID = 23,
    INT4RANGEARRAYOID = 3905,
    INT4RANGEOID = 3904,
    INT4_BTREE_OPS_OID = 1978,
    INT8ARRAYOID = 1016,
    INT8OID = 20,
    INT8RANGEARRAYOID = 3927,
    INT8RANGEOID = 3926,
    INT8_BTREE_OPS_OID = 3124,
    INTEGER_BTREE_FAM_OID = 1976,
    INTERNALOID = 2281,
    INTERVALARRAYOID = 1187,
    INTERVALOID = 1186,
    IndexRelationId = 2610,
    JSONARRAYOID = 199,
    JSONBARRAYOID = 3807,
    JSONBOID = 3802,
    JSONOID = 114,
    LANGUAGE_HANDLEROID = 2280,
    LINEARRAYOID = 629,
    LINEOID = 628,
    LSEGARRAYOID = 1018,
    LSEGOID = 601,
    LSNOID = 3220,
    MACADDR8ARRAYOID = 775,
    MACADDR8OID = 774,
    MACADDRARRAYOID = 1040,
    MACADDROID = 829,
    MONEYARRAYOID = 791,
    NAMEARRAYOID = 1003,
    NAMEOID = 19,
    NAME_BTREE_FAM_OID = 1986,
    NETWORK_BTREE_FAM_OID = 1974,
    NUMERICARRAYOID = 1231,
    NUMERICOID = 1700,
    NUMERIC_BTREE_OPS_OID = 3125,
    NUMRANGEARRAYOID = 3907,
    NUMRANGEOID = 3906,
    NamespaceRelationId = 2615,
    OIDARRAYOID = 1028,
    OIDOID = 26,
    OIDVECTORARRAYOID = 1013,
    OIDVECTOROID = 30,
    OID_BTREE_FAM_OID = 1989,
    OID_BTREE_OPS_OID = 1981,
    OPAQUEOID = 2282,
    OperatorClassRelationId = 2616,
    OperatorFamilyRelationId = 2753,
    OperatorRelationId = 2617,
    PATHARRAYOID = 1019,
    PATHOID = 602,
    PGDDLCOMMANDOID = 32,
    PGDEPENDENCIESOID = 3402,
    PGNDISTINCTOID = 3361,
    PGNODETREEOID = 194,
    PG_LSNARRAYOID = 3221,
    POINTARRAYOID = 1017,
    POINTOID = 600,
    POLYGONARRAYOID = 1027,
    POLYGONOID = 604,
    ProcedureRelationId = 1255,
    PublicationRelationId = 6104,
    RECORDARRAYOID = 2287,
    RECORDOID = 2249,
    REFCURSORARRAYOID = 2201,
    REFCURSOROID = 1790,
    REGCLASSARRAYOID = 2210,
    REGCLASSOID = 2205,
    REGCONFIGARRAYOID = 3735,
    REGCONFIGOID = 3734,
    REGDICTIONARYARRAYOID = 3770,
    REGDICTIONARYOID = 3769,
    REGNAMESPACEARRAYOID = 4090,
    REGNAMESPACEOID = 4089,
    REGOPERARRAYOID = 2208,
    REGOPERATORARRAYOID = 2209,
    REGOPERATOROID = 2204,
    REGOPEROID = 2203,
    REGPROCARRAYOID = 1008,
    REGPROCEDUREARRAYOID = 2207,
    REGPROCEDUREOID = 2202,
    REGPROCOID = 24,
    REGROLEARRAYOID = 4097,
    REGROLEOID = 4096,
    REGTYPEARRAYOID = 2211,
    REGTYPEOID = 2206,
    RELTIMEARRAYOID = 1024,
    RELTIMEOID = 703,
    RelationRelationId = 1259,
    SMGROID = 210,
    StatisticRelationId = 2619,
    TEXTARRAYOID = 1009,
    TEXTOID = 25,
    TEXT_BTREE_FAM_OID = 1994,
    TEXT_BTREE_OPS_OID = 3126,
    TEXT_PATTERN_BTREE_FAM_OID = 2095,
    TEXT_SPGIST_FAM_OID = 4017,
    TIDARRAYOID = 1010,
    TIDOID = 27,
    TIMEARRAYOID = 1183,
    TIMEOID = 1083,
    TIMESTAMPARRAYOID = 1115,
    TIMESTAMPOID = 1114,
    TIMESTAMPTZARRAYOID = 1185,
    TIMESTAMPTZOID = 1184,
    TIMESTAMPTZ_BTREE_OPS_OID = 3127,
    TIMESTAMP_BTREE_OPS_OID = 3128,
    TIMETZARRAYOID = 1270,
    TIMETZOID = 1266,
    TINTERVALARRAYOID = 1025,
    TINTERVALOID = 704,
    TRIGGEROID = 2279,
    TSM_HANDLEROID = 3310,
    TSQUERYARRAYOID = 3645,
    TSQUERYOID = 3615,
    TSRANGEARRAYOID = 3909,
    TSRANGEOID = 3908,
    TSTZRANGEARRAYOID = 3911,
    TSTZRANGEOID = 3910,
    TSVECTORARRAYOID = 3643,
    TSVECTOROID = 3614,
    TXID_SNAPSHOTARRAYOID = 2949,
    TXID_SNAPSHOTOID = 2970,
    TableSpaceRelationId = 1213,
    TemplateDbOid = 1,
    TriggerRelationId = 2620,
    TypeRelationId = 1247,
    UNKNOWNOID = 705,
    UUIDARRAYOID = 2951,
    UUIDOID = 2950,
    VARBITARRAYOID = 1563,
    VARBITOID = 1562,
    VARCHARARRAYOID = 1015,
    VARCHAROID = 1043,
    VOIDOID = 2278,
    XIDARRAYOID = 1011,
    XIDOID = 28,
    XMLARRAYOID = 143,
    XMLOID = 142,
}
impl BuiltinOid {
    pub const fn from_u32(uint: u32) -> Result<BuiltinOid, NotBuiltinOid> {
        match uint {
            0 => Err(NotBuiltinOid::Invalid),
            1023 => Ok(BuiltinOid::ABSTIMEARRAYOID),
            702 => Ok(BuiltinOid::ABSTIMEOID),
            1034 => Ok(BuiltinOid::ACLITEMARRAYOID),
            1033 => Ok(BuiltinOid::ACLITEMOID),
            2277 => Ok(BuiltinOid::ANYARRAYOID),
            2283 => Ok(BuiltinOid::ANYELEMENTOID),
            3500 => Ok(BuiltinOid::ANYENUMOID),
            2776 => Ok(BuiltinOid::ANYNONARRAYOID),
            2276 => Ok(BuiltinOid::ANYOID),
            3831 => Ok(BuiltinOid::ANYRANGEOID),
            8 => Ok(BuiltinOid::AT_REWRITE_ALTER_OID),
            2602 => Ok(BuiltinOid::AccessMethodOperatorRelationId),
            1249 => Ok(BuiltinOid::AttributeRelationId),
            1260 => Ok(BuiltinOid::AuthIdRelationId),
            1561 => Ok(BuiltinOid::BITARRAYOID),
            1560 => Ok(BuiltinOid::BITOID),
            1000 => Ok(BuiltinOid::BOOLARRAYOID),
            16 => Ok(BuiltinOid::BOOLOID),
            424 => Ok(BuiltinOid::BOOL_BTREE_FAM_OID),
            2222 => Ok(BuiltinOid::BOOL_HASH_FAM_OID),
            1020 => Ok(BuiltinOid::BOXARRAYOID),
            603 => Ok(BuiltinOid::BOXOID),
            1014 => Ok(BuiltinOid::BPCHARARRAYOID),
            1042 => Ok(BuiltinOid::BPCHAROID),
            426 => Ok(BuiltinOid::BPCHAR_BTREE_FAM_OID),
            2097 => Ok(BuiltinOid::BPCHAR_PATTERN_BTREE_FAM_OID),
            1001 => Ok(BuiltinOid::BYTEAARRAYOID),
            17 => Ok(BuiltinOid::BYTEAOID),
            428 => Ok(BuiltinOid::BYTEA_BTREE_FAM_OID),
            790 => Ok(BuiltinOid::CASHOID),
            1002 => Ok(BuiltinOid::CHARARRAYOID),
            18 => Ok(BuiltinOid::CHAROID),
            1012 => Ok(BuiltinOid::CIDARRAYOID),
            29 => Ok(BuiltinOid::CIDOID),
            651 => Ok(BuiltinOid::CIDRARRAYOID),
            650 => Ok(BuiltinOid::CIDROID),
            719 => Ok(BuiltinOid::CIRCLEARRAYOID),
            718 => Ok(BuiltinOid::CIRCLEOID),
            1263 => Ok(BuiltinOid::CSTRINGARRAYOID),
            2275 => Ok(BuiltinOid::CSTRINGOID),
            1182 => Ok(BuiltinOid::DATEARRAYOID),
            1082 => Ok(BuiltinOid::DATEOID),
            3913 => Ok(BuiltinOid::DATERANGEARRAYOID),
            3912 => Ok(BuiltinOid::DATERANGEOID),
            3122 => Ok(BuiltinOid::DATE_BTREE_OPS_OID),
            1663 => Ok(BuiltinOid::DEFAULTTABLESPACE_OID),
            1262 => Ok(BuiltinOid::DatabaseRelationId),
            3838 => Ok(BuiltinOid::EVTTRIGGEROID),
            3501 => Ok(BuiltinOid::EnumRelationId),
            3466 => Ok(BuiltinOid::EventTriggerRelationId),
            3079 => Ok(BuiltinOid::ExtensionRelationId),
            3115 => Ok(BuiltinOid::FDW_HANDLEROID),
            1021 => Ok(BuiltinOid::FLOAT4ARRAYOID),
            700 => Ok(BuiltinOid::FLOAT4OID),
            1022 => Ok(BuiltinOid::FLOAT8ARRAYOID),
            701 => Ok(BuiltinOid::FLOAT8OID),
            3123 => Ok(BuiltinOid::FLOAT8_BTREE_OPS_OID),
            1417 => Ok(BuiltinOid::ForeignServerRelationId),
            3118 => Ok(BuiltinOid::ForeignTableRelationId),
            1664 => Ok(BuiltinOid::GLOBALTABLESPACE_OID),
            3644 => Ok(BuiltinOid::GTSVECTORARRAYOID),
            3642 => Ok(BuiltinOid::GTSVECTOROID),
            325 => Ok(BuiltinOid::INDEX_AM_HANDLEROID),
            1041 => Ok(BuiltinOid::INETARRAYOID),
            869 => Ok(BuiltinOid::INETOID),
            1005 => Ok(BuiltinOid::INT2ARRAYOID),
            21 => Ok(BuiltinOid::INT2OID),
            1006 => Ok(BuiltinOid::INT2VECTORARRAYOID),
            22 => Ok(BuiltinOid::INT2VECTOROID),
            1979 => Ok(BuiltinOid::INT2_BTREE_OPS_OID),
            1007 => Ok(BuiltinOid::INT4ARRAYOID),
            23 => Ok(BuiltinOid::INT4OID),
            3905 => Ok(BuiltinOid::INT4RANGEARRAYOID),
            3904 => Ok(BuiltinOid::INT4RANGEOID),
            1978 => Ok(BuiltinOid::INT4_BTREE_OPS_OID),
            1016 => Ok(BuiltinOid::INT8ARRAYOID),
            20 => Ok(BuiltinOid::INT8OID),
            3927 => Ok(BuiltinOid::INT8RANGEARRAYOID),
            3926 => Ok(BuiltinOid::INT8RANGEOID),
            3124 => Ok(BuiltinOid::INT8_BTREE_OPS_OID),
            1976 => Ok(BuiltinOid::INTEGER_BTREE_FAM_OID),
            2281 => Ok(BuiltinOid::INTERNALOID),
            1187 => Ok(BuiltinOid::INTERVALARRAYOID),
            1186 => Ok(BuiltinOid::INTERVALOID),
            2610 => Ok(BuiltinOid::IndexRelationId),
            199 => Ok(BuiltinOid::JSONARRAYOID),
            3807 => Ok(BuiltinOid::JSONBARRAYOID),
            3802 => Ok(BuiltinOid::JSONBOID),
            114 => Ok(BuiltinOid::JSONOID),
            2280 => Ok(BuiltinOid::LANGUAGE_HANDLEROID),
            629 => Ok(BuiltinOid::LINEARRAYOID),
            628 => Ok(BuiltinOid::LINEOID),
            1018 => Ok(BuiltinOid::LSEGARRAYOID),
            601 => Ok(BuiltinOid::LSEGOID),
            3220 => Ok(BuiltinOid::LSNOID),
            775 => Ok(BuiltinOid::MACADDR8ARRAYOID),
            774 => Ok(BuiltinOid::MACADDR8OID),
            1040 => Ok(BuiltinOid::MACADDRARRAYOID),
            829 => Ok(BuiltinOid::MACADDROID),
            791 => Ok(BuiltinOid::MONEYARRAYOID),
            1003 => Ok(BuiltinOid::NAMEARRAYOID),
            19 => Ok(BuiltinOid::NAMEOID),
            1986 => Ok(BuiltinOid::NAME_BTREE_FAM_OID),
            1974 => Ok(BuiltinOid::NETWORK_BTREE_FAM_OID),
            1231 => Ok(BuiltinOid::NUMERICARRAYOID),
            1700 => Ok(BuiltinOid::NUMERICOID),
            3125 => Ok(BuiltinOid::NUMERIC_BTREE_OPS_OID),
            3907 => Ok(BuiltinOid::NUMRANGEARRAYOID),
            3906 => Ok(BuiltinOid::NUMRANGEOID),
            2615 => Ok(BuiltinOid::NamespaceRelationId),
            1028 => Ok(BuiltinOid::OIDARRAYOID),
            26 => Ok(BuiltinOid::OIDOID),
            1013 => Ok(BuiltinOid::OIDVECTORARRAYOID),
            30 => Ok(BuiltinOid::OIDVECTOROID),
            1989 => Ok(BuiltinOid::OID_BTREE_FAM_OID),
            1981 => Ok(BuiltinOid::OID_BTREE_OPS_OID),
            2282 => Ok(BuiltinOid::OPAQUEOID),
            2616 => Ok(BuiltinOid::OperatorClassRelationId),
            2753 => Ok(BuiltinOid::OperatorFamilyRelationId),
            2617 => Ok(BuiltinOid::OperatorRelationId),
            1019 => Ok(BuiltinOid::PATHARRAYOID),
            602 => Ok(BuiltinOid::PATHOID),
            32 => Ok(BuiltinOid::PGDDLCOMMANDOID),
            3402 => Ok(BuiltinOid::PGDEPENDENCIESOID),
            3361 => Ok(BuiltinOid::PGNDISTINCTOID),
            194 => Ok(BuiltinOid::PGNODETREEOID),
            3221 => Ok(BuiltinOid::PG_LSNARRAYOID),
            1017 => Ok(BuiltinOid::POINTARRAYOID),
            600 => Ok(BuiltinOid::POINTOID),
            1027 => Ok(BuiltinOid::POLYGONARRAYOID),
            604 => Ok(BuiltinOid::POLYGONOID),
            1255 => Ok(BuiltinOid::ProcedureRelationId),
            6104 => Ok(BuiltinOid::PublicationRelationId),
            2287 => Ok(BuiltinOid::RECORDARRAYOID),
            2249 => Ok(BuiltinOid::RECORDOID),
            2201 => Ok(BuiltinOid::REFCURSORARRAYOID),
            1790 => Ok(BuiltinOid::REFCURSOROID),
            2210 => Ok(BuiltinOid::REGCLASSARRAYOID),
            2205 => Ok(BuiltinOid::REGCLASSOID),
            3735 => Ok(BuiltinOid::REGCONFIGARRAYOID),
            3734 => Ok(BuiltinOid::REGCONFIGOID),
            3770 => Ok(BuiltinOid::REGDICTIONARYARRAYOID),
            3769 => Ok(BuiltinOid::REGDICTIONARYOID),
            4090 => Ok(BuiltinOid::REGNAMESPACEARRAYOID),
            4089 => Ok(BuiltinOid::REGNAMESPACEOID),
            2208 => Ok(BuiltinOid::REGOPERARRAYOID),
            2209 => Ok(BuiltinOid::REGOPERATORARRAYOID),
            2204 => Ok(BuiltinOid::REGOPERATOROID),
            2203 => Ok(BuiltinOid::REGOPEROID),
            1008 => Ok(BuiltinOid::REGPROCARRAYOID),
            2207 => Ok(BuiltinOid::REGPROCEDUREARRAYOID),
            2202 => Ok(BuiltinOid::REGPROCEDUREOID),
            24 => Ok(BuiltinOid::REGPROCOID),
            4097 => Ok(BuiltinOid::REGROLEARRAYOID),
            4096 => Ok(BuiltinOid::REGROLEOID),
            2211 => Ok(BuiltinOid::REGTYPEARRAYOID),
            2206 => Ok(BuiltinOid::REGTYPEOID),
            1024 => Ok(BuiltinOid::RELTIMEARRAYOID),
            703 => Ok(BuiltinOid::RELTIMEOID),
            1259 => Ok(BuiltinOid::RelationRelationId),
            210 => Ok(BuiltinOid::SMGROID),
            2619 => Ok(BuiltinOid::StatisticRelationId),
            1009 => Ok(BuiltinOid::TEXTARRAYOID),
            25 => Ok(BuiltinOid::TEXTOID),
            1994 => Ok(BuiltinOid::TEXT_BTREE_FAM_OID),
            3126 => Ok(BuiltinOid::TEXT_BTREE_OPS_OID),
            2095 => Ok(BuiltinOid::TEXT_PATTERN_BTREE_FAM_OID),
            4017 => Ok(BuiltinOid::TEXT_SPGIST_FAM_OID),
            1010 => Ok(BuiltinOid::TIDARRAYOID),
            27 => Ok(BuiltinOid::TIDOID),
            1183 => Ok(BuiltinOid::TIMEARRAYOID),
            1083 => Ok(BuiltinOid::TIMEOID),
            1115 => Ok(BuiltinOid::TIMESTAMPARRAYOID),
            1114 => Ok(BuiltinOid::TIMESTAMPOID),
            1185 => Ok(BuiltinOid::TIMESTAMPTZARRAYOID),
            1184 => Ok(BuiltinOid::TIMESTAMPTZOID),
            3127 => Ok(BuiltinOid::TIMESTAMPTZ_BTREE_OPS_OID),
            3128 => Ok(BuiltinOid::TIMESTAMP_BTREE_OPS_OID),
            1270 => Ok(BuiltinOid::TIMETZARRAYOID),
            1266 => Ok(BuiltinOid::TIMETZOID),
            1025 => Ok(BuiltinOid::TINTERVALARRAYOID),
            704 => Ok(BuiltinOid::TINTERVALOID),
            2279 => Ok(BuiltinOid::TRIGGEROID),
            3310 => Ok(BuiltinOid::TSM_HANDLEROID),
            3645 => Ok(BuiltinOid::TSQUERYARRAYOID),
            3615 => Ok(BuiltinOid::TSQUERYOID),
            3909 => Ok(BuiltinOid::TSRANGEARRAYOID),
            3908 => Ok(BuiltinOid::TSRANGEOID),
            3911 => Ok(BuiltinOid::TSTZRANGEARRAYOID),
            3910 => Ok(BuiltinOid::TSTZRANGEOID),
            3643 => Ok(BuiltinOid::TSVECTORARRAYOID),
            3614 => Ok(BuiltinOid::TSVECTOROID),
            2949 => Ok(BuiltinOid::TXID_SNAPSHOTARRAYOID),
            2970 => Ok(BuiltinOid::TXID_SNAPSHOTOID),
            1213 => Ok(BuiltinOid::TableSpaceRelationId),
            1 => Ok(BuiltinOid::TemplateDbOid),
            2620 => Ok(BuiltinOid::TriggerRelationId),
            1247 => Ok(BuiltinOid::TypeRelationId),
            705 => Ok(BuiltinOid::UNKNOWNOID),
            2951 => Ok(BuiltinOid::UUIDARRAYOID),
            2950 => Ok(BuiltinOid::UUIDOID),
            1563 => Ok(BuiltinOid::VARBITARRAYOID),
            1562 => Ok(BuiltinOid::VARBITOID),
            1015 => Ok(BuiltinOid::VARCHARARRAYOID),
            1043 => Ok(BuiltinOid::VARCHAROID),
            2278 => Ok(BuiltinOid::VOIDOID),
            1011 => Ok(BuiltinOid::XIDARRAYOID),
            28 => Ok(BuiltinOid::XIDOID),
            143 => Ok(BuiltinOid::XMLARRAYOID),
            142 => Ok(BuiltinOid::XMLOID),
            _ => Err(NotBuiltinOid::Ambiguous),
        }
    }
}
