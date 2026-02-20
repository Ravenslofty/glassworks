// SPDX-License-Identifier: ISC

use nom::{
    IResult, Parser,
    combinator::{map, map_opt},
    multi::length_data,
    number::complete::{be_u8, be_u16},
};

pub fn parse_length_string(input: &[u8]) -> IResult<&[u8], String> {
    map_opt(length_data(be_u16), |text: &[u8]| {
        str::from_utf8(text).ok().map(|text| text.to_string())
    })
    .parse(input)
}

pub fn parse_u8_bool(input: &[u8]) -> IResult<&[u8], bool> {
    map(be_u8, |x| x == 0x1).parse(input)
}
