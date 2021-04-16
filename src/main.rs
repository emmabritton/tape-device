use anyhow::Result;
use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

mod common;
mod compiler;
mod constants;
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
            SubCommand::with_name("compile").arg(
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
        .arg(
            Arg::with_name("tape")
                .help("Execute device tape")
                .takes_value(true)
                .multiple(false)
                .required(true),
        )
        .arg(
            Arg::with_name("input")
                .help("Optional data tape for tape to use")
                .takes_value(true)
                .multiple(false),
        )
        .arg(
            Arg::with_name("debug_pc")
                .long("debug_pc")
                .help("Print PC for each statment")
                .takes_value(false)
                .multiple(false),
        )
        .get_matches();

    if matches.is_present("tape") {
        device::start(
            matches.value_of("tape").unwrap(),
            matches.value_of("input"),
            matches.is_present("debug_pc"),
        )?;
    } else if let Some(matches) = matches.subcommand_matches("compile") {
        compiler::start(matches.values_of("file").unwrap().collect())?;
    } else if let Some(matches) = matches.subcommand_matches("decompile") {
        decompiler::start(matches.value_of("file").unwrap())?;
    }

    Ok(())
}
