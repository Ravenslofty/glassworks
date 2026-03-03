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
use std::io::Write;
use std::io;

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct FloorPlan {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub data: Vec<u16>,
}

impl FloorPlan {
    pub fn get(&self, x: usize, y: usize, z: usize) -> u16 {
        let index = x * (self.y * self.z) + y * self.z + z;
        
        self.data[index]
    }
}

#[derive(Debug, Default)]
pub struct Cell {
    name: String,
    triple: (u16, u16, u16),
    flag: bool,
    a: u16,
    b: u16,
    patterns: Vec<Pattern>,
    functions: Vec<Function>,
}

#[derive(Debug, Default)]
pub struct Pattern {
    flag: bool,
    a: u16,
    t1: (u16, u16, u16),
    t2: (u16, u16, u16),
    t3: (u16, u16, u16),
}

#[derive(Debug, Default)]
pub struct Function {
    name: String,
    a: u8,
    b: u32,
    flag: bool,
    picture: Option<Picture>,
    bus_refs: Vec<BusRef>,
    c: u32,
    attributes: Vec<Attribute>,
}

#[derive(Debug, Default)]
pub struct BusRef {
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
pub struct Attribute {
    kind: u8,
    val: AttributeEnum,
}

#[derive(Debug, Default)]
pub enum AttributeEnum {
    ThreeDoubles(String, String, String),
    StringList(Vec<String>), // Would have extra u32 with String if ver > 3
    DoubleList(Vec<String>), // Would have extra u32 with Double String if ver > 3
    // Only in ver > 3 U32Pair(u32, u32),
    #[default]
    Empty,
}

#[derive(Debug, Default)]
pub struct Bus {
    name: String,
    a: u8,
    bus_elements: Option<Vec<BusElement>>,
}

#[derive(Debug, Default)]
pub struct BusElement {
    name: String,
    a: u32,
    vec3s: Vec<(u16, u16, u16)>,
    pattern: Pattern,
    picture: Option<Picture>,
    quad_u16: (u16, u16, u16, u16),
    b: u32,
}

#[derive(Debug, Default)]
pub struct Device {
    pub ver: u8,
    family: String,
    device: String,
    picture: Option<Picture>,
    pub floorplan: FloorPlan,
    cells: Vec<Cell>,
    busses: Vec<Option<Bus>>,
    io_pins: Vec<(u16, u16, u16)>,
}

impl Device {
    pub fn new(mut r: impl Read) -> Option<Device> {
        let mut file = Vec::new();
        r.read_to_end(&mut file).ok()?;
        dbg_dmp(parse_device, "device")(&file).ok().map(|(_i, d)| d)
    }

    pub fn dump_floorplan(&self, mut output: impl Write) -> io::Result<()> {
        for i in 0..self.floorplan.z {
            write!(output, "Array {}: \n", i)?;
            for x in 0..self.floorplan.x {
                for y in 0..self.floorplan.y {
                    let val = self.floorplan.get(x, y, i) as usize;
                    let val = val & 0xfff;
                    let name = if val < self.cells.len() { &self.cells[val].name } else { "" };
                    write!(output, "{:16} ", name)?;
                }
                write!(output, "\n")?;
            }
            write!(output, "\n")?;
        }
        let mut map = HashMap::<&[u16], usize>::new();
        let mut types = 0;
        write!(output, "\nTile Types: \n")?;
        for i in 0..self.floorplan.x {
            for j in 0..self.floorplan.y {
                let index = i * self.floorplan.y * self.floorplan.z + j * self.floorplan.z;
                let tile = &self.floorplan.data[index..index+self.floorplan.z];
                let typ = map.entry(tile).or_insert_with(|| {let typ = types; types = types + 1; typ});
                write!(output, "{:03} ", typ)?;
            }
            write!(output, "\n")?;
        }
        Ok(())
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

    let floorplan = FloorPlan { x, y, z, data };

    let (input, cells) = parse_cell_array(input)?;

    let (input, busses) = parse_bus_array(input)?;

    let (input, io_pins) = length_count(be_u32, (be_u16, be_u16, be_u16)).parse(input)?;

    //println!("Cells: {} Buss: {} IoPins: {}", cells.len(), busses.len(), io_pins.len());

    Ok((
        input,
        Device {
            ver,
            family,
            device,
            picture,
            floorplan,
            cells,
            busses,
            io_pins,
        },
    ))
}

fn parse_3_be_u16(input: &[u8]) -> IResult<&[u8], (u16, u16, u16)> {
    (be_u16, be_u16, be_u16).parse(input)
}

fn parse_pattern(input: &[u8]) -> IResult<&[u8], Pattern> {
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
        Pattern {
            flag,
            a,
            t1,
            t2,
            t3,
        },
    ))
}

fn parse_function(input: &[u8]) -> IResult<&[u8], Function> {
    let (input, (name, a, b, flag, picture, bus_refs, c, attributes)) = (
        parse_length_string,
        be_u8,
        be_u32,
        parse_u8_bool,
        parse_picture_bin,
        dbg_dmp(parse_bus_ref_array, "bus_refs"),
        be_u32,
        //This is only present if ver is >2
        length_count(be_u32, dbg_dmp(parse_attribute, "attribute")),
    )
        .parse(input)?;
    Ok((
        input,
        Function {
            name,
            a,
            b,
            flag,
            picture,
            bus_refs,
            c,
            attributes,
        },
    ))
}

fn parse_bus_ref_array(
    input: &[u8],
) -> IResult<&[u8], Vec<BusRef>> {
    let (input, elems) = map((be_u32, be_u32), |(x, _)| x as usize).parse(input)?;
    count(parse_bus_ref, elems).parse(input)
}

fn parse_bus_ref(input: &[u8]) -> IResult<&[u8], BusRef> {
    let (input, (f1, f2, f3, f4, a, b, picture, c)) = (
        parse_u8_bool,
        parse_u8_bool,
        parse_u8_bool,
        parse_u8_bool,
        be_u16,
        be_u32,
        parse_picture_bin,
        // Only present before version 5
        be_u32,
    )
        .parse(input)?;

    Ok((
        input,
        BusRef {
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

fn parse_attribute(input: &[u8]) -> IResult<&[u8], Attribute> {
    let (input, (kind, typ)) = (be_u8, be_u8).parse(input)?;
    //println!("attribute type: {typ}");
    let (input, val) = match typ {
        0 => map(
            (
                parse_length_string,
                parse_length_string,
                parse_length_string,
            ),
            |(x, y, z)| AttributeEnum::ThreeDoubles(x, y, z),
        )
        .parse(input)?,
        1 => map(length_count(be_u32, parse_length_string), |v| {
            AttributeEnum::StringList(v)
        })
        .parse(input)?, // Would include a u32 in the list content if ver > 3
        2 => map(length_count(be_u32, parse_length_string), |v| {
            AttributeEnum::DoubleList(v)
        })
        .parse(input)?, // Would include a u32 in the list content if ver > 3
        // 3 => Would have two u32s if ver > 3
        _ => fail::<_, AttributeEnum, _>().parse(input)?,
    };

    Ok((input, Attribute { kind, val }))
}

pub fn parse_cell(input: &[u8]) -> IResult<&[u8], Cell> {
    let (input, (name, triple)) = (parse_length_string, parse_3_be_u16).parse(input)?;
    // This is only present if ver is > 1
    let (input, flag) = be_u8(input)?;
    //println!("Cell name: {}", name);
    let (input, (a, b)) = (be_u16, be_u16).parse(input)?;
    let (input, patterns) = length_count(be_u32, parse_pattern).parse(input)?;
    let (input, functions_present) = be_u8(input)?;
    let (input, functions) = if functions_present != 0 {
        let (input, elems) = map((be_u32, be_u32), |(elems, _elem_size)| elems).parse(input)?;
        count(dbg_dmp(parse_function, "function"), elems as usize).parse(input)?
    } else {
        (input, vec![])
    };
    let result = Cell {
        name,
        triple,
        flag: flag == 0x1,
        a,
        b,
        patterns,
        functions,
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

fn parse_bus(input: &[u8]) -> IResult<&[u8], Option<Bus>> {
    let (input, present) = parse_u8_bool(input)?;
    if present {
        let (input, (name, a, bus_elems_present)) =
            (parse_length_string, be_u8, parse_u8_bool).parse(input)?;
        //println!("Elment2: {}", name);
        let (input, bus_elements) = if bus_elems_present {
            map(parse_bus_element_array, |v| Some(v)).parse(input)?
        } else {
            (input, None)
        };

        Ok((
            input,
            Some(Bus {
                name,
                a,
                bus_elements,
            }),
        ))
    } else {
        Ok((input, None))
    }
}

fn parse_bus_array(input: &[u8]) -> IResult<&[u8], Vec<Option<Bus>>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, busses) = count(dbg_dmp(parse_bus, "bus"), length).parse(input)?;
    Ok((input, busses))
}

fn parse_bus_element(input: &[u8]) -> IResult<&[u8], BusElement> {
    let (input, (name, a)) = (parse_length_string, be_u32).parse(input)?;
    let (input, vec3s) = parse_bus_element_vec3_array(input)?;
    let (input, (pattern, picture, quad_u16, b)) = (
        parse_pattern,
        parse_picture_bin,
        (be_u16, be_u16, be_u16, be_u16),
        be_u32,
    )
        .parse(input)?;

    Ok((
        input,
        BusElement {
            name,
            a,
            vec3s,
            pattern,
            picture,
            quad_u16,
            b,
        },
    ))
}

fn parse_bus_element_array(input: &[u8]) -> IResult<&[u8], Vec<BusElement>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, bus_elements) = count(
        dbg_dmp(parse_bus_element, "bus_element"),
        length,
    )
    .parse(input)?;
    Ok((input, bus_elements))
}

fn parse_bus_element_vec3_array(input: &[u8]) -> IResult<&[u8], Vec<(u16, u16, u16)>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, vec3s) = count(
        dbg_dmp(parse_3_be_u16, "vec3"),
        length,
    )
    .parse(input)?;
    Ok((input, vec3s))
}
