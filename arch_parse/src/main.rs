// SPDX-License-Identifier: ISC

mod dev;
mod picture;
mod util;

use dev::Device;

use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    for filename in std::env::args().skip(1) {
        let file = File::open(&filename)?;
        println!("{filename}:");

        let device = Device::new(file).unwrap();
        println!("{device:#?}");
    }

    Ok(())
}
