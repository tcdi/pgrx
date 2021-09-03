![cargo test --all](https://github.com/zombodb/pgx/workflows/cargo%20test%20--all/badge.svg)
[![crates.io badge](https://img.shields.io/crates/v/pgx.svg)](https://crates.io/crates/pgx)
[![docs.rs badge](https://docs.rs/pgx/badge.svg)](https://docs.rs/pgx)
[![Twitter Follow](https://img.shields.io/twitter/follow/zombodb.svg?style=flat)](https://twitter.com/zombodb)
[![Discord Chat](https://img.shields.io/discord/561648697805504526.svg)](https://discord.gg/hPb93Y9)

---

![Logo](logo.png)

# pgx

###### Build Postgres Extensions with Rust!

`pgx` is a framework for developing PostgreSQL extensions in Rust and strives to be as idiomatic and safe as possible.

`pgx` supports Postgres v10, v11, v12, and v13.

Feel free to join our [Discord Server](https://discord.gg/hPb93Y9).

## Key Features

#### A Managed Development Environment
 - A cargo sub-command (`cargo-pgx`) for managing the `pgx` development environment
    - Quickly create a new extension template crate via `cargo pgx new`
    - Install, configure, compile, and privately install all required Postgres versions via `cargo pgx init`
    - Run your extension and interactively test with `psql` via `cargo pgx run`
    - Unit-test your extension across multiple Postgres versions via `cargo pgx test`
    - Create installation packages for your extension via `cargo pgx package`

#### Target Multiple Postgres Versions
 - Support Postgres v10, v11, v12, and v13 from the same codebase
    - Postgres Rust bindings are organized into `pgXX.rs` modules
 - Use Rust feature gating to use version-specific APIs
 - Seamlessly test against all versions

#### Automatic Schema Generation
 - Generates DDL for common SQL objects such as
    - Functions
    - Types
    - Enums
 - Hand-written SQL is supported through the `extension_sql!` & `extension_sql_file!` macros
 - Control the order in which SQL is executed during `CREATE EXTENSION ...;`

#### Safety First
 - Translates Rust `panic!`s into Postgres `ERROR`s that abort the transaction, not the process
 - Memory Management follows Rust's drop semantics, even in the face of `panic!` and `elog(ERROR)`
 - `#[pg_guard]` procedural macro to ensure the above
 - Postgres `Datum` is simply `Option<T> where T: FromDatum` -- `NULL` Datums are safely represented as `Option::None`
 - `#[pg_test]` proc-macro for unit testing **in-process** within Postgres

#### First-class UDF support
 - Annotate functions with `#[pg_extern]` to expose them to Postgres
 - Return `impl std::iter::Iterator<Item = T> where T: IntoDatum` for automatic set-returning-functions (both `RETURNS SETOF` and `RETURNS TABLE (...)` variants
 - DDL automatically generated

#### Most Postgres Data Types Transparently Converted to Rust

Postgres Type | Rust Type (as `Option<T>`)
--------------|-----------
`bytea` | `Vec<u8>` or `&[u8]` (zero-copy)
`text` | `String` or `&str` (zero-copy)
`varchar` | `String` or `&str` (zero-copy) or `char`
`"char"` | `i8`
`smallint` | `i16`
`integer` | `i32`
`bigint` | `i64`
`oid` | `u32`
`real` | `f32`
`double precision` | `f64`
`bool` | `bool`
`json` | `pgx::Json(serde_json::Value)`
`jsonb` | `pgx::Json(serde_json::Value)`
`date` | `pgx::Date`
`time` | `pgx::Time`
`timestamp` | `pgx::Timestamp`
`time with time zone` | `pgx::TimeWithTimeZone`
`timestamp with time zone` | `pgx::TimestampWithTimeZone`
`anyarray` | `pgx::AnyArray`
`anyelement` | `pgx::AnyElement`
`box` | `pgx::pg_sys::BOX`
`point` | `pgx::pgx_sys::Point`
`tid` | `pgx::pg_sys::ItemPointerData`
`cstring` | `&std::ffi::CStr`
`inet` | `pgx::Inet(String)` -- TODO: needs better support
`numeric` | `pgx::Numeric(String)` -- TODO: needs better support
`void` | `()`
`ARRAY[]::<type>` | `Vec<Option<T>>` or `pgx::Array<T>` (zero-copy)
`NULL` | `Option::None`
`internal` | `pgx::PgBox<T>` where `T` is any Rust/Postgres struct
`uuid` | `pgx::Uuid([u8; 16])`

There are also `IntoDatum` and `FromDatum` traits for implementing additional type conversions,
along with `#[derive(PostgresType)]` and `#[derive(PostgresEnum)]` for automatic conversion of
custom types.

#### Easy Custom Types
 - `#[derive(PostgresType)]` to use a Rust struct as a Postgres type, represented as a CBOR-encoded object in-memory/on-disk, and JSON as human-readable
 	- can provide custom implementations for custom in-memory/on-disk/human-readable representations
 - `#[derive(PostgresEnum)]` to use a Rust enum as a Postgres enum
 - DDL automatically generated

#### Server Programming Interface (SPI)
 - Safe access into SPI
 - Transparently return owned Datums from an SPI context

#### Advanced Features
 - Safe access to Postgres' `MemoryContext` system via `pgx::PgMemoryContexts`
 - Executor/planner/transaction/subtransaction hooks
 - Safely use Postgres-provided pointers with `pgx::PgBox<T>` (akin to `alloc::boxed::Box<T>`)
 - `#[pg_guard]` proc-macro for guarding `extern "C"` Rust functions that need to be passed into Postgres
 - Access Postgres' logging system through `eprintln!`-like macros
 - Direct `unsafe` access to large parts of Postgres internals via the `pgx::pg_sys` module
 - lots more!

## System Requirements

 - `cargo install rustfmt`
 - `git`
 - `libclang.so`
   - Ubuntu: `libclang-dev` or `clang`
   - RHEL: `clang`
 - [build dependencies for PostgreSQL](https://wiki.postgresql.org/wiki/Compile_and_Install_from_source_code)

## Getting Started

### 1. Install `cargo-pgx`
First you'll want to install the `pgx` cargo sub-command from crates.io. You'll use it almost exclusively during
your development and testing workflow.

```shell script
$ cargo install cargo-pgx
```

### 2. Initialize it

Next, `pgx` needs to be initialized.  You only need to do this once.

```shell script
$ cargo pgx init
```

The `init` command downloads Postgres versions v10, v11, v12, v13, compiles them to `~/.pgx/`, and runs `initdb`.
These installations are needed by `pgx` not only for auto-generating Rust bindings from each version's header files,
but also for `pgx`'s test framework.

See the documentation for [`cargo-pgx`](cargo-pgx/README.md) for details on how to limit the required postgres versions.


### 3. Create a new extension

```shell script
$ cargo pgx new my_extension
$ cd my_extension
```

This will create a new directory for the extension crate.

```
my_extension/
├── Cargo.toml
├── my_extension.control
├── sql
│   ├── lib.generated.sql
│   └── load-order.txt
└── src
    └── lib.rs
```

The new extension includes an example, so you can go ahead and run it right away.

### 4. Run your extension

```shell script
$ cargo pgx run pg13  # or pg10 or pg11 or pg12
```

This compiles the extension to a shared library, copies it to the specified Postgres installation (in `~/.pgx/`),
starts that Postgres instance and connects you, via `psql`, to a database named for the extension.

The first time, compilation takes a few minutes as `pgx` needs to generate almost 200k lines of Rust "bindings" from
Postgres' header files.

Once compiled you'll be placed in a `psql` shell, for, in this case, Postgres 13.
Now, we can just [load the extension](https://www.postgresql.org/docs/13/sql-createextension.html) and do a select on the example function.

```sql
CREATE EXTENSION my_extension;
SELECT hello_my_extension();
```

This will return something along the lines of

```
 hello_my_extension
---------------------
 Hello, my_extension
(1 row)
```

## Digging Deeper

 - [cargo-pgx sub-command](cargo-pgx/)
 - [Custom Types](pgx-examples/custom_types/)
 - [Postgres Operator Functions and Operator Classes/Families](pgx-examples/operators/)
 - [Shared Memory Support](pgx-examples/shmem/)
 - [various examples](pgx-examples/)

## Caveats & Known Issues

There's probably more than are listed here, but a primary things of note are:

 - Threading is not really supported.  Postgres is strictly single-threaded.  As such, if you do venture into using threads, those threads **MUST NOT** call *any* internal Postgres function, or otherwise use any Postgres-provided pointer.  There's also a potential problem with Postgres' use of `sigprocmask`.  This was being discussed on the -hackers list, even with a patch provided, but the conversation seems to have stalled (https://www.postgresql.org/message-id/flat/5EF20168.2040508%40anastigmatix.net#4533edb74194d30adfa04a6a2ce635ba).

 - `async` interactions are unknown right now.

 - `pgx` uses lots of `unsafe` Rust.  That's generally the nature of the beast when doing FFI wrappers, so be aware.

 - Not all of Postgres' internals are included or even wrapped.  This isn't due to it not being possible, it's simply due to it being an incredibly large task.  If you identify internal Postgres APIs you need, open an issue and we'll get them exposed, at least through the `pgx::pg_sys` module.

 - Windows is not supported.  It could be, but will require a bit of work with `cargo-pgx` and figuring out how to compile `pgx`'s "cshim" static library.

 - Sessions started before `ALTER EXTENSION my_extension UPDATE;` will continue to see the old version of `my_extension`. New sessions will see the updated version of the extension.

## TODO

There's a few things on our immediate TODO list

 - Better trigger function support.  `pgx` does support creating trigger functions in Rust (need examples!)
but it doesn't automatically generate any of the DDL for them.  This too likely needs a procmaro like `#[pg_trigger]`
 - Automatic extension schema upgrade scripts, based on diffs from a previous git tag and HEAD.  Likely, this
will be built into the `cargo-pgx` subcommand and make use of https://github.com/zombodb/postgres-parser.
 - More examples -- especially around memory management and the various derive macros `#[derive(PostgresType/Enum)]`


## Contributing

We are most definitely open to contributions of any kind.  Bug Reports, Feature Requests, Documentation,
and even [sponsorships](https://github.com/sponsors/eeeebbbbrrrr).

If you'd like to contribute code via a Pull Request, please make it against our `develop` branch.  The `master` branch is meant to represent what is currently available on crates.io.

Providing wrappers for Postgres' internals is not a straightforward task, and completely wrapping it is going
to take quite a bit of time.  `pgx` is generally ready for use now, and it will continue to be developed as
time goes on.  Your feedback about what you'd like to be able to do with `pgx` is greatly appreciated.


## License

```
Copyright 2020 ZomboDB, LLC <zombodb@gmail.com>. All rights reserved.
Use of this source code is governed by the MIT license that can be found in the LICENSE file.
```
