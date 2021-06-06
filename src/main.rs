#![feature(destructuring_assignment)]

#[macro_use]
extern crate bitflags;

use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand, Values};
use git_version::git_version;

mod assembler;
mod common;
mod constants;
mod decompiler;
mod device;
mod language;
mod printer;
mod tape_reader;

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(format!("{}-{}", crate_version!(), git_version!()).as_str())
        .author(crate_authors!())
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::SubcommandsNegateReqs,
            AppSettings::VersionlessSubcommands,
        ])
        .subcommand(
            SubCommand::with_name("assemble").arg(
                Arg::with_name("file")
                    .help("Compile .basm into .tape")
                    .takes_value(true)
                    .min_values(1)
                    .max_values(2)
                    .required(true),
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
            SubCommand::with_name("run")
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
        .get_matches();

    if matches.is_present("tape") {
        device::start(
            matches.value_of("tape").unwrap(),
            convert(matches.values_of("input")),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("run") {
        device::start(
            matches.value_of("tape").unwrap(),
            convert(matches.values_of("input")),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("assemble") {
        assembler::start(
            matches.value_of("file").unwrap(),
            matches.is_present("keep_whitespace"),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("decompile") {
        decompiler::start(matches.value_of("file").unwrap())?;
    }

    Ok(())
}

fn convert(values: Option<Values>) -> Vec<&str> {
    if let Some(values) = values {
        values.collect()
    } else {
        vec![]
    }
}
