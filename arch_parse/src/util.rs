// SPDX-License-Identifier: ISC

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until},
    combinator::{map, map_opt},
    multi::length_data,
    number::complete::{be_i16, be_u8, be_u16},
};

pub fn parse_length_string(input: &[u8]) -> IResult<&[u8], String> {
    map_opt(length_data(be_u16), |text| {
        str::from_utf8(text).ok().map(|text| text.to_string())
    })
    .parse(input)
}

const NUL: &[u8] = &[0_u8];

pub fn parse_nul_term_string(input: &[u8]) -> IResult<&[u8], String> {
    map_opt((take_until(NUL), tag(NUL)), |(text, _)| {
        str::from_utf8(text).ok().map(|text| text.to_string())
    })
    .parse(input)
}

pub fn parse_u8_bool(input: &[u8]) -> IResult<&[u8], bool> {
    map(be_u8, |x| x == 0x1).parse(input)
}

pub fn parse_3_be_u16(input: &[u8]) -> IResult<&[u8], (u16, u16, u16)> {
    (be_u16, be_u16, be_u16).parse(input)
}

pub fn parse_3_be_i16(input: &[u8]) -> IResult<&[u8], (i16, i16, i16)> {
    (be_i16, be_i16, be_i16).parse(input)
}
