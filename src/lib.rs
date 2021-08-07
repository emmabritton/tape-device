#![feature(destructuring_assignment)]

#[macro_use]
extern crate bitflags;

use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand, Values};
use git_version::git_version;

pub mod assembler;
pub mod common;
pub mod constants;
pub mod decompiler;
pub mod device;
pub mod language;
pub mod tape_reader;

pub fn run() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(format!("{}-{}", crate_version!(), git_version!()).as_str())
        .author(crate_authors!())
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::SubcommandsNegateReqs,
            AppSettings::VersionlessSubcommands,
        ])
        .subcommand(
            SubCommand::with_name("assemble")
                .arg(
                    Arg::with_name("file")
                        .help("Compile .basm into .tape")
                        .takes_value(true)
                        .min_values(1)
                        .max_values(2)
                        .required(true),
                )
                .arg(
                    Arg::with_name("build_debug")
                        .help("Print assembler interpretation of program")
                        .takes_value(false)
                        .long("--save-intermediate")
                        .short("-i")
                        .required(false)
                        .multiple(false),
                )
                .arg(
                    Arg::with_name("debug")
                        .help("Output data for debugger")
                        .takes_value(false)
                        .long("--save-debug")
                        .short("-d")
                        .required(false)
                        .multiple(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("decompile").arg(
                Arg::with_name("file")
                    .help("Decompile .tape")
                    .takes_value(true)
                    .multiple(false)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("debug")
                .arg(
                    Arg::with_name("tape")
                        .help("Device tape to debug")
                        .takes_value(true)
                        .multiple(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("debug_file")
                        .help("Debug info file")
                        .takes_value(true)
                        .multiple(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("input")
                        .help("Data tape for reading/writing")
                        .takes_value(true)
                        .multiple(true)
                        .required(false),
                ),
        )
        .arg(
            Arg::with_name("tape")
                .help("Device tape to execute")
                .takes_value(true)
                .multiple(false)
                .required(true),
        )
        .arg(
            Arg::with_name("input")
                .help("Data tape for reading/writing")
                .takes_value(true)
                .multiple(true)
                .required(false),
        )
        .arg(
            Arg::with_name("piped")
                .help("Start in piped mode")
                .takes_value(false)
                .multiple(false)
                .required(false)
                .long("piped"),
        )
        .get_matches();

    if matches.is_present("tape") {
        if matches.is_present("piped") {
            device::start_piped(
                matches.value_of("tape").unwrap(),
                validate(convert(matches.values_of("input"))),
            )?;
        } else {
            device::start(
                matches.value_of("tape").unwrap(),
                validate(convert(matches.values_of("input"))),
            )?;
        }
    } else if let Some(matches) = matches.subcommand_matches("debug") {
        device::start_debug(
            matches.value_of("tape").unwrap(),
            matches.value_of("debug_file").unwrap(),
            validate(convert(matches.values_of("input"))),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("assemble") {
        assembler::start(
            matches.value_of("file").unwrap(),
            matches.is_present("build_debug"),
            matches.is_present("debug"),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("decompile") {
        decompiler::start(matches.value_of("file").unwrap())?;
    }

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn convert(values: Option<Values>) -> Vec<&str> {
    if let Some(values) = values {
        values.collect()
    } else {
        vec![]
    }
}

fn validate(files: Vec<&str>) -> Vec<&str> {
    for file in files.iter() {
        if !std::path::Path::new(file).is_file() {
            eprintln!("'{}' is not a file", file);
            std::process::exit(1);
        }
    }
    files
}
