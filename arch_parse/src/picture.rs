// SPDX-License-Identifier: ISC

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    error::dbg_dmp,
    multi::length_count,
    number::complete::{be_i16, be_u8, be_u16, be_u32},
    sequence::preceded,
};

use crate::util::parse_length_string;

#[derive(Debug, Default)]
pub struct Picture {
    bounds: (i16, i16, i16, i16),
    commands: Vec<Command>,
}

#[derive(Debug)]
pub enum Command {
    LineTo(i16, i16),
    MoveTo(i16, i16),
    Arc(u16, u16, u16, u16),
    TextSize(u32),
    LineWidth(u32),
    TextRot(u32),
    Colour(u16),
    Text(String),
    Unknown8(u32),
    Unknown9(u32),
    Unknown10(u16, u16),
}

const LINETO: &[u8] = &[0_u8];
const MOVETO: &[u8] = &[1_u8];
const ARC: &[u8] = &[2_u8];
const TEXTSIZE: &[u8] = &[3_u8];
const LINEWIDTH: &[u8] = &[4_u8];
const TEXTROT: &[u8] = &[5_u8];
const COLOUR: &[u8] = &[6_u8];
const TEXT: &[u8] = &[7_u8];
const UNKNOWN8: &[u8] = &[8_u8];
const UNKNOWN9: &[u8] = &[9_u8];
const UNKNOWN10: &[u8] = &[10_u8];

fn parse_lineto(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(LINETO), (be_i16, be_i16)), |(x, y)| {
        Command::LineTo(x, y)
    })
    .parse(input)
}

fn parse_moveto(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(MOVETO), (be_i16, be_i16)), |(x, y)| {
        Command::MoveTo(x, y)
    })
    .parse(input)
}

fn parse_arc(input: &[u8]) -> IResult<&[u8], Command> {
    map(
        preceded(tag(ARC), (be_u16, be_u16, be_u16, be_u16)),
        |(x1, y1, x2, y2)| Command::Arc(x1, y1, x2, y2),
    )
    .parse(input)
}

fn parse_textsize(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(TEXTSIZE), be_u32), |size| {
        Command::TextSize(size)
    })
    .parse(input)
}

fn parse_linewidth(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(LINEWIDTH), be_u32), |width| {
        Command::LineWidth(width)
    })
    .parse(input)
}

fn parse_textrot(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(TEXTROT), be_u32), |rot| Command::TextSize(rot)).parse(input)
}

fn parse_colour(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(COLOUR), be_u16), |colour| {
        Command::Colour(colour)
    })
    .parse(input)
}

fn parse_text(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(TEXT), parse_length_string), |text| {
        Command::Text(text)
    })
    .parse(input)
}

fn parse_unknown8(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(UNKNOWN8), be_u32), |unk| {
        Command::Unknown8(unk)
    })
    .parse(input)
}

fn parse_unknown9(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(UNKNOWN9), be_u32), |unk| {
        Command::Unknown9(unk)
    })
    .parse(input)
}

fn parse_unknown10(input: &[u8]) -> IResult<&[u8], Command> {
    map(preceded(tag(UNKNOWN10), (be_u16, be_u16)), |(x, y)| {
        Command::Unknown10(x, y)
    })
    .parse(input)
}

fn parse_command(input: &[u8]) -> IResult<&[u8], Command> {
    alt((
        parse_lineto,
        parse_moveto,
        parse_arc,
        parse_textsize,
        parse_linewidth,
        parse_textrot,
        parse_colour,
        parse_text,
        parse_unknown8,
        parse_unknown9,
        parse_unknown10,
    ))
    .parse(input)
}

pub fn parse_picture_bin(input: &[u8]) -> IResult<&[u8], Option<Picture>> {
    let (input, present) = be_u8(input)?;
    if present != 0 {
        let (input, (x1, y1, x2, y2)) = (be_i16, be_i16, be_i16, be_i16).parse(input)?;
        //println!("Picture {} {} {} {}", x1, y1, x2, y2);
        let (input, commands) = length_count(
            be_u32,
            map(dbg_dmp(parse_command, "command"), |cmd| {
                //println!("{:?}", cmd);
                cmd
            }),
        )
        .parse(input)?;
        Ok((
            input,
            Some(Picture {
                bounds: (x1, y1, x2, y2),
                commands,
            }),
        ))
    } else {
        Ok((input, None))
    }
}
