#![feature(destructuring_assignment)]

use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

mod common;
mod compiler;
mod constants;
mod debugger;
mod decompiler;
mod device;
mod printer;
mod tape_reader;

fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::SubcommandsNegateReqs,
            AppSettings::VersionlessSubcommands,
        ])
        .subcommand(
            SubCommand::with_name("compile")
                .arg(
                    Arg::with_name("keep_whitespace")
                        .long("keep_whitespace")
                        .help("Keep whitespace at end of strings")
                        .takes_value(false)
                        .multiple(false),
                )
                .arg(
                    Arg::with_name("file")
                        .help("Compile .tasm and .str into .tape")
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
            SubCommand::with_name("debug")
                .arg(
                    Arg::with_name("tape")
                        .help("Device tape to debug")
                        .takes_value(true)
                        .multiple(false)
                        .required(true),
                )
                .arg(
                    Arg::with_name("input")
                        .help("Optional data tape")
                        .takes_value(true)
                        .multiple(false),
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
                .help("Optional data tape")
                .takes_value(true)
                .multiple(false),
        )
        .get_matches();

    if matches.is_present("tape") {
        device::start(matches.value_of("tape").unwrap(), matches.value_of("input"))?;
    } else if let Some(matches) = matches.subcommand_matches("debug") {
        debugger::start(matches.value_of("tape").unwrap(), matches.value_of("input"))?;
    } else if let Some(matches) = matches.subcommand_matches("compile") {
        compiler::start(
            matches.values_of("file").unwrap().collect(),
            matches.is_present("keep_whitespace"),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("decompile") {
        decompiler::start(matches.value_of("file").unwrap())?;
    }

    Ok(())
}
