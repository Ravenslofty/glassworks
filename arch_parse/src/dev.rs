// SPDX-License-Identifier: ISC

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    combinator::{cond, fail, map},
    error::{context, dbg_dmp},
    multi::{count, length_count},
    number::complete::{be_i16, be_u8, be_u16, be_u32},
    sequence::preceded,
};

use crate::picture::{Picture, parse_picture_bin};

use crate::util::{parse_3_be_i16, parse_3_be_u16, parse_length_string, parse_u8_bool};

use std::io;
use std::io::Read;
use std::io::Write;

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct FloorPlan {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub data: Vec<u16>,
}

impl FloorPlan {
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<u16> {
        if x >= self.x || y >= self.y || z >= self.z {
            None
        } else {
            let index = x * (self.y * self.z) + y * self.z + z;

            Some(self.data[index])
        }
    }
}

#[derive(Debug, Default)]
pub struct Cell {
    name: String,
    triple: (u16, u16, u16),
    global: bool,
    a: u16,
    b: u16,
    patterns: Vec<Pattern>,
    functions: Vec<Function>,
}

#[derive(Debug, Default)]
pub struct PatternRange {
    start: u16,
    stop: u16,
    step: u16,
}

impl PatternRange {
    pub fn from_tuple((start, stop, step): (u16, u16, u16)) -> PatternRange {
        PatternRange { start, stop, step }
    }
}

#[derive(Debug, Default)]
pub struct Pattern {
    flag: bool,
    a: u16,
    x: PatternRange,
    y: PatternRange,
    z: PatternRange,
}

#[derive(Debug, Default)]
pub struct Function {
    name: String,
    a: u8,
    b: u32,
    flag: bool,
    picture: Option<Picture>,
    bus_refs: Vec<BusRef>,
    c: Option<u32>,
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
    c: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Attribute {
    kind: u8,
    val: AttributeEnum,
}

#[derive(Debug, Default)]
pub enum AttributeEnum {
    AttrPattern(String, String, String),
    AttrEnum(Vec<Enumerator>), // Would have extra u32 with String if ver > 3
    AttrNumericEnum(Vec<NumericEnumerator>), // Would have extra u32 with Double String if ver > 3
    // Only in ver > 3 U32Pair(u32, u32),
    #[default]
    Empty,
}

#[derive(Debug, Default)]
pub struct Enumerator {
    name: String,
    a: Option<u32>,
}

#[derive(Debug, Default)]
pub struct NumericEnumerator {
    dval: String,
    a: Option<u32>,
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
    vec3s: Vec<(i16, i16, i16)>,
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

    fn get_cell_at_loc(&self, x: usize, y: usize, z: usize) -> Option<&Cell> {
        self.floorplan.get(x, y, z).map(|t| self.get_cell(t))?
    }

    fn get_cell(&self, cell_id: u16) -> Option<&Cell> {
        let cell_type = cell_id as usize;
        let cell_type = cell_type & 0xfff;
        self.cells.get(cell_type)
    }

    fn get_bus(&self, bus_index: u16) -> Option<&Bus> {
        let bus_index = bus_index as usize;
        self.busses.get(bus_index)?.as_ref()
    }

    pub fn dump_floorplan(&self, mut output: impl Write) -> io::Result<()> {
        for i in 0..self.floorplan.z {
            write!(output, "Array {}: \n", i)?;
            for x in 0..self.floorplan.x {
                for y in 0..self.floorplan.y {
                    let name = self.get_cell_at_loc(x, y, i).map_or("", |c| &c.name);
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
                let tile = &self.floorplan.data[index..index + self.floorplan.z];
                let typ = map.entry(tile).or_insert_with(|| {
                    let typ = types;
                    types = types + 1;
                    typ
                });
                write!(output, "{:03} ", typ)?;
            }
            write!(output, "\n")?;
        }
        Ok(())
    }

    pub fn dump_location(
        &self,
        mut output: impl Write,
        x: usize,
        y: usize,
        z: Option<usize>,
    ) -> io::Result<()> {
        if x >= self.floorplan.x || y >= self.floorplan.y {
            write!(output, "Location is outside of the grid")?;
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Location is outside of the grid",
            ))
        } else {
            match z {
                Some(z) => {
                    println!(
                        "{} {} {}",
                        self.floorplan.x, self.floorplan.y, self.floorplan.z
                    );
                    if z >= self.floorplan.z {
                        write!(output, "z co-ord is out of range")?;
                        Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "z co-ord is out of range",
                        ))
                    } else {
                        write!(
                            output,
                            "x={} y={} z={}:\n{}",
                            x,
                            y,
                            z,
                            self.get_cell_at_loc(x, y, z).map_or("", |c| &c.name)
                        )?;
                        Ok(())
                    }
                }
                None => {
                    write!(output, "x={} y={}:\n", x, y)?;
                    for z in 0..self.floorplan.z {
                        let cell_name = self.get_cell_at_loc(x, y, z).map_or("", |c| &c.name);
                        write!(output, "  {}: {}\n", z, cell_name)?;
                    }
                    Ok(())
                }
            }
        }
    }

    pub fn dump_cells(&self, mut output: impl Write) -> io::Result<()> {
        for cell in &self.cells {
            self.dump_cell(cell, &mut output)?;
        }
        Ok(())
    }

    pub fn dump_cell(&self, cell: &Cell, mut output: impl Write) -> io::Result<()> {
        let margin = 4;
        let margin_str = (0..margin).map(|_| " ").collect::<String>();
        write!(
            output,
            "Cell: {} x={} y={} z={}\n",
            cell.name, cell.triple.0, cell.triple.1, cell.triple.2
        )?;
        write!(output, "{}global: {}\n", margin_str, cell.global)?;
        write!(output, "{}a={} b={}\n", margin_str, cell.a, cell.b)?;
        write!(output, "{}functions:\n", margin_str)?;
        for func in &cell.functions {
            self.dump_function(margin + 4, func, cell.a, &mut output)?;
        }
        Ok(())
    }

    pub fn dump_function(
        &self,
        margin: usize,
        func: &Function,
        cell_a: u16,
        mut output: impl Write,
    ) -> io::Result<()> {
        let margin_str = (0..margin).map(|_| " ").collect::<String>();
        write!(output, "{}Func: {}\n", margin_str, func.name)?;
        write!(
            output,
            "{}  a={} b={} flag={} c={:?}\n",
            margin_str, func.a, func.b, func.flag, func.c
        )?;
        if cell_a as usize != func.bus_refs.len() {
            write!(
                output,
                "{}  BusRefs Length does not match Cell\n",
                margin_str
            )?;
        }
        for busref in &func.bus_refs {
            self.dump_bus_ref(margin + 4, busref, &mut output)?;
        }
        Ok(())
    }

    pub fn dump_bus_ref(
        &self,
        margin: usize,
        bus_ref: &BusRef,
        mut output: impl Write,
    ) -> io::Result<()> {
        let margin_str = (0..margin).map(|_| " ").collect::<String>();
        let bus = self.get_bus(bus_ref.a);
        let bus_name = bus.map_or("<unknown>", |b| &b.name);
        write!(output, "{}BusRef: target={}\n", margin_str, bus_name)?;
        write!(
            output,
            "{}  a={} b={:#x} c={:?}\n",
            margin_str, bus_ref.a, bus_ref.b, bus_ref.c
        )?;
        write!(
            output,
            "{}  {} {} {} {}\n",
            margin_str,
            if bus_ref.f1 { "inv" } else { "   " },
            if bus_ref.f2 { "in" } else { "  " },
            if bus_ref.f3 { "out" } else { "   " },
            if bus_ref.f4 { "f4" } else { "  " },
        )?;

        Ok(())
    }
}

const MAGIC_PREFIX: &[u8] = &[0x00, 0x00, b'U'];

pub fn parse_device(input: &[u8]) -> IResult<&[u8], Device> {
    let (input, ver) = preceded(tag(MAGIC_PREFIX), be_u8).parse(input)?;
    if ver > 5 {
        context("Version too high", fail::<_, &[u8], _>()).parse(input)?;
    }
    // 2.3 has version 3 files, 2.4.4 has version 5
    match ver {
        3 | 5 => parse_device_inner(ver, input),
        _ => context("Version mismatch", fail::<_, Device, _>()).parse(input),
    }
}

fn parse_device_inner(ver: u8, input: &[u8]) -> IResult<&[u8], super::Device> {
    let (input, (family, device)) = (parse_length_string, parse_length_string).parse(input)?;
    //println!("family: {:?} device: {:?}", family, device);
    let (input, picture) = dbg_dmp(parse_picture_bin, "picture")(input)?;

    let (input, floorplan) = parse_floorplan(input)?;

    let (input, cells) = parse_cell_array(ver, input)?;

    let (input, busses) = parse_bus_array(input)?;

    let (input, io_pins) = length_count(be_u32, parse_3_be_u16).parse(input)?;

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

fn parse_floorplan(input: &[u8]) -> IResult<&[u8], FloorPlan> {
    let (input, (x, y, z, _)) = (be_u32, be_u32, be_u32, be_u32).parse(input)?;
    let (x, y, z) = (x as usize, y as usize, z as usize);
    let (input, data) = count(be_u16, x * y * z).parse(input)?;

    Ok((input, FloorPlan { x, y, z, data }))
}

fn parse_cell_array(ver: u8, input: &[u8]) -> IResult<&[u8], Vec<Cell>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, cells) = count(dbg_dmp(|i| parse_cell(ver, i), "cell"), length).parse(input)?;
    Ok((input, cells))
}

pub fn parse_cell(ver: u8, input: &[u8]) -> IResult<&[u8], Cell> {
    let (input, (name, triple)) = (parse_length_string, parse_3_be_u16).parse(input)?;
    // This is only present if ver is > 1
    let (input, global) = parse_u8_bool(input)?;
    //println!("Cell name: {}", name);
    let (input, (a, b)) = (be_u16, be_u16).parse(input)?;
    let (input, patterns) = length_count(be_u32, parse_pattern).parse(input)?;
    let (input, functions_present) = be_u8(input)?;
    let (input, functions) = if functions_present != 0 {
        let (input, elems) = map((be_u32, be_u32), |(elems, _elem_size)| elems).parse(input)?;
        count(
            dbg_dmp(|i| parse_function(ver, i), "function"),
            elems as usize,
        )
        .parse(input)?
    } else {
        (input, vec![])
    };
    let result = Cell {
        name,
        triple,
        global,
        a,
        b,
        patterns,
        functions,
    };
    //println!("{:#?}", result);
    Ok((input, result))
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
            x: PatternRange::from_tuple(t1),
            y: PatternRange::from_tuple(t2),
            z: PatternRange::from_tuple(t3),
        },
    ))
}

fn parse_function(ver: u8, input: &[u8]) -> IResult<&[u8], Function> {
    let (input, (name, a, b, flag, picture, bus_refs, c, attributes)) = (
        parse_length_string,
        be_u8,
        be_u32,
        parse_u8_bool,
        parse_picture_bin,
        dbg_dmp(|i| parse_bus_ref_array(ver, i), "bus_refs"),
        // only present before version 5
        cond(ver < 5, be_u32),
        length_count(be_u32, dbg_dmp(|i| parse_attribute(ver, i), "attribute")),
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

fn parse_bus_ref_array(ver: u8, input: &[u8]) -> IResult<&[u8], Vec<BusRef>> {
    let (input, elems) = map((be_u32, be_u32), |(x, _)| x as usize).parse(input)?;
    count(|i| parse_bus_ref(ver, i), elems).parse(input)
}

fn parse_bus_ref(ver: u8, input: &[u8]) -> IResult<&[u8], BusRef> {
    let (input, (f1, f2, f3, f4, a, b, picture, c)) = (
        parse_u8_bool,
        parse_u8_bool,
        parse_u8_bool,
        parse_u8_bool,
        be_u16,
        be_u32,
        parse_picture_bin,
        // Only present before version 5
        cond(ver < 5, be_u32),
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

fn parse_attribute(ver: u8, input: &[u8]) -> IResult<&[u8], Attribute> {
    let (input, (kind, typ)) = (be_u8, be_u8).parse(input)?;
    //println!("attribute type: {typ}");
    let (input, val) = match typ {
        0 => map(
            (
                parse_length_string,
                parse_length_string,
                parse_length_string,
            ),
            |(x, y, z)| AttributeEnum::AttrPattern(x, y, z),
        )
        .parse(input)?,
        1 => map(
            length_count(be_u32, |i| parse_attr_enumerator(ver, i)),
            |v| AttributeEnum::AttrEnum(v),
        )
        .parse(input)?,
        2 => map(
            length_count(be_u32, |i| parse_attr_numeric_enumerator(ver, i)),
            |v| AttributeEnum::AttrNumericEnum(v),
        )
        .parse(input)?, // Would include a u32 in the list content if ver > 3
        // 3 => Would have two u32s if ver > 3
        _ => fail::<_, AttributeEnum, _>().parse(input)?,
    };

    Ok((input, Attribute { kind, val }))
}

fn parse_attr_enumerator(ver: u8, input: &[u8]) -> IResult<&[u8], Enumerator> {
    let (input, (name, a)) = (parse_length_string, cond(ver > 3, be_u32)).parse(input)?;

    Ok((input, Enumerator { name, a }))
}

fn parse_attr_numeric_enumerator(ver: u8, input: &[u8]) -> IResult<&[u8], NumericEnumerator> {
    let (input, (dval, a)) = (parse_length_string, cond(ver > 3, be_u32)).parse(input)?;

    Ok((input, NumericEnumerator { dval, a }))
}

fn parse_bus_array(input: &[u8]) -> IResult<&[u8], Vec<Option<Bus>>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, busses) = count(dbg_dmp(parse_bus, "bus"), length).parse(input)?;
    Ok((input, busses))
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

fn parse_bus_element_array(input: &[u8]) -> IResult<&[u8], Vec<BusElement>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, bus_elements) =
        count(dbg_dmp(parse_bus_element, "bus_element"), length).parse(input)?;
    Ok((input, bus_elements))
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

fn parse_bus_element_vec3_array(input: &[u8]) -> IResult<&[u8], Vec<(i16, i16, i16)>> {
    let (input, (length, _elem_size)) = (be_u32, be_u32).parse(input)?;
    let length = length as usize;
    let (input, vec3s) = count(dbg_dmp(parse_3_be_i16, "vec3"), length).parse(input)?;
    Ok((input, vec3s))
}
