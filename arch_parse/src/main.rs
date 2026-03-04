// SPDX-License-Identifier: ISC

mod cfg;
mod dev;
mod picture;
mod util;

use cfg::Config;
use dev::Device;

use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

use clap::{Command, arg, value_parser};

fn main() -> io::Result<()> {
    let matches = Command::new("ArchParse")
        .version("0.1.0")
        .about("Parses the various binary formats describing MPA1000 family FPGAs")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("dev")
                .about("Parse and Dump Device Architecture Description")
                .arg(arg!(<input>).value_parser(value_parser!(PathBuf)))
                .arg(
                    arg!(-o --output <output>)
                        .required(false)
                        .value_parser(value_parser!(PathBuf)),
                ).subcommand(
                    Command::new("floorplan")
                    .about("Dump the floorplan array from Device Architecture Description")
                ).subcommand(
                    Command::new("location")
                    .about("Dump the cell name(s) from floorplan location in Device Architecture Description")
                    .arg(arg!(<x>)
                            .value_parser(value_parser!(usize)))
                    .arg(arg!(<y>)
                            .value_parser(value_parser!(usize)))
                    .arg(arg!([z]).required(false)
                            .value_parser(value_parser!(usize))),
                ),
        )
        .subcommand(
            Command::new("cfg")
                .about("Parse and Dump Device Configuration Description")
                .arg(arg!(<input>).value_parser(value_parser!(PathBuf)))
                .arg(
                    arg!(-o --output <output>)
                        .required(false)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("dev", dev_matches)) => {
            let filepath = dev_matches.get_one::<PathBuf>("input").expect("required");
            let file = File::open(&filepath)?;
            println!("{}:", filepath.display());

            let mut output: Box<dyn Write> = match dev_matches.get_one::<PathBuf>("output") {
                Some(out_path) => {
                    println!("Output writing to {}", out_path.display());
                    Box::new(BufWriter::new(File::create(&out_path)?))
                }
                None => Box::new(io::stdout()),
            };

            let device = Device::new(file).unwrap();

            match dev_matches.subcommand() {
                Some(("floorplan", _floorplan_matches)) => {
                    device.dump_floorplan(output)?;
                }
                Some(("location", loc_matches)) => {
                    let x = loc_matches
                        .get_one::<usize>("x")
                        .copied()
                        .expect("required");
                    let y = loc_matches
                        .get_one::<usize>("y")
                        .copied()
                        .expect("required");
                    let z = loc_matches.get_one::<usize>("z").copied();

                    device.dump_location(output, x, y, z)?;
                }
                None => write!(output, "{device:#?}")?,
                _ => unreachable!("No other subcommands"),
            }
        }
        Some(("cfg", dev_matches)) => {
            let filepath = dev_matches.get_one::<PathBuf>("input").expect("required");
            let file = File::open(&filepath)?;
            println!("{}:", filepath.display());

            let mut output: Box<dyn Write> = match dev_matches.get_one::<PathBuf>("output") {
                Some(out_path) => {
                    println!("Output writing to {}", out_path.display());
                    Box::new(BufWriter::new(File::create(&out_path)?))
                }
                None => Box::new(io::stdout()),
            };

            let device = Config::new(file).unwrap();
            write!(output, "{device:#?}")?;
        }
        _ => unreachable!("All subcommands matched and subcommand_required prevents None"),
    }

    Ok(())
}
