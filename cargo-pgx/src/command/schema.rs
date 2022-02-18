use crate::{
    command::get::{find_control_file, get_property},
    CommandExecute,
};
use colored::Colorize;
use eyre::{eyre, WrapErr};
use pgx_utils::{
    pg_config::{PgConfig, Pgx},
    sql_entity_graph::{SqlGraphEntity, PgxSql},
    pgx_pg_sys_stub::PgxPgSysStub,
};
use std::{
    collections::HashSet,
    path::PathBuf,
    process::{Command, Stdio},
};
use symbolic::{
    common::{ByteView, DSymPathExt},
    debuginfo::{Archive, SymbolIterator},
};

/// Generate extension schema files
///
/// The SQL generation process requires configuring a few settings in the crate.
/// Normally `cargo pgx schema --force-default` can set these automatically.
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct Schema {
    /// Build in test mode (for `cargo pgx test`)
    #[clap(long)]
    test: bool,
    /// Do you want to run against Postgres `pg10`, `pg11`, `pg12`, `pg13`, `pg14`?
    pg_version: Option<String>,
    /// Compile for release mode (default is debug)
    #[clap(env = "PROFILE", long, short)]
    release: bool,
    /// The `pg_config` path (default is first in $PATH)
    #[clap(long, short = 'c', parse(from_os_str))]
    pg_config: Option<PathBuf>,
    #[clap(flatten)]
    features: clap_cargo::Features,
    /// A path to output a produced SQL file (default is `sql/$EXTNAME-$VERSION.sql`)
    #[clap(long, short, parse(from_os_str))]
    out: Option<PathBuf>,
    /// A path to output a produced GraphViz DOT file
    #[clap(long, short, parse(from_os_str))]
    dot: Option<PathBuf>,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
}

impl CommandExecute for Schema {
    #[tracing::instrument(level = "error", skip(self))]
    fn execute(self) -> eyre::Result<()> {
        let (_, extname) = crate::command::get::find_control_file()?;
        let metadata = crate::metadata::metadata(&Default::default())?;
        crate::metadata::validate(&metadata)?;
        let manifest = crate::manifest::manifest(&metadata)?;

        let out = match self.out {
            Some(out) => out,
            None => format!(
                "sql/{}-{}.sql",
                extname,
                crate::command::install::get_version()?,
            )
            .into(),
        };

        let log_level = if let Ok(log_level) = std::env::var("RUST_LOG") {
            Some(log_level)
        } else {
            match self.verbose {
                0 => Some("warn".into()),
                1 => Some("info".into()),
                2 => Some("debug".into()),
                _ => Some("trace".into()),
            }
        };

        let (pg_config, pg_version) = match self.pg_config {
            None => match self.pg_version {
                None => {
                    let pg_version = match self.pg_version {
                        Some(s) => s,
                        None => crate::manifest::default_pg_version(&manifest)
                            .ok_or(eyre!("No provided `pg$VERSION` flag."))?,
                    };
                    (Pgx::from_config()?.get(&pg_version)?.clone(), pg_version)
                }
                Some(pgver) => (Pgx::from_config()?.get(&pgver)?.clone(), pgver),
            },
            Some(config) => {
                let pg_config = PgConfig::new(PathBuf::from(config));
                let pg_version = format!("pg{}", pg_config.major_version()?);
                (pg_config, pg_version)
            }
        };

        let features = crate::manifest::features_for_version(self.features, &manifest, &pg_version);

        generate_schema(
            &manifest,
            &pg_config,
            self.release,
            self.test,
            &features,
            &out,
            self.dot,
            log_level,
        )
    }
}

#[tracing::instrument(level = "error", skip_all, fields(
    pg_version = %pg_config.version()?,
    release = is_release,
    test = is_test,
    path = %path.as_ref().display(),
    dot,
    features = ?features.features,
))]
pub(crate) fn generate_schema(
    manifest: &cargo_toml::Manifest,
    pg_config: &PgConfig,
    is_release: bool,
    is_test: bool,
    features: &clap_cargo::Features,
    path: impl AsRef<std::path::Path>,
    dot: Option<impl AsRef<std::path::Path>>,
    log_level: Option<String>,
) -> eyre::Result<()> {
    let (control_file, _extname) = find_control_file()?;
    let package_name = &manifest.package
        .as_ref()
        .ok_or_else(|| eyre!("Could not find crate name in Cargo.toml."))?
        .name;

    if get_property("relocatable")? != Some("false".into()) {
        return Err(eyre!(
            "{}:  The `relocatable` property MUST be `false`.  Please update your .control file.",
            control_file.display()
        ));
    }

    let flags = std::env::var("PGX_BUILD_FLAGS").unwrap_or_default();

    
    let mut target_dir_with_profile = pgx_utils::get_target_dir()?;
    target_dir_with_profile.push(if is_release { "release" } else { "debug" });

    // First, build the SQL generator so we can get a look at the symbol table
    let mut command = Command::new("cargo");
    if is_test {
        command.arg("test");
        command.arg("--no-run");
    } else {
        command.arg("build");
    }
    if is_release {
        command.arg("--release");
    }

    if let Some(log_level) = &log_level {
        command.env("RUST_LOG", log_level);
    }

    let features_arg = features.features.join(" ");
    if !features_arg.trim().is_empty() {
        command.arg("--features");
        command.arg(&features_arg);
    }

    if features.no_default_features {
        command.arg("--no-default-features");
    }

    if features.all_features {
        command.arg("--all-features");
    }

    command.arg("--message-format=json");

    for arg in flags.split_ascii_whitespace() {
        command.arg(arg);
    }

    let command = command.stderr(Stdio::inherit());
    let command_str = format!("{:?}", command);
    println!(
        "{} for SQL generation with features `{}`\n{}",
        "    Building".bold().green(),
        features_arg,
        command_str
    );
    let cargo_output = command
        .output()
        .wrap_err_with(|| format!("failed to spawn cargo: {}", command_str))?;
    if !cargo_output.status.success() {
        return Err(eyre!("failed to build SQL generator"));
    }

    println!("{} `pgx-pg-sys` stubs to `dlopen` during SQL entity discovery", "    Creating".bold().green());
    let cargo_stdout_bytes = cargo_output.stdout;
    let cargo_stdout_stream = serde_json::Deserializer::from_slice(&cargo_stdout_bytes).into_iter::<serde_json::Value>();

    let mut pgx_pg_sys_out_dir = None;
    for stdout_stream_item in cargo_stdout_stream {
        let stdout_stream_item = stdout_stream_item?;
        if let Some(stdout_stream_object) = stdout_stream_item.as_object() {
            if let Some(reason) = stdout_stream_object.get("reason").and_then(serde_json::Value::as_str) {
                if reason != "build-script-executed" {
                    continue
                }
            }
            if let Some(package_id) = stdout_stream_object.get("package_id").and_then(serde_json::Value::as_str) {
                if !package_id.starts_with("pgx-pg-sys") {
                    continue
                }
            }
            if let Some(out_dir) = stdout_stream_object.get("out_dir").and_then(serde_json::Value::as_str) {
                pgx_pg_sys_out_dir = Some(out_dir.to_string());
                break;
            }
        }
    }
    let pgx_pg_sys_out_dir = pgx_pg_sys_out_dir.ok_or(eyre!("Could not get `pgx-pg-sys` `out_dir` from Cargo output."))?;
    let pgx_pg_sys_out_dir = PathBuf::from(pgx_pg_sys_out_dir);

    let pg_version = pg_config.major_version()?;
    let pg_version_file_name = format!("pg{}.rs", pg_version);
    // Create stubbed `pgx_pg_sys` bindings for the generator to link with.
    let mut pgx_pg_sys_stub_file = target_dir_with_profile.clone();
    pgx_pg_sys_stub_file.push("pg_sys_stub");
    pgx_pg_sys_stub_file.push(&pg_version_file_name);
    
    let mut pgx_pg_sys_file = PathBuf::from(pgx_pg_sys_out_dir);
    pgx_pg_sys_file.push(&pg_version_file_name);

    PgxPgSysStub::from_file(&pgx_pg_sys_file)?
        .write_to_file(&pgx_pg_sys_stub_file)?;
    
    let mut pgx_pg_sys_stub_built = target_dir_with_profile.clone();
    pgx_pg_sys_stub_built.push("pg_sys_stub");
    pgx_pg_sys_stub_built.push("pgx_pg_sys_stub.so");
    
    let mut so_rustc_invocation = Command::new("rustc");
    so_rustc_invocation.args([
            "--crate-type", "cdylib",
            "-o", pgx_pg_sys_stub_built.to_str().unwrap(),
            pgx_pg_sys_stub_file.to_str().unwrap(),
        ]);
    
    let so_rustc_invocation_str = format!("{:?}", so_rustc_invocation);
    tracing::debug!(command = %so_rustc_invocation_str, "Running");
    let output = so_rustc_invocation.output()
        .wrap_err_with(||
            eyre!("Could not invoke `rustc` on {}", &pgx_pg_sys_stub_file.display())
        )?;

    let code = output.status.code().unwrap();
    tracing::trace!(status_code = %code, command = %so_rustc_invocation_str, "Finished");

    // Inspect the symbol table for a list of `__pgx_internals` we should have the generator call
    let mut lib_so = target_dir_with_profile.clone();

    let so_extension = if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    };

    lib_so.push(&format!("lib{}{}", package_name.replace("-", "_"), so_extension));

    println!("{} SQL entities", " Discovering".bold().green(),);
    let dsym_path = lib_so.resolve_dsym();
    let buffer = ByteView::open(dsym_path.as_deref().unwrap_or(&lib_so))
        .wrap_err_with(||
            eyre!("Could not get byte view into {}", &dsym_path.as_deref().unwrap_or(&lib_so).display())
        )?;
    let archive = Archive::parse(&buffer).expect("Could not parse archive");

    // Some users reported experiencing duplicate entries if we don't ensure `fns_to_call`
    // has unique entries.
    let mut fns_to_call = HashSet::new();
    for object in archive.objects() {
        match object {
            Ok(object) => match object.symbols() {
                SymbolIterator::Elf(iter) => {
                    for symbol in iter {
                        if let Some(name) = symbol.name {
                            if name.starts_with("__pgx_internals") || name.starts_with("__pgx_marker") {
                                fns_to_call.insert(name);
                            }
                        }
                    }
                }
                SymbolIterator::MachO(iter) => {
                    for symbol in iter {
                        if let Some(name) = symbol.name {
                            if name.starts_with("__pgx_internals") || name.starts_with("__pgx_marker") {
                                fns_to_call.insert(name);
                            }
                        }
                    }
                }
                _ => panic!("Unable to parse non-ELF or Mach0 symbols. (Please report this, we can  probably fix this!)"),
            },
            Err(e) => {
                panic!("Got error inspecting objects: {}", e);
            }
        }
    }
    let mut seen_schemas = Vec::new();
    let mut num_funcs = 0_usize;
    let mut num_types = 0_usize;
    let mut num_enums = 0_usize;
    let mut num_sqls = 0_usize;
    let mut num_ords = 0_usize;
    let mut num_hashes = 0_usize;
    let mut num_aggregates = 0_usize;
    for func in &fns_to_call {
        if func.starts_with("__pgx_internals_schema_") {
            let schema = func
                .split("_")
                .skip(5)
                .next()
                .expect("Schema extern name was not of expected format");
            seen_schemas.push(schema);
        } else if func.starts_with("__pgx_internals_fn_") {
            num_funcs += 1;
        } else if func.starts_with("__pgx_internals_type_") {
            num_types += 1;
        } else if func.starts_with("__pgx_internals_enum_") {
            num_enums += 1;
        } else if func.starts_with("__pgx_internals_sql_") {
            num_sqls += 1;
        } else if func.starts_with("__pgx_internals_ord_") {
            num_ords += 1;
        } else if func.starts_with("__pgx_internals_hash_") {
            num_hashes += 1;
        } else if func.starts_with("__pgx_internals_aggregate_") {
            num_aggregates += 1;
        }
    }

    println!(
        "{} {} SQL entities: {} schemas ({} unique), {} functions, {} types, {} enums, {} sqls, {} ords, {} hashes, {} aggregates",
        "  Discovered".bold().green(),
        fns_to_call.len().to_string().bold().cyan(),
        seen_schemas.iter().count().to_string().bold().cyan(),
        seen_schemas.iter().collect::<std::collections::HashSet<_>>().iter().count().to_string().bold().cyan(),
        num_funcs.to_string().bold().cyan(),
        num_types.to_string().bold().cyan(),
        num_enums.to_string().bold().cyan(),
        num_sqls.to_string().bold().cyan(),
        num_ords.to_string().bold().cyan(),
        num_hashes.to_string().bold().cyan(),
        num_aggregates.to_string().bold().cyan(),
    );

    let path = path.as_ref();
    let _ = path.parent().map(|p| std::fs::create_dir_all(&p).unwrap());

    tracing::debug!("Collecting {} SQL entities", fns_to_call.len());
    let mut entities = Vec::default();
    let typeid_sql_mapping;
    let source_only_sql_mapping;

    unsafe {
        let _pgx_pg_sys = libloading::os::unix::Library::open(
            Some(&pgx_pg_sys_stub_built),
            libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL,
        ).expect(&format!("Couldn't libload {}", pgx_pg_sys_stub_built.display()));

        let lib = libloading::os::unix::Library::open(
            Some(&lib_so),
            libloading::os::unix::RTLD_LAZY,
        ).expect(&format!("Couldn't libload {}", lib_so.display()));

        let typeid_sql_mappings_symbol: libloading::os::unix::Symbol<
            unsafe extern fn() -> std::collections::HashSet<pgx_utils::sql_entity_graph::RustSqlMapping>
        > = lib.get("__pgx_typeid_sql_mappings".as_bytes()).expect(&format!("Couldn't call __pgx_typeid_sql_mappings"));
        typeid_sql_mapping = typeid_sql_mappings_symbol();
        let source_only_sql_mapping_symbol: libloading::os::unix::Symbol<
            unsafe extern fn() -> std::collections::HashSet<pgx_utils::sql_entity_graph::RustSourceOnlySqlMapping>
        > = lib.get("__pgx_source_only_sql_mappings".as_bytes()).expect(&format!("Couldn't call __pgx_source_only_sql_mappings"));
        source_only_sql_mapping = source_only_sql_mapping_symbol();

        let symbol: libloading::os::unix::Symbol<
            unsafe extern fn() -> SqlGraphEntity
        > = lib.get("__pgx_marker".as_bytes()).expect(&format!("Couldn't call __pgx_marker"));
        let control_file_entity = symbol();
        entities.push(
            control_file_entity
        );

        for symbol_to_call in fns_to_call {
            let symbol: libloading::os::unix::Symbol<
                unsafe extern fn() -> SqlGraphEntity
            > = lib.get(symbol_to_call.as_bytes()).expect(&format!("Couldn't call {:#?}", symbol_to_call));
            let entity = symbol();
            entities.push(entity);
        }
    };

    let pgx_sql = PgxSql::build(
        typeid_sql_mapping.clone().into_iter(),
        source_only_sql_mapping.clone().into_iter(),
        entities.into_iter()
    ).unwrap();

    tracing::debug!("Writing SQL");
    pgx_sql.to_file(path)
        .wrap_err_with(|| eyre!("Could not write SQL to {}", path.display()))?;

    if let Some(dot_path) = dot {
        let dot_path = dot_path.as_ref();
        tracing::info!(dot = %dot_path.display(), "Writing Graphviz DOT");
        pgx_sql.to_dot(dot_path)?;
    }
    Ok(())
}