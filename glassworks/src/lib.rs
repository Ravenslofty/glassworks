use std::io::{Read, self};

pub fn ecb_calc(row: &[u8]) -> u8 {
    row.iter().map(|b| *b as u16).reduce(|acc, b| {
        let acc_carry = (acc >> 8) ^ 1;
        let acc_value = acc & 0xff;
        acc_value + b + acc_carry
    }).unwrap() as u8
}

#[derive(Clone, Copy)]
pub enum Device {
    Mpa1016,
    Mpa1036,
    Mpa1064,
    Mpa1100,
}

impl Device {
    pub fn try_from_jtag(idcode: u32) -> Option<Self> {
        const MPA1016_JTAG_ID: u32 = 0x1390E01D;
        const MPA1036_JTAG_ID: u32 = 0x1391E01D;
        const MPA1064_JTAG_ID: u32 = 0x1393401D;
        const MPA1100_JTAG_ID: u32 = 0x1392001D;
        match idcode {
            MPA1016_JTAG_ID => Some(Self::Mpa1016),
            MPA1036_JTAG_ID => Some(Self::Mpa1036),
            MPA1064_JTAG_ID => Some(Self::Mpa1064),
            MPA1100_JTAG_ID => Some(Self::Mpa1100),
            _ => None
        }
    }

    pub const fn rows(self) -> usize {
        match self {
            Self::Mpa1016 => 95,
            Self::Mpa1036 => 139,
            Self::Mpa1064 => 183,
            Self::Mpa1100 => 227,
        }
    }

    pub const fn bytes_per_row(self) -> usize {
        match self {
            Self::Mpa1016 => 576 / 8,
            Self::Mpa1036 => 840 / 8,
            Self::Mpa1064 => 1104 / 8,
            Self::Mpa1100 => 1360 / 8,
        }
    }
}

struct Bitstream {
    device: Device,
    data_type: u8,
}

impl Bitstream {
    pub fn new<R: Read>(mut input: R) -> io::Result<Self> {
        // Bytes 0-3: JTAG ID (big-endian)
        let mut jtag_id = [0; 4];
        input.read_exact(&mut jtag_id)?;

        let idcode = u32::from_be_bytes(jtag_id);
        let device = if let Some(device) = Device::try_from_jtag(idcode) {
            device
        } else {
            panic!("Unrecognised JTAG IDCODE {idcode:08x}");
        };

        // Byte 4: data type
        let mut data_type = [0; 1];
        input.read_exact(&mut data_type)?;
        let data_type = data_type[0];

        assert_eq!(data_type & 0x1, 0, "test data mode not yet implemented");
        assert_eq!(data_type & 0x2, 0, "encrypted data mode unsupported");
        assert_eq!(data_type & 0x4, 0, "compressed data mode unsupported");
        assert_eq!(data_type & 0xF8, 0, "must-be-zero section not zero");

        // for each row:
        for row_index in 0..device.rows() {
            let mut row = vec![0; device.bytes_per_row()];
            input.read_exact(&mut row)?;

            for column_index in 0..(device.bytes_per_row()-1) {
                for bit in 0..8 {
                    if (row[column_index] & (1 << bit)) != 0 {
                        println!("{row_index}:{column_index}:{bit}");
                    }
                }
            }

            // Last byte: error check byte
            assert_eq!(ecb_calc(&row[0..device.bytes_per_row()-1]), row[device.bytes_per_row()-1], "ECB checksum mismatch for row {row_index}");
        }

        Ok(Self {
            device,
            data_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ecb_calc, Bitstream};

    #[test]
    fn ecb_calc_is_correct() {
        let test = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(ecb_calc(&test), 0x67);
        let test = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x01, 0x00, 0x00, 0x10, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(ecb_calc(&test), 0xAB);
    }

    #[test]
    fn import_mpa1036_bitstream() {
        let bytes = include_bytes!("and_mpa1036.bit");
        let _ = Bitstream::new(bytes.as_slice()).unwrap();
    }

    #[test]
    fn import_mpa1100_bitstream() {
        let bytes = include_bytes!("nor_mpa1100.bit");
        let _ = Bitstream::new(bytes.as_slice()).unwrap();
    }
}
