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

use crate::util::{parse_length_string, parse_nul_term_string, parse_u8_bool};

use std::io::Read;

#[derive(Debug, Default)]
enum CfgAtom1 {
    ArrayU32Ref0(u32),
    Integer1(u32),
    Expr2(
        Box<CfgAtom1>,
        Box<CfgAtom1>,
        (CfgStringIntItem, CfgStringIntItem),
        CfgAtom1ExprOp,
    ),
    #[default]
    None,
}

#[derive(Debug, Default)]
enum CfgAtom1ExprOp {
    OpPlus,
    OpMinus,
    OpMult,
    OpDiv,
    OpMod,
    #[default]
    None,
}

#[derive(Debug, Default)]
enum CfgTopLevelListItem {
    Type0(CfgTopLevelListItemType0),
    Type1(CfgTopLevelListItemType1),
    Type2(CfgTopLevelListItemType2),
    //Type3,
    Type4(CfgTopLevelListItemType4),
    Type5,
    #[default]
    None,
}

#[derive(Debug, Default)]
struct CfgTopLevelListItemType0 {
    a1: CfgAtom1,
    a2: CfgAtom1,
    a3: CfgAtom1,
    str_num_pair: (CfgStringIntItem, CfgStringIntItem),
    s: String,
    l: Vec<CfgTopLevelListItem>,
}

#[derive(Debug, Default)]
struct CfgTopLevelListItemType1 {
    sub: CfgTopLevelListItemType1SubItem,
    l1: Vec<CfgTopLevelListItem>,
    l2: Vec<CfgTopLevelListItem>,
    str_num_pair: (CfgStringIntItem, CfgStringIntItem),
}

#[derive(Debug, Default)]
enum CfgTopLevelListItemType1SubItem {
    Type2(CfgAtom1, CfgAtom1),
    #[default]
    None,
}

#[derive(Debug, Default)]
struct CfgTopLevelListItemType2 {
    str_num_pair: (CfgStringIntItem, CfgStringIntItem),
    content: CfgTopLevelListItemType2Enum,
    t: (CfgAtom1, CfgAtom1, CfgAtom1),
    l: Vec<CfgTopLevelListItem>,
}

#[derive(Debug, Default)]
enum CfgTopLevelListItemType2Enum {
    Type0(u32, u32, Vec<(u32, Vec<CfgTopLevelListItem>)>),
    Type1(u32, Vec<(u32, Vec<CfgTopLevelListItem>)>),
    Type2(String, String, Vec<(String, Vec<CfgTopLevelListItem>)>),
    Type4(u32, u32, u32, Vec<(u32, Vec<CfgTopLevelListItem>)>),
    #[default]
    None,
}

#[derive(Debug, Default)]
enum CfgTopLevelListItemType4 {
    Type0(String),
    Type1(CfgTopLevelListItemType4Type1),
    Type2(CfgTopLevelListItemType4Type2),
    #[default]
    None,
}

#[derive(Debug, Default)]
struct CfgTopLevelListItemType4Type1 {
    atom1: CfgAtom1,
    atom2: CfgAtom1,
    str_num_pair: (CfgStringIntItem, CfgStringIntItem),
}

#[derive(Debug, Default)]
struct CfgTopLevelListItemType4Type2 {
    num: u32,
    string_list: Vec<String>,
    atom_list: Vec<CfgAtom1>,
    cfg_list: Vec<CfgTopLevelListItem>,
    str_num_pair: (CfgStringIntItem, CfgStringIntItem),
}

#[derive(Debug, Default)]
enum CfgStringIntItem {
    StringNum(String, u32),
    NumOnly(u32),
    Neither,
    #[default]
    None,
}

#[derive(Debug, Default)]
pub struct Config {
    family: String,
    device: String,
    pair: (CfgAtom1, Vec<CfgTopLevelListItem>),
}

impl Config {
    pub fn new(mut r: impl Read) -> Option<Config> {
        let mut file = Vec::new();
        r.read_to_end(&mut file).ok()?;
        dbg_dmp(parse_cfg, "cfg")(&file).ok().map(|(_i, d)| {
            //println!("{_i:#?}");
            d
        })
    }
}

fn dbg_rest(input: &[u8]) -> IResult<&[u8], ()> {
    fail::<_, (), _>().parse(input)
}

const MAGIC_PREFIX: &[u8] = &[0x00, 0x00, b'f'];

fn parse_cfg(input: &[u8]) -> IResult<&[u8], Config> {
    let (input, ver) = preceded(tag(MAGIC_PREFIX), be_u8).parse(input)?;
    println!("version: {ver}");
    let (input, (family, device)) = (parse_nul_term_string, parse_nul_term_string).parse(input)?;
    println!("familY: {family}, device: {device}");
    let (input, flag) = parse_u8_bool(input)?;
    println!("Extra Present: {flag:?}");
    // We don't handle the structure that would be here so make sure it fails, this isn't present in any cfgs we have
    let (input, _) = if flag {
        fail::<_, (), _>().parse(input)?
    } else {
        (input, ())
    };

    let (input, pair) = parse_cfg_top_level_pair(input)?;
    //println!("{pair:?}");

    //println!("remaining length: {}", input.len());
    // let (input, _) = if input.len() > 0 {
    //     dbg_dmp(dbg_rest, "remaining").parse(input)?
    // } else {
    //     (input, ())
    // };

    Ok((
        input,
        Config {
            family,
            device,
            pair,
        },
    ))
}

fn parse_variable_length_int(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, n) = be_u8(input)?;
    let (input, val) = match n {
        0x7e => map(be_u16, |n| n as u32).parse(input)?,
        0x7f => be_u32(input)?,
        n => (input, n as u32),
    };

    //println!("VarInt({val})");

    Ok((input, val))
}

fn parse_cfg_atom1_expr2(input: &[u8]) -> IResult<&[u8], CfgAtom1> {
    let (input, (a1, a2, str_num_pair, op)) = (
        parse_cfg_atom1,
        parse_cfg_atom1,
        (parse_cfg_string_int_item, parse_cfg_string_int_item),
        be_u8,
    )
        .parse(input)?;

    //println!("Atom1Pair2 OP: {op}");

    let (input, op) = match op {
        0x0 => (input, CfgAtom1ExprOp::OpPlus),
        0x1 => (input, CfgAtom1ExprOp::OpMinus),
        0x2 => (input, CfgAtom1ExprOp::OpMult),
        0x3 => (input, CfgAtom1ExprOp::OpDiv),
        0x4 => (input, CfgAtom1ExprOp::OpMod),

        _ => map(dbg_dmp(dbg_rest, "Unhandled Op"), |_| CfgAtom1ExprOp::None).parse(input)?,
    };

    let result = CfgAtom1::Expr2(Box::new(a1), Box::new(a2), str_num_pair, op);

    //println!("{result:#?}");

    Ok((input, result))
}

fn parse_cfg_atom1(input: &[u8]) -> IResult<&[u8], CfgAtom1> {
    let (input, typ) = be_u8(input)?;

    //println!("CfgAtom1: t={typ}");
    let (input, a) = match typ {
        0x0 => map(parse_variable_length_int, |n| CfgAtom1::ArrayU32Ref0(n)).parse(input)?,
        0x1 => map(parse_variable_length_int, |n| CfgAtom1::Integer1(n)).parse(input)?,
        0x2 => parse_cfg_atom1_expr2(input)?,
        0x3 => map(dbg_dmp(dbg_rest, "Unhandled case 3"), |_| CfgAtom1::None).parse(input)?,
        _ => map(dbg_dmp(dbg_rest, "Unknown type!"), |_| CfgAtom1::None).parse(input)?,
    };

    //println!("{a:?}");

    Ok((input, a))
}

fn parse_cfg_top_level_pair(input: &[u8]) -> IResult<&[u8], (CfgAtom1, Vec<CfgTopLevelListItem>)> {
    (parse_cfg_atom1, parse_cfg_top_level_list).parse(input)
}

fn parse_cfg_top_level_list(input: &[u8]) -> IResult<&[u8], Vec<CfgTopLevelListItem>> {
    let (input, list) =
        length_count(parse_variable_length_int, parse_cfg_top_level_item).parse(input)?;

    // Useful for finding cases where the list is not a single element.
    // A parser that expects a single element list may be helpful to cut down on
    // Vec's in the data where only a single value appears to exist.
    // if list.len() != 1 {
    //     println!("List that is not length 1: len {}, {:?}", list.len(), list);
    // }

    Ok((input, list))
}

fn parse_cfg_top_level_item(input: &[u8]) -> IResult<&[u8], CfgTopLevelListItem> {
    let (input, typ) = be_u8(input)?;

    //println!("Top Level Item Type: {typ:?}");
    let (input, item) = match typ {
        0x0 => map(parse_cfg_top_level_item_type0, |t| {
            CfgTopLevelListItem::Type0(t)
        })
        .parse(input)?,
        0x1 => map(parse_cfg_top_level_item_type1, |t| {
            CfgTopLevelListItem::Type1(t)
        })
        .parse(input)?,
        0x2 => map(parse_cfg_top_level_item_type2, |t| {
            CfgTopLevelListItem::Type2(t)
        })
        .parse(input)?,
        0x4 => map(parse_cfg_top_level_item_type4, |t| {
            CfgTopLevelListItem::Type4(t)
        })
        .parse(input)?,
        _ => map(dbg_dmp(dbg_rest, "Unknown type!"), |_| {
            CfgTopLevelListItem::None
        })
        .parse(input)?,
    };

    Ok((input, item))
}

fn parse_cfg_top_level_item_type0(input: &[u8]) -> IResult<&[u8], CfgTopLevelListItemType0> {
    let (input, (a1, a2, a3, str_num_pair, s, l)) = (
        parse_cfg_atom1,
        parse_cfg_atom1,
        parse_cfg_atom1,
        (parse_cfg_string_int_item, parse_cfg_string_int_item),
        parse_nul_term_string,
        parse_cfg_top_level_list,
    )
        .parse(input)?;

    let content = CfgTopLevelListItemType0 {
        a1,
        a2,
        a3,
        str_num_pair,
        s,
        l,
    };

    //println!("{content:?}");

    Ok((input, content))
}

fn parse_cfg_top_level_item_type1(input: &[u8]) -> IResult<&[u8], CfgTopLevelListItemType1> {
    let (input, (sub, l1, l2, str_num_pair)) = (
        parse_cfg_top_level_item_type1_sub_item,
        parse_cfg_top_level_list,
        parse_cfg_top_level_list,
        (parse_cfg_string_int_item, parse_cfg_string_int_item),
    )
        .parse(input)?;

    let content = CfgTopLevelListItemType1 {
        sub,
        l1,
        l2,
        str_num_pair,
    };

    //println!("{content:?}");

    Ok((input, content))
}

fn parse_cfg_top_level_item_type1_sub_item(
    input: &[u8],
) -> IResult<&[u8], CfgTopLevelListItemType1SubItem> {
    let (input, typ) = be_u8(input)?;

    //println!("Type1SubItem Type: {typ:?}");
    let (input, content) = match typ {
        0x2 => map((parse_cfg_atom1, parse_cfg_atom1), |(a1, a2)| {
            CfgTopLevelListItemType1SubItem::Type2(a1, a2)
        })
        .parse(input)?,
        _ => map(dbg_dmp(dbg_rest, "Unknown type!"), |_| {
            CfgTopLevelListItemType1SubItem::None
        })
        .parse(input)?,
    };

    //println!("{content:?}");

    Ok((input, content))
}

fn parse_cfg_top_level_item_type2(input: &[u8]) -> IResult<&[u8], CfgTopLevelListItemType2> {
    let (input, (str_num_pair, typ)) = (
        (parse_cfg_string_int_item, parse_cfg_string_int_item),
        be_u8,
    )
        .parse(input)?;

    //println!("Type2 Type: {typ:?}");
    let (input, content) = match typ {
        0x0 => map(
            (
                parse_variable_length_int,
                parse_variable_length_int,
                parse_number_top_level_list_pair_list,
            ),
            |(n1, n2, l)| CfgTopLevelListItemType2Enum::Type0(n1, n2, l),
        )
        .parse(input)?,
        0x1 => {
            map(
                (
                    map(parse_variable_length_int, |n| {
                        /*println!("n: {n:?}");*/
                        n
                    }),
                    parse_number_top_level_list_pair_list,
                ),
                |(n, l)| CfgTopLevelListItemType2Enum::Type1(n, l),
            )
            .parse(input)?
        }
        0x2 => map(
            (
                map(parse_nul_term_string, |s| {
                    println!("s1: {s:?}");
                    s
                }),
                parse_nul_term_string,
                length_count(
                    parse_variable_length_int,
                    (parse_nul_term_string, parse_cfg_top_level_list),
                ),
            ),
            |(s1, s2, l)| CfgTopLevelListItemType2Enum::Type2(s1, s2, l),
        )
        .parse(input)?,
        0x4 => map(
            (
                parse_variable_length_int,
                parse_variable_length_int,
                parse_variable_length_int,
                parse_number_top_level_list_pair_list,
            ),
            |(n1, n2, n3, l)| CfgTopLevelListItemType2Enum::Type4(n1, n2, n3, l),
        )
        .parse(input)?,
        _ => map(dbg_dmp(dbg_rest, "Unknown type!"), |_| {
            CfgTopLevelListItemType2Enum::None
        })
        .parse(input)?,
    };

    let (input, (t, l)) = (
        (parse_cfg_atom1, parse_cfg_atom1, parse_cfg_atom1),
        parse_cfg_top_level_list,
    )
        .parse(input)?;

    let result = CfgTopLevelListItemType2 {
        str_num_pair,
        content,
        t,
        l,
    };

    //println!("{result:?}");

    Ok((input, result))
}

fn parse_cfg_top_level_item_type4(input: &[u8]) -> IResult<&[u8], CfgTopLevelListItemType4> {
    let (input, typ) = be_u8(input)?;

    //println!("Type4 Type: {typ:?}");
    let (input, item) = match typ {
        0x0 => map(parse_nul_term_string, |s| {
            CfgTopLevelListItemType4::Type0(s)
        })
        .parse(input)?,
        0x1 => {
            map(
                (
                    parse_cfg_atom1,
                    parse_cfg_atom1,
                    (parse_cfg_string_int_item, parse_cfg_string_int_item),
                ),
                |(atom1, atom2, str_num_pair)| {
                    let i = CfgTopLevelListItemType4Type1 {
                        atom1,
                        atom2,
                        str_num_pair,
                    };
                    //println!("{i:?}");
                    CfgTopLevelListItemType4::Type1(i)
                },
            )
            .parse(input)?
        }
        0x2 => {
            map(
                (
                    parse_variable_length_int,
                    length_count(parse_variable_length_int, parse_nul_term_string),
                    length_count(parse_variable_length_int, parse_cfg_atom1),
                    parse_cfg_top_level_list,
                    (parse_cfg_string_int_item, parse_cfg_string_int_item),
                ),
                |(num, string_list, atom_list, cfg_list, str_num_pair)| {
                    let i = CfgTopLevelListItemType4Type2 {
                        num,
                        string_list,
                        atom_list,
                        cfg_list,
                        str_num_pair,
                    };
                    //println!("{i:?}");
                    CfgTopLevelListItemType4::Type2(i)
                },
            )
            .parse(input)?
        }
        _ => map(dbg_dmp(dbg_rest, "Unknown type!"), |_| {
            CfgTopLevelListItemType4::None
        })
        .parse(input)?,
    };

    Ok((input, item))
}

fn parse_cfg_string_int_item(input: &[u8]) -> IResult<&[u8], CfgStringIntItem> {
    let (input, typ) = be_u8(input)?;
    //println!("CfgStringIntItem Type: {typ:?}");
    let (input, item) = match typ {
        0x0 => map(
            (parse_nul_term_string, parse_variable_length_int),
            |(s, n)| CfgStringIntItem::StringNum(s, n),
        )
        .parse(input)?,
        0x1 => map(parse_variable_length_int, |n| CfgStringIntItem::NumOnly(n)).parse(input)?,
        0x2 => (input, CfgStringIntItem::Neither),
        _ => map(dbg_dmp(dbg_rest, "Unknown type!"), |_| {
            CfgStringIntItem::None
        })
        .parse(input)?,
    };

    //println!("{item:?}");

    Ok((input, item))
}

fn parse_number_top_level_list_pair_list(
    input: &[u8],
) -> IResult<&[u8], Vec<(u32, Vec<CfgTopLevelListItem>)>> {
    length_count(
        parse_variable_length_int,
        (parse_variable_length_int, parse_cfg_top_level_list),
    )
    .parse(input)
}
