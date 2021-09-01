/* 
This file is auto generated by pgx.

The ordering of items is not stable, it is driven by a dependency graph.
*/

-- src/lib.rs:19
CREATE SCHEMA IF NOT EXISTS some_schema; /* schemas::some_schema */

-- src/lib.rs:27
-- schemas::some_schema::hello_some_schema
CREATE OR REPLACE FUNCTION some_schema."hello_some_schema"() RETURNS text /* &str */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'hello_some_schema_wrapper';

-- src/lib.rs:12
-- schemas::hello_default_schema
CREATE OR REPLACE FUNCTION "hello_default_schema"() RETURNS text /* &str */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'hello_default_schema_wrapper';

-- src/lib.rs:51
-- schemas::public::hello_public
CREATE OR REPLACE FUNCTION public."hello_public"() RETURNS text /* &str */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'hello_public_wrapper';

-- src/lib.rs:24
-- schemas::some_schema::MySomeSchemaType
CREATE TYPE some_schema.MySomeSchemaType;

-- src/lib.rs:24
-- schemas::some_schema::mysomeschematype_in
CREATE OR REPLACE FUNCTION some_schema."mysomeschematype_in"(
	"input" cstring /* &std::ffi::c_str::CStr */
) RETURNS some_schema.MySomeSchemaType /* schemas::some_schema::MySomeSchemaType */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'mysomeschematype_in_wrapper';

-- src/lib.rs:24
-- schemas::some_schema::mysomeschematype_out
CREATE OR REPLACE FUNCTION some_schema."mysomeschematype_out"(
	"input" some_schema.MySomeSchemaType /* schemas::some_schema::MySomeSchemaType */
) RETURNS cstring /* &std::ffi::c_str::CStr */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'mysomeschematype_out_wrapper';

-- src/lib.rs:24
-- schemas::some_schema::MySomeSchemaType
CREATE TYPE some_schema.MySomeSchemaType (
	INTERNALLENGTH = variable,
	INPUT = some_schema.mysomeschematype_in, /* schemas::some_schema::mysomeschematype_in */
	OUTPUT = some_schema.mysomeschematype_out, /* schemas::some_schema::mysomeschematype_out */
	STORAGE = extended
);

-- src/lib.rs:41
-- schemas::pg_catalog::MyPgCatalogType
CREATE TYPE pg_catalog.MyPgCatalogType;

-- src/lib.rs:41
-- schemas::pg_catalog::mypgcatalogtype_in
CREATE OR REPLACE FUNCTION pg_catalog."mypgcatalogtype_in"(
	"input" cstring /* &std::ffi::c_str::CStr */
) RETURNS pg_catalog.MyPgCatalogType /* schemas::pg_catalog::MyPgCatalogType */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'mypgcatalogtype_in_wrapper';

-- src/lib.rs:41
-- schemas::pg_catalog::mypgcatalogtype_out
CREATE OR REPLACE FUNCTION pg_catalog."mypgcatalogtype_out"(
	"input" pg_catalog.MyPgCatalogType /* schemas::pg_catalog::MyPgCatalogType */
) RETURNS cstring /* &std::ffi::c_str::CStr */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'mypgcatalogtype_out_wrapper';

-- src/lib.rs:41
-- schemas::pg_catalog::MyPgCatalogType
CREATE TYPE pg_catalog.MyPgCatalogType (
	INTERNALLENGTH = variable,
	INPUT = pg_catalog.mypgcatalogtype_in, /* schemas::pg_catalog::mypgcatalogtype_in */
	OUTPUT = pg_catalog.mypgcatalogtype_out, /* schemas::pg_catalog::mypgcatalogtype_out */
	STORAGE = extended
);

-- src/lib.rs:9
-- schemas::MyType
CREATE TYPE MyType;

-- src/lib.rs:9
-- schemas::mytype_in
CREATE OR REPLACE FUNCTION "mytype_in"(
	"input" cstring /* &std::ffi::c_str::CStr */
) RETURNS MyType /* schemas::MyType */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'mytype_in_wrapper';

-- src/lib.rs:9
-- schemas::mytype_out
CREATE OR REPLACE FUNCTION "mytype_out"(
	"input" MyType /* schemas::MyType */
) RETURNS cstring /* &std::ffi::c_str::CStr */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'mytype_out_wrapper';

-- src/lib.rs:9
-- schemas::MyType
CREATE TYPE MyType (
	INTERNALLENGTH = variable,
	INPUT = mytype_in, /* schemas::mytype_in */
	OUTPUT = mytype_out, /* schemas::mytype_out */
	STORAGE = extended
);
