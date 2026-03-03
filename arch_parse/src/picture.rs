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
    elements: Vec<Element>,
}

#[derive(Debug)]
pub enum Element {
    LineTo(i16, i16),
    MoveTo(i16, i16),
    Arc(u16, u16, u16, u16),
    TextSize(u32),
    LineWidth(u32),
    TextRot(u32),
    Colour(u16),
    Text(String),
    FillType(u32),
    LineType(u32),
    PolyPoint(u16, u16),
    PolyClose,
}

const LINETO: &[u8] = &[0_u8];
const MOVETO: &[u8] = &[1_u8];
const ARC: &[u8] = &[2_u8];
const TEXTSIZE: &[u8] = &[3_u8];
const LINEWIDTH: &[u8] = &[4_u8];
const TEXTROT: &[u8] = &[5_u8];
const COLOUR: &[u8] = &[6_u8];
const TEXT: &[u8] = &[7_u8];
const FILLTYPE: &[u8] = &[8_u8];
const LINETYPE: &[u8] = &[9_u8];
const POLYPOINT: &[u8] = &[10_u8];
const POLYCLOSE: &[u8] = &[11_u8];

fn parse_lineto(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(LINETO), (be_i16, be_i16)), |(x, y)| {
        Element::LineTo(x, y)
    })
    .parse(input)
}

fn parse_moveto(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(MOVETO), (be_i16, be_i16)), |(x, y)| {
        Element::MoveTo(x, y)
    })
    .parse(input)
}

fn parse_arc(input: &[u8]) -> IResult<&[u8], Element> {
    map(
        preceded(tag(ARC), (be_u16, be_u16, be_u16, be_u16)),
        |(x1, y1, x2, y2)| Element::Arc(x1, y1, x2, y2),
    )
    .parse(input)
}

fn parse_textsize(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(TEXTSIZE), be_u32), |size| {
        Element::TextSize(size)
    })
    .parse(input)
}

fn parse_linewidth(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(LINEWIDTH), be_u32), |width| {
        Element::LineWidth(width)
    })
    .parse(input)
}

fn parse_textrot(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(TEXTROT), be_u32), |rot| Element::TextSize(rot)).parse(input)
}

fn parse_colour(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(COLOUR), be_u16), |colour| {
        Element::Colour(colour)
    })
    .parse(input)
}

fn parse_text(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(TEXT), parse_length_string), |text| {
        Element::Text(text)
    })
    .parse(input)
}

fn parse_fill_type(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(FILLTYPE), be_u32), |unk| {
        Element::FillType(unk)
    })
    .parse(input)
}

fn parse_line_type(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(LINETYPE), be_u32), |unk| {
        Element::LineType(unk)
    })
    .parse(input)
}

fn parse_poly_point(input: &[u8]) -> IResult<&[u8], Element> {
    map(preceded(tag(POLYPOINT), (be_u16, be_u16)), |(x, y)| {
        Element::PolyPoint(x, y)
    })
    .parse(input)
}

fn parse_poly_close(input: &[u8]) -> IResult<&[u8], Element> {
    map(tag(LINETYPE), |_| {
        Element::PolyClose
    })
    .parse(input)
}

fn parse_element(input: &[u8]) -> IResult<&[u8], Element> {
    alt((
        parse_lineto,
        parse_moveto,
        parse_arc,
        parse_textsize,
        parse_linewidth,
        parse_textrot,
        parse_colour,
        parse_text,
        parse_fill_type,
        parse_line_type,
        parse_poly_point,
        parse_poly_close,
    ))
    .parse(input)
}

pub fn parse_picture_bin(input: &[u8]) -> IResult<&[u8], Option<Picture>> {
    let (input, present) = be_u8(input)?;
    if present != 0 {
        let (input, (x1, y1, x2, y2)) = (be_i16, be_i16, be_i16, be_i16).parse(input)?;
        //println!("Picture {} {} {} {}", x1, y1, x2, y2);
        let (input, elements) = length_count(
            be_u32,
            map(dbg_dmp(parse_element, "element"), |elem| {
                //println!("{:?}", elem);
                elem
            }),
        )
        .parse(input)?;
        Ok((
            input,
            Some(Picture {
                bounds: (x1, y1, x2, y2),
                elements,
            }),
        ))
    } else {
        Ok((input, None))
    }
}
