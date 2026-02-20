// SPDX-License-Identifier: ISC

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    combinator::{fail, map},
    error::{context, dbg_dmp},
    multi::{count, length_count},
    number::complete::{be_u8, be_u16, be_u32},
    sequence::preceded,
};

use crate::picture::{Picture, parse_picture_bin};

use crate::util::{parse_length_string, parse_u8_bool};

use std::io::Read;

#[derive(Debug, Default)]
pub struct MysteryArray3 {
    x: usize,
    y: usize,
    z: usize,
    data: Vec<u16>,
}

#[derive(Debug, Default)]
pub struct Cell {
    name: String,
    triple: (u16, u16, u16),
    flag: bool,
    a: u16,
    b: u16,
    sub_elements: Vec<SubElement1>,
    sub_element2s: Vec<SubElement2>,
}

#[derive(Debug, Default)]
pub struct SubElement1 {
    flag: bool,
    a: u16,
    t1: (u16, u16, u16),
    t2: (u16, u16, u16),
    t3: (u16, u16, u16),
}

#[derive(Debug, Default)]
pub struct SubElement2 {
    name: String,
    a: u8,
    b: u32,
    flag: bool,
    picture: Option<Picture>,
    sub_element1s: Vec<SubElement2SubElement1>,
    c: u32,
    sub_element2s: Vec<SubElement2SubElement2>,
}

#[derive(Debug, Default)]
pub struct SubElement2SubElement1 {
    f1: bool,
    f2: bool,
    f3: bool,
    f4: bool,
    a: u16,
    b: u32,
    picture: Option<Picture>,
    c: u32,
}

#[derive(Debug, Default)]
pub struct SubElement2SubElement2 {
    a: u8,
    val: SubElement2SubElement2Enum,
}

#[derive(Debug, Default)]
pub enum SubElement2SubElement2Enum {
    ThreeDoubles(String, String, String),
    StringList(Vec<String>), // Would have extra u32 with String if ver > 3
    DoubleList(Vec<String>), // Would have extra u32 with Double String if ver > 3
    // Only in ver > 3 U32Pair(u32, u32),
    #[default]
    Empty,
}

#[derive(Debug, Default)]
pub struct Element2 {
    name: String,
    a: u8,
    sub_elements: Option<Vec<Element2SubElement>>,
}

#[derive(Debug, Default)]
pub struct Element2SubElement {
    name: String,
    a: u32,
    sub_elements: Vec<(u16, u16, u16)>,
    weird: WeirdThing,
    picture: Option<Picture>,
    quad_u16: (u16, u16, u16, u16),
    b: u32,
}

#[derive(Debug, Default)]
pub struct WeirdThing {
    flag: bool,
    a: u16,
    t1: (u16, u16, u16),
    t2: (u16, u16, u16),
    t3: (u16, u16, u16),
}

#[derive(Debug, Default)]
pub struct Device {
    pub ver: u8,
    family: String,
    device: String,
    picture: Option<Picture>,
    mystery_array3: MysteryArray3,
    cells: Vec<Cell>,
    element2s: Vec<Option<Element2>>,
    io_pins: Vec<(u16, u16, u16)>,
}

impl Device {
    pub fn new(mut r: impl Read) -> Option<Device> {
        let mut file = Vec::new();
        r.read_to_end(&mut file).ok()?;
        dbg_dmp(parse_device, "device")(&file).ok().map(|(_i, d)| d)
    }
}

const MAGIC_PREFIX: &[u8] = &[0x00, 0x00, b'U'];

pub fn parse_device(input: &[u8]) -> IResult<&[u8], Device> {
    let (input, ver) = preceded(tag(MAGIC_PREFIX), be_u8).parse(input)?;
    if ver > 5 {
        context("Version too high", fail::<_, &[u8], _>()).parse(input)?;
    }
    // Only version 3 files are seen in current known software so exclude other cases
    if ver != 3 {
        context("Version mismatch", fail::<_, &[u8], _>()).parse(input)?;
    }
    let (input, (family, device)) = (parse_length_string, parse_length_string).parse(input)?;
    //println!("family: {:?} device: {:?}", family, device);
    let (input, picture) = dbg_dmp(parse_picture_bin, "picture")(input)?;
    let (input, (x, y, z, _)) = (be_u32, be_u32, be_u32, be_u32).parse(input)?;
    let (x, y, z) = (x as usize, y as usize, z as usize);
    let (input, data) = count(be_u16, x * y * z).parse(input)?;

    let mystery_array3 = MysteryArray3 { x, y, z, data };

    let (input, cells) = parse_cell_array(input)?;

    let (input, element2s) = parse_element2_array(input)?;

    let (input, io_pins) = length_count(be_u32, (be_u16, be_u16, be_u16)).parse(input)?;

    Ok((
        input,
        Device {
            ver,
            family,
            device,
            picture,
            mystery_array3,
            cells,
            element2s,
            io_pins,
        },
    ))
}

fn parse_3_be_u16(input: &[u8]) -> IResult<&[u8], (u16, u16, u16)> {
    (be_u16, be_u16, be_u16).parse(input)
}

fn parse_sub_element1(input: &[u8]) -> IResult<&[u8], SubElement1> {
    let (input, (flag, a, t1, t2, t3)) = (
        parse_u8_bool,
        be_u16,
        parse_3_be_u16,
        parse_3_be_u16,
        parse_3_be_u16,
    )
        .parse(input)?;
    Ok((
        input,
        SubElement1 {
            flag,
            a,
            t1,
            t2,
            t3,
        },
    ))
}

fn parse_sub_element2(input: &[u8]) -> IResult<&[u8], SubElement2> {
    let (input, (name, a, b, flag, picture, sub_element1s, c, sub_element2s)) = (
        parse_length_string,
        be_u8,
        be_u32,
        parse_u8_bool,
        parse_picture_bin,
        parse_sub_element2_sub_element1_array,
        be_u32,
        //This is only present if ver is <2
        length_count(be_u32, parse_sub_element2_sub_element2),
    )
        .parse(input)?;
    Ok((
        input,
        SubElement2 {
            name,
            a,
            b,
            flag,
            picture,
            sub_element1s,
            c,
            sub_element2s,
        },
    ))
}

fn parse_sub_element2_sub_element1_array(
    input: &[u8],
) -> IResult<&[u8], Vec<SubElement2SubElement1>> {
    let (input, elems) = map((be_u32, be_u32), |(x, _)| x as usize).parse(input)?;
    count(parse_sub_element2_sub_element1, elems).parse(input)
}

fn parse_sub_element2_sub_element1(input: &[u8]) -> IResult<&[u8], SubElement2SubElement1> {
    let (input, (f1, f2, f3, f4, a, b, picture, c)) = (
        parse_u8_bool,
        parse_u8_bool,
        parse_u8_bool,
        parse_u8_bool,
        be_u16,
        be_u32,
        parse_picture_bin,
        be_u32,
    )
        .parse(input)?;

    Ok((
        input,
        SubElement2SubElement1 {
            f1,
            f2,
            f3,
            f4,
            a,
            b,
            picture,
            c,
        },
    ))
}

fn parse_sub_element2_sub_element2(input: &[u8]) -> IResult<&[u8], SubElement2SubElement2> {
    let (input, (a, typ)) = (be_u8, be_u8).parse(input)?;
    let (input, val) = match typ {
        0 => map(
            (
                parse_length_string,
                parse_length_string,
                parse_length_string,
            ),
            |(x, y, z)| SubElement2SubElement2Enum::ThreeDoubles(x, y, z),
        )
        .parse(input)?,
        1 => map(length_count(be_u32, parse_length_string), |v| {
            SubElement2SubElement2Enum::StringList(v)
        })
        .parse(input)?, // Would include a u32 in the list content if ver > 3
        2 => map(length_count(be_u32, parse_length_string), |v| {
            SubElement2SubElement2Enum::DoubleList(v)
        })
        .parse(input)?, // Would include a u32 in the list content if ver > 3
        // 3 => Would have two u32s if ver > 3
        _ => fail::<_, SubElement2SubElement2Enum, _>().parse(input)?,
    };

    Ok((input, SubElement2SubElement2 { a, val }))
}

pub fn parse_cell(input: &[u8]) -> IResult<&[u8], Cell> {
    let (input, (name, triple)) = (parse_length_string, parse_3_be_u16).parse(input)?;
    // This is only present if ver is > 1
    let (input, flag) = be_u8(input)?;
    //println!("Cell name: {}", name);
    let (input, (a, b)) = (be_u16, be_u16).parse(input)?;
    let (input, sub_elements) = length_count(be_u32, parse_sub_element1).parse(input)?;
    let (input, sub_element2s_present) = be_u8(input)?;
    let (input, sub_element2s) = if sub_element2s_present != 0 {
        let (input, elems) = map((be_u32, be_u32), |(elems, _elem_size)| elems).parse(input)?;
        count(parse_sub_element2, elems as usize).parse(input)?
    } else {
        (input, vec![])
    };
    let result = Cell {
        name,
        triple,
        flag: flag == 0x1,
        a,
        b,
        sub_elements,
        sub_element2s,
    };
    //println!("{:#?}", result);
    Ok((input, result))
}

fn parse_cell_array(input: &[u8]) -> IResult<&[u8], Vec<Cell>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, cells) = count(dbg_dmp(parse_cell, "cell"), length).parse(input)?;
    Ok((input, cells))
}

fn parse_element2(input: &[u8]) -> IResult<&[u8], Option<Element2>> {
    let (input, present) = parse_u8_bool(input)?;
    if present {
        let (input, (name, a, sub_elems_present)) =
            (parse_length_string, be_u8, parse_u8_bool).parse(input)?;
        //println!("Elment2: {}", name);
        let (input, sub_elements) = if sub_elems_present {
            map(parse_element2_sub_element_array, |v| Some(v)).parse(input)?
        } else {
            (input, None)
        };

        Ok((
            input,
            Some(Element2 {
                name,
                a,
                sub_elements,
            }),
        ))
    } else {
        Ok((input, None))
    }
}

fn parse_element2_array(input: &[u8]) -> IResult<&[u8], Vec<Option<Element2>>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, element2s) = count(dbg_dmp(parse_element2, "element2"), length).parse(input)?;
    Ok((input, element2s))
}

fn parse_element2_sub_element(input: &[u8]) -> IResult<&[u8], Element2SubElement> {
    let (input, (name, a)) = (parse_length_string, be_u32).parse(input)?;
    let (input, sub_elements) = parse_element2_sub_sub_element_array(input)?;
    let (input, (weird, picture, quad_u16, b)) = (
        parse_weird,
        parse_picture_bin,
        (be_u16, be_u16, be_u16, be_u16),
        be_u32,
    )
        .parse(input)?;

    Ok((
        input,
        Element2SubElement {
            name,
            a,
            sub_elements,
            weird,
            picture,
            quad_u16,
            b,
        },
    ))
}

fn parse_weird(input: &[u8]) -> IResult<&[u8], WeirdThing> {
    let (input, (flag, a, t1, t2, t3)) = (
        parse_u8_bool,
        be_u16,
        (be_u16, be_u16, be_u16),
        (be_u16, be_u16, be_u16),
        (be_u16, be_u16, be_u16),
    )
        .parse(input)?;
    Ok((
        input,
        WeirdThing {
            flag,
            a,
            t1,
            t2,
            t3,
        },
    ))
}

fn parse_element2_sub_element_array(input: &[u8]) -> IResult<&[u8], Vec<Element2SubElement>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, element2sub_elems) = count(
        dbg_dmp(parse_element2_sub_element, "element2sub_element"),
        length,
    )
    .parse(input)?;
    Ok((input, element2sub_elems))
}

fn parse_element2_sub_sub_element_array(input: &[u8]) -> IResult<&[u8], Vec<(u16, u16, u16)>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, element2sub_elems) = count(
        dbg_dmp(parse_element2_sub_sub_element, "element2sub_sub_element"),
        length,
    )
    .parse(input)?;
    Ok((input, element2sub_elems))
}

fn parse_element2_sub_sub_element(input: &[u8]) -> IResult<&[u8], (u16, u16, u16)> {
    let (input, t) = (be_u16, be_u16, be_u16).parse(input)?;
    Ok((input, t))
}
