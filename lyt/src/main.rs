// SPDX-License-Identifier: ISC

use std::assert_matches;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{self, Read};

use lexpr::Value;

#[derive(Debug, Default)]
struct Element {
    x: i64,
    y: i64,
    z: i64,
    properties: i64,
    config: i64,
    net_idx: i64,
    net: Option<String>,
    port: Option<(String, i64)>,
}

#[derive(Debug, Default)]
struct Layout {
    key_transform: (i64, i64, i64, i64, i64, i64),
    shape_transform: (i64, i64, i64, i64, i64, i64),
    elements: Vec<Element>,
}

impl Layout {
    pub fn new(r: impl Read) -> Option<Self> {
        let parse_number = |v: &Value| -> Option<i64> {
            if v.is_number() {
                v.as_number()?.as_i64()
            } else if v.is_symbol() {
                let s = v.as_symbol()?;
                let radix = if s.starts_with("%x") {
                    16
                } else if s.starts_with("%b") {
                    2
                } else {
                    panic!("don't know radix of symbol '{s}'");
                };
                i64::from_str_radix(&s[2..], radix).ok()
            } else {
                panic!("don't know how to parse '{v:?}' as number");
            }
        };

        let mut this = Self::default();

        let layout = lexpr::from_reader(r).unwrap();
        let layout = layout.to_vec()?;
        assert_eq!(layout[0].as_symbol(), Some("layout"));

        for item in layout.iter().skip(1) {
            assert!(item[0].is_symbol(), "expected symbol, found {item:?}");
            let symbol = item[0].as_symbol()?.to_string();
            match symbol.as_str() {
                "status" => {
                    assert_matches!(item[1].as_number()?.as_u64()?, 1 | /* DFR */ 2);
                }
                "key" => {
                    assert_eq!(item[1].as_str().unwrap(), "mpa1000");
                    assert_matches!(item[2].as_str().unwrap(), "mpa1000" | "mpa1036");
                    assert!(item[3].is_cons());
                    let t = item[3].to_vec()?;
                    assert_matches!(t[0].as_symbol().unwrap(), "t" | "transform");
                    this.key_transform.0 = parse_number(&t[1])?;
                    this.key_transform.1 = parse_number(&t[2])?;
                    this.key_transform.2 = parse_number(&t[3])?;
                    this.key_transform.3 = parse_number(&t[4])?;
                    this.key_transform.4 = parse_number(&t[5])?;
                    this.key_transform.5 = parse_number(&t[6])?;
                    /*assert_matches!(this.key_transform, (1, 0, 0, 1, _, _));
                    assert_matches!(
                        (this.key_transform.4, this.key_transform.5),
                        /* DFR */
                        (4, 5) | /* AND */ (10, 10) | /* DF */ (10, 11) | /* DFER */ (12, 17) | /* OR */ (16, 16) | /* DFE */ (16, 17) | /* XOR */ (17, 17)
                    );*/
                }
                "n" => {
                    /*
                    assert_eq!(item[1].as_number()?.as_i64()?, 0);
                    // item[2] appears to be net name.
                    assert_eq!(item[3].as_number()?.as_i64()?, 2);
                    if item[4] == Value::Nil {
                        continue;
                    }
                    let usestatus = item[4].to_vec()?;
                    assert_eq!(usestatus[0].as_symbol()?, "usestatus");
                    assert_eq!(usestatus[1].as_number()?.as_i64()?, 0);
                    if item[5] == Value::Nil {
                        continue;
                    }
                    let r#use = item[5].to_vec()?;
                    assert_eq!(r#use[0].as_symbol()?, "use");
                    assert_eq!(r#use[1].as_str()?, "clk");
                    assert_eq!(r#use[2].as_str()?, "(null)");
                    */
                }
                "p" => {
                    // TODO
                }
                "shape" => {
                    this.shape_transform.0 = parse_number(&item[1])?;
                    this.shape_transform.1 = parse_number(&item[2])?;
                    this.shape_transform.2 = parse_number(&item[3])?;
                    this.shape_transform.3 = parse_number(&item[4])?;
                    this.shape_transform.4 = parse_number(&item[5])?;
                    this.shape_transform.5 = parse_number(&item[6])?;
                    let item = item.to_vec()?;
                    for item in item.iter().skip(7) {
                        let symbol = item[0].as_symbol()?.to_string();
                        match symbol.as_str() {
                            "element" => {
                                let mut element = Element::default();
                                element.x = parse_number(&item[1])?;
                                element.y = parse_number(&item[2])?;
                                element.z = parse_number(&item[3])?;
                                element.properties = parse_number(&item[4])?;
                                element.config = parse_number(&item[5])?;
                                element.net_idx = parse_number(&item[6])?;
                                let net = item[7].to_vec()?;
                                assert_eq!(net[0].as_symbol()?, "net");
                                element.net = Some(net[1].as_str()?.to_string());
                                let port = item[8].to_vec()?;
                                assert_eq!(port[0].as_symbol()?, "port");
                                let port_name = port[1].as_str()?.to_string();
                                let port_index = port[2].as_number()?.as_i64()?;
                                element.port = Some((port_name, port_index));
                                this.elements.push(element);
                            }
                            "e" => {
                                let mut element = Element::default();
                                element.x = parse_number(&item[1])?;
                                element.y = parse_number(&item[2])?;
                                element.z = parse_number(&item[3])?;
                                element.properties = parse_number(&item[4])?;
                                element.config = parse_number(&item[5])?;
                                element.net_idx = parse_number(&item[6])?;
                                this.elements.push(element);
                            }
                            unknown_keyword => {
                                unimplemented!("keyword '{unknown_keyword}' inside layout::shape")
                            }
                        }
                    }
                }
                unknown_keyword => unimplemented!("keyword '{unknown_keyword}' inside layout"),
            }
        }

        Some(this)
    }

    fn disassemble_core_cell(&self, subelements: &[(i64, i64, i64)]) {
        let mut should_break = false;
        let mut should_break_2 = true;
        for &(z, properties, config) in subelements {
            match z {
                0 => {
                    assert!(config <= 1023, "A input mux has more than 10 config bits");
                    assert!(
                        properties <= 1,
                        "A input mux: {properties} has more than 2 properties"
                    );
                    const INPUT_INVERTED: i64 = 0b00_0000_0001;
                    /// Never set?
                    const UNKNOWN_BIT1: i64 = 0b00_0000_0010;
                    const HORIZONTAL_MEDIUM_BUS: i64 = 0b10_0000_0100;
                    const VERTICAL_MEDIUM_BUS: i64 = 0b10_0000_1000;
                    const LL_LOCAL_INTERCONNECT: i64 = 0b10_0001_0000;
                    const U_LOCAL_INTERCONNECT: i64 = 0b10_0010_0000;
                    const FBB_LOCAL_INTERCONNECT: i64 = 0b10_0100_0000;
                    const FB_LOCAL_INTERCONNECT: i64 = 0b10_1000_0000;
                    const F_LOCAL_INTERCONNECT: i64 = 0b11_0000_0000;
                    const MUX_ENABLE: i64 = 0b10_0000_0000;

                    println!("    Z=0 - A input mux:");
                    if config & INPUT_INVERTED == INPUT_INVERTED {
                        println!("        -- ---- ---1: input inverted");
                    }
                    if config & UNKNOWN_BIT1 == UNKNOWN_BIT1 {
                        println!("        -- ---- --1-: UNKNOWN bit 1 = 1");
                        should_break = true;
                    }
                    if config & HORIZONTAL_MEDIUM_BUS == HORIZONTAL_MEDIUM_BUS {
                        println!("        1- ---- -1--: input = horizontal medium bus");
                    }
                    if config & VERTICAL_MEDIUM_BUS == VERTICAL_MEDIUM_BUS {
                        println!("        1- ---- 1---: input = vertical medium bus");
                    }
                    if config & LL_LOCAL_INTERCONNECT == LL_LOCAL_INTERCONNECT {
                        println!("        1- ---1 ----: input = LL local interconnect");
                    }
                    if config & U_LOCAL_INTERCONNECT == U_LOCAL_INTERCONNECT {
                        println!("        1- --1- ----: input = U local interconnect");
                    }
                    if config & FBB_LOCAL_INTERCONNECT == FBB_LOCAL_INTERCONNECT {
                        println!("        1- -1-- ----: input = FBB local interconnect");
                    }
                    if config & FB_LOCAL_INTERCONNECT == FB_LOCAL_INTERCONNECT {
                        println!("        1- 1--- ----: input = FB local interconnect");
                    }
                    if config & F_LOCAL_INTERCONNECT == F_LOCAL_INTERCONNECT {
                        println!("        11 ---- ----: input = F local interconnect");
                    }
                    if config & MUX_ENABLE == 0 {
                        println!("        0- ---- ----: input = constant");
                    }
                    match properties {
                        0 => println!("        properties=0: noninverted"),
                        1 => println!("        properties=1: inverted"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                1 => {
                    assert!(
                        config <= 1023,
                        "B input mux: {config} has more than 10 config bits"
                    );
                    assert!(
                        properties <= 1,
                        "B input mux: {properties} has more than 2 properties"
                    );
                    const INPUT_INVERTED: i64 = 0b00_0000_0001;
                    /// Never set?
                    const UNKNOWN_BIT1: i64 = 0b00_0000_0010;
                    const HORIZONTAL_MEDIUM_BUS: i64 = 0b10_0000_0100;
                    const VERTICAL_MEDIUM_BUS: i64 = 0b10_0000_1000;
                    const UU_LOCAL_INTERCONNECT: i64 = 0b10_0001_0000;
                    const L_LOCAL_INTERCONNECT: i64 = 0b10_0010_0000;
                    const FF_LOCAL_INTERCONNECT: i64 = 0b10_0100_0000;
                    const FB_LOCAL_INTERCONNECT: i64 = 0b10_1000_0000;
                    const F_LOCAL_INTERCONNECT: i64 = 0b11_0000_0000;
                    const MUX_ENABLE: i64 = 0b10_0000_0000;

                    println!("    Z=1 - B input mux:");
                    if config & INPUT_INVERTED == INPUT_INVERTED {
                        println!("        -- ---- ---1: input inverted");
                    }
                    if config & UNKNOWN_BIT1 == UNKNOWN_BIT1 {
                        println!("        -- ---- --1-: UNKNOWN bit 1 = 1");
                        should_break = true;
                    }
                    if config & HORIZONTAL_MEDIUM_BUS == HORIZONTAL_MEDIUM_BUS {
                        println!("        1- ---- -1--: input = horizontal medium bus");
                    }
                    if config & VERTICAL_MEDIUM_BUS == VERTICAL_MEDIUM_BUS {
                        println!("        1- ---- 1---: input = vertical medium bus");
                    }
                    if config & UU_LOCAL_INTERCONNECT == UU_LOCAL_INTERCONNECT {
                        println!("        1- ---1 ----: input = UU local interconnect");
                    }
                    if config & L_LOCAL_INTERCONNECT == L_LOCAL_INTERCONNECT {
                        println!("        1- --1- ----: input = L local interconnect");
                    }
                    if config & FF_LOCAL_INTERCONNECT == FF_LOCAL_INTERCONNECT {
                        println!("        1- -1-- ----: input = FF local interconnect");
                    }
                    if config & FB_LOCAL_INTERCONNECT == FB_LOCAL_INTERCONNECT {
                        println!("        1- 1--- ----: input = FB local interconnect");
                    }
                    if config & F_LOCAL_INTERCONNECT == F_LOCAL_INTERCONNECT {
                        println!("        11 ---- ----: input = F local interconnect");
                    }
                    if config & MUX_ENABLE == 0 {
                        println!("        0- ---- ----: input = constant");
                    }
                    match properties {
                        0 => println!("        properties=0: noninverted"),
                        1 => println!("        properties=1: inverted"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                2 => {
                    assert!(config <= 31, "2: {config} has more than 5 config bits");
                    assert!(
                        properties <= 5,
                        "2: {properties} has more than 5 properties"
                    );
                    const RESET: i64 = 0b0_0001;
                    const CLOCK: i64 = 0b0_0010;
                    // Never set?
                    const UNKNOWN_BIT2: i64 = 0b0_0100;
                    const UNKNOWN_BIT3: i64 = 0b0_1000;
                    const UNKNOWN_BIT4: i64 = 0b1_0000;

                    println!("    Z=2 - ??:");

                    if config & RESET == RESET {
                        println!("        - ---1: connect to reset");
                    }
                    if config & CLOCK == CLOCK {
                        println!("        - --1-: connect to clock");
                    }
                    if config & UNKNOWN_BIT2 == 0 {
                        println!("        - -0--: UNKNOWN bit 2 = 0");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT3 == 0 {
                        println!("        - 0---: UNKNOWN bit 3 = 0");
                        //should_break = true;
                    }
                    if config & UNKNOWN_BIT4 == 0 {
                        println!("        0 ----: df/ndf/dfr/dl?");
                    }

                    match properties {
                        0 => println!("        properties=0: and"),
                        1 => println!("        properties=1: buffer"),
                        2 => println!("        properties=2: pull-up"),
                        3 => println!("        properties=3: pull-down"),
                        4 => println!("        properties=4: xor/dff"),
                        5 => println!("        properties=5: dlatch"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                3 => {
                    assert!(
                        config <= 524287,
                        "medium bus: {config} has more than 19 config bits"
                    );
                    assert!(
                        properties <= 0,
                        "medium bus: {properties} has more than 0 properties"
                    );
                    /// This only ever appears on buffers.
                    const UNKNOWN_BIT0: i64 = 0b000_0000_0000_0000_0001;
                    const LOCAL_INTERCONNECT_CONNECTED_TO_MEDIUM_BUS: i64 =
                        0b000_0000_0000_0000_0010;
                    /// Never set?
                    const UNKNOWN_BIT2: i64 = 0b000_0000_0000_0000_0100;
                    /// Never set?
                    const UNKNOWN_BIT3: i64 = 0b000_0000_0000_0000_1000;
                    /// Never set?
                    const UNKNOWN_BIT4: i64 = 0b000_0000_0000_0001_0000;
                    /// Never set?
                    const UNKNOWN_BIT5: i64 = 0b000_0000_0000_0010_0000;
                    const UU_A_LOCAL_INTERCONNECT: i64 = 0b000_0000_0000_0100_0000;
                    const LL_B_LOCAL_INTERCONNECT: i64 = 0b000_0000_0000_1000_0000;
                    const U_B_LOCAL_INTERCONNECT: i64 = 0b000_0000_0001_0000_0000;
                    const L_A_LOCAL_INTERCONNECT: i64 = 0b000_0000_0010_0000_0000;
                    const FBB_A_LOCAL_INTERCONNECT: i64 = 0b000_0000_0100_0000_0000;
                    /// Never set?
                    const UNKNOWN_BIT11: i64 = 0b000_0000_1000_0000_0000;
                    const FF_B_LOCAL_INTERCONNECT: i64 = 0b000_0001_0000_0000_0000;
                    const FB_A_LOCAL_INTERCONNECT: i64 = 0b000_0010_0000_0000_0000;
                    const FB_B_LOCAL_INTERCONNECT: i64 = 0b000_0100_0000_0000_0000;
                    const F_A_LOCAL_INTERCONNECT: i64 = 0b000_1000_0000_0000_0000;
                    const F_B_LOCAL_INTERCONNECT: i64 = 0b001_0000_0000_0000_0000;
                    /// Never set?
                    const UNKNOWN_BIT17: i64 = 0b010_0000_0000_0000_0000;
                    /// This only ever appears on buffers.
                    const UNKNOWN_BIT18: i64 = 0b100_0000_0000_0000_0000;

                    println!("    Z=3 - medium bus??");
                    if config & UNKNOWN_BIT0 == UNKNOWN_BIT0 {
                        println!("        --- ---- ---- ---- ---1: UNKNOWN bit 0 = 1 (buffer?)");
                    }
                    if config & LOCAL_INTERCONNECT_CONNECTED_TO_MEDIUM_BUS
                        == LOCAL_INTERCONNECT_CONNECTED_TO_MEDIUM_BUS
                    {
                        println!(
                            "        --- ---- ---- ---- --1-: connect medium bus to local interconnect"
                        );
                    }
                    if config & UNKNOWN_BIT2 == UNKNOWN_BIT2 {
                        println!("        --- ---- ---- ---- -1--: UNKNOWN bit 2 = 1");
                    }
                    if config & UNKNOWN_BIT3 == UNKNOWN_BIT3 {
                        println!("        --- ---- ---- ---- 1---: UNKNOWN bit 3 = 1");
                    }
                    if config & UNKNOWN_BIT4 == UNKNOWN_BIT4 {
                        println!("        --- ---- ---- ---1 ----: UNKNOWN bit 4 = 1");
                    }
                    if config & UNKNOWN_BIT5 == UNKNOWN_BIT5 {
                        println!("        --- ---- ---- --1- ----: UNKNOWN bit 5 = 1");
                    }
                    if config & UU_A_LOCAL_INTERCONNECT == UU_A_LOCAL_INTERCONNECT {
                        println!("        --- ---- ---- -1-- ----: UU.A local interconnect");
                    }
                    if config & LL_B_LOCAL_INTERCONNECT == LL_B_LOCAL_INTERCONNECT {
                        println!("        --- ---- ---- 1--- ----: LL.B local interconnect");
                    }
                    if config & U_B_LOCAL_INTERCONNECT == U_B_LOCAL_INTERCONNECT {
                        println!("        --- ---- ---1 ---- ----: U.B local interconnect");
                    }
                    if config & L_A_LOCAL_INTERCONNECT == L_A_LOCAL_INTERCONNECT {
                        println!("        --- ---- --1- ---- ----: L.A local interconnect");
                    }
                    if config & FBB_A_LOCAL_INTERCONNECT == FBB_A_LOCAL_INTERCONNECT {
                        println!("        --- ---- -1-- ---- ----: FBB.A local interconnect");
                    }
                    if config & UNKNOWN_BIT11 == UNKNOWN_BIT11 {
                        println!("        --- ---- 1--- ---- ----: UNKNOWN bit 11 = 1");
                        should_break = true;
                    }
                    if config & FF_B_LOCAL_INTERCONNECT == FF_B_LOCAL_INTERCONNECT {
                        println!("        --- ---1 ---- ---- ----: FF.B local interconnect");
                    }
                    if config & FB_A_LOCAL_INTERCONNECT == FB_A_LOCAL_INTERCONNECT {
                        println!("        --- --1- ---- ---- ----: FB.A local interconnect");
                    }
                    if config & FB_B_LOCAL_INTERCONNECT == FB_B_LOCAL_INTERCONNECT {
                        println!("        --- -1-- ---- ---- ----: FB.B local interconnect");
                    }
                    if config & F_A_LOCAL_INTERCONNECT == F_A_LOCAL_INTERCONNECT {
                        println!("        --- 1--- ---- ---- ----: F.A local interconnect");
                    }
                    if config & F_B_LOCAL_INTERCONNECT == F_B_LOCAL_INTERCONNECT {
                        println!("        --1 ---- ---- ---- ----: F.B local interconnect");
                    }
                    if config & UNKNOWN_BIT17 == UNKNOWN_BIT17 {
                        println!("        -1- ---- ---- ---- ----: UNKNOWN bit 17 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT18 == UNKNOWN_BIT18 {
                        println!("        1-- ---- ---- ---- ----: UNKNOWN bit 18 = 1 (buffer?)");
                    }
                }
                4 => {
                    assert!(config <= 1023, "4: {config} has more than 10 config bits");
                    assert!(
                        properties <= 0,
                        "4: {properties} has more than 0 properties"
                    );
                    println!("    Z=4 - ??: {config:010b}");
                    //should_break = true;
                }
                5 => {
                    assert!(config <= 1023, "5: {config} has more than 10 config bits");
                    assert!(
                        properties <= 0,
                        "5: {properties} has more than 0 properties"
                    );
                    println!("    Z=5 - ??: {config:010b}");
                    //should_break = true;
                }
                6 => {
                    assert!(config <= 3, "X Bus: {config} has more than 2 config bits");
                    assert!(
                        properties <= 1,
                        "X Bus: {properties} has more than 2 properties"
                    );
                    println!("    Z=6 - X Bus:");
                    const LEFT_VERTICAL_X_BUS: i64 = 0b01;
                    const BELOW_HORIZONTAL_X_BUS: i64 = 0b10;
                    if config & LEFT_VERTICAL_X_BUS == LEFT_VERTICAL_X_BUS {
                        println!("        -1: connect to left vertical X bus");
                    }
                    if config & BELOW_HORIZONTAL_X_BUS == BELOW_HORIZONTAL_X_BUS {
                        println!("        1-: connect to below horizontal X bus");
                    }
                    match properties {
                        0 => println!("        properties=0: ?"),
                        1 => println!("        properties=1: ?"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                7 => {
                    assert!(
                        config <= 524287,
                        "function: {config} has more than 19 config bits"
                    );
                    assert!(
                        properties <= 0,
                        "function: {properties} has more than 0 properties"
                    );
                    const UNKNOWN_BIT0: i64 = 0b000_0000_0000_0000_0001;
                    // Never set?
                    const UNKNOWN_BIT1: i64 = 0b000_0000_0000_0000_0010;
                    const BELOW_HORIZONTAL_MEDIUM_BUS: i64 = 0b000_0000_0000_0000_0100;
                    const LEFT_VERTICAL_MEDIUM_BUS: i64 = 0b000_0000_0000_0000_1000;
                    const ABOVE_HORIZONTAL_MEDIUM_BUS: i64 = 0b000_0000_0000_0001_0000;
                    const RIGHT_VERTICAL_MEDIUM_BUS: i64 = 0b000_0000_0000_0010_0000;
                    // Never set?
                    const UNKNOWN_BIT6: i64 = 0b000_0000_0000_0100_0000;
                    // Never set?
                    const UNKNOWN_BIT7: i64 = 0b000_0000_0000_1000_0000;
                    // Never set?
                    const UNKNOWN_BIT8: i64 = 0b000_0000_0001_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT9: i64 = 0b000_0000_0010_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT10: i64 = 0b000_0000_0100_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT11: i64 = 0b000_0000_1000_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT12: i64 = 0b000_0001_0000_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT13: i64 = 0b000_0010_0000_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT14: i64 = 0b000_0100_0000_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT15: i64 = 0b000_1000_0000_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT16: i64 = 0b001_0000_0000_0000_0000;
                    // Never set?
                    const UNKNOWN_BIT17: i64 = 0b010_0000_0000_0000_0000;
                    const UNKNOWN_BIT18: i64 = 0b100_0000_0000_0000_0000;

                    println!("    Z=7 - function??:");
                    if config & UNKNOWN_BIT0 == UNKNOWN_BIT0 {
                        println!("        --- ---- ---- ---- ---1: UNKNOWN bit 0 = 1");
                    }
                    if config & UNKNOWN_BIT1 == UNKNOWN_BIT1 {
                        println!("        --- ---- ---- ---- --1-: UNKNOWN bit 1 = 1");
                        should_break = true;
                    }
                    if config & BELOW_HORIZONTAL_MEDIUM_BUS == BELOW_HORIZONTAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- ---- -1--: connect to below horizontal medium bus"
                        );
                    }
                    if config & LEFT_VERTICAL_MEDIUM_BUS == LEFT_VERTICAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- ---- 1---: connect to left vertical medium bus"
                        );
                    }
                    if config & ABOVE_HORIZONTAL_MEDIUM_BUS == ABOVE_HORIZONTAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- ---1 ----: connect to above horizontal medium bus"
                        );
                    }
                    if config & RIGHT_VERTICAL_MEDIUM_BUS == RIGHT_VERTICAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- --1- ----: connect to right vertical medium bus"
                        );
                    }
                    if config & UNKNOWN_BIT6 == UNKNOWN_BIT6 {
                        println!("        --- ---- ---- -1-- ----: UNKNOWN bit 6 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT7 == UNKNOWN_BIT7 {
                        println!("        --- ---- ---- 1--- ----: UNKNOWN bit 7 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT8 == UNKNOWN_BIT8 {
                        println!("        --- ---- ---1 ---- ----: UNKNOWN bit 8 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT9 == UNKNOWN_BIT9 {
                        println!("        --- ---- --1- ---- ----: UNKNOWN bit 9 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT10 == UNKNOWN_BIT10 {
                        println!("        --- ---- -1-- ---- ----: UNKNOWN bit 10 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT11 == UNKNOWN_BIT11 {
                        println!("        --- ---- 1--- ---- ----: UNKNOWN bit 11 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT12 == UNKNOWN_BIT12 {
                        println!("        --- ---1 ---- ---- ----: UNKNOWN bit 12 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT13 == UNKNOWN_BIT15 {
                        println!("        --- --1- ---- ---- ----: UNKNOWN bit 13 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT14 == UNKNOWN_BIT14 {
                        println!("        --- -1-- ---- ---- ----: UNKNOWN bit 14 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT15 == UNKNOWN_BIT15 {
                        println!("        --- 1--- ---- ---- ----: UNKNOWN bit 15 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT16 == UNKNOWN_BIT16 {
                        println!("        --1 ---- ---- ---- ----: UNKNOWN bit 16 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT17 == UNKNOWN_BIT17 {
                        println!("        -1- ---- ---- ---- ----: UNKNOWN bit 17 = 1");
                        should_break = true;
                    }
                    if config & UNKNOWN_BIT18 == UNKNOWN_BIT18 {
                        println!("        1-- ---- ---- ---- ----: UNKNOWN bit 18 = 1");
                        //should_break = true;
                    }
                }
                8 => {
                    assert!(config <= 1023, "8: {config} has more than 10 config bits");
                    assert!(
                        properties <= 0,
                        "8: {properties} has more than 0 properties"
                    );
                    println!("    Z=8 - ??: {config:010b}");
                    should_break = true;
                }
                _ => panic!(
                    "    don't know how to disassemble core cell z = {z} with config {config:b}"
                ),
            }
        }
        if should_break && should_break_2 {
            panic!("at the disco");
        }
    }

    fn disassemble_io_pad(&self, subelements: &[(i64, i64, i64)]) {
        for &(z, properties, config) in subelements {
            match z {
                3 => {
                    assert!(config <= 0, "3: {config} has more than 0 config bits");
                    assert!(
                        properties <= 0,
                        "3: {properties} has more than 0 properties"
                    );
                    println!("    Z=3 - ??");
                }
                _ => panic!(
                    "    don't know how to disassemble i/o pad z = {z} with config {config:b}"
                ),
            }
        }
    }

    fn disassemble_peripheral_bus(&self, subelements: &[(i64, i64, i64)]) {
        for &(z, properties, config) in subelements {
            match z {
                4 => {
                    assert!(config <= 3, "4: {config} has more than 2 config bits");
                    assert!(
                        properties <= 3,
                        "4: {properties} has more than 3 properties"
                    );
                    const UNKNOWN_BIT0: i64 = 0b01;
                    const UNKNOWN_BIT1: i64 = 0b10;
                    println!("    Z=4 - inverter?:");
                    if config & UNKNOWN_BIT0 == UNKNOWN_BIT0 {
                        println!("        -1: UNKNOWN bit 0 = 1");
                    }
                    if config & UNKNOWN_BIT1 == UNKNOWN_BIT1 {
                        println!("        1-: UNKNOWN bit 1 = 1");
                    }
                    match properties {
                        0 => println!("        properties=0: noninverted"),
                        1 => println!("        properties=1: pull-down"),
                        2 => println!("        properties=2: inverted"),
                        3 => println!("        properties=3: pull-up"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                6 => {
                    assert!(config <= 511, "6: {config} has more than 9 config bits");
                    assert!(
                        properties <= 0,
                        "6: {properties} has more than 0 properties"
                    );
                    const GLOBAL_BUS_0: i64 = 0b1_0000_0001;
                    const GLOBAL_BUS_1: i64 = 0b1_0000_0010;
                    const GLOBAL_BUS_2: i64 = 0b1_0000_0100;
                    const GLOBAL_BUS_3: i64 = 0b1_0000_1000;
                    const MEDIUM_BUS_0: i64 = 0b1_0001_0000;
                    const MEDIUM_BUS_1: i64 = 0b1_0010_0000;
                    const X_BUS: i64 = 0b0_0100_0000;
                    const UNKNOWN_BIT7: i64 = 0b0_1000_0000;
                    const MUX_ENABLE: i64 = 0b1_0000_0000;
                    println!("    Z=6 - Pad/P-Bus Input?:");
                    if config & GLOBAL_BUS_0 == GLOBAL_BUS_0 {
                        println!("        1 ---- ---1: input = global bus 0");
                    }
                    if config & GLOBAL_BUS_1 == GLOBAL_BUS_1 {
                        println!("        1 ---- --1-: input = global bus 1");
                    }
                    if config & GLOBAL_BUS_2 == GLOBAL_BUS_2 {
                        println!("        1 ---- -1--: input = global bus 2");
                    }
                    if config & GLOBAL_BUS_3 == GLOBAL_BUS_3 {
                        println!("        1 ---- 1---: input = global bus 3");
                    }
                    if config & MEDIUM_BUS_0 == MEDIUM_BUS_0 {
                        println!("        1 ---1 ----: input = medium bus 0");
                    }
                    if config & MEDIUM_BUS_1 == MEDIUM_BUS_1 {
                        println!("        1 --1- ----: input = medium bus 1");
                    }
                    if config & X_BUS == X_BUS {
                        println!("        1 -1-- ----: input = X bus");
                    }
                    if config & UNKNOWN_BIT7 == UNKNOWN_BIT7 {
                        println!("        - 1--- ----: UNKNOWN bit 7 = 1");
                    }
                    if config & MUX_ENABLE == 0 {
                        println!("        0 ---- ----: input = disabled?");
                    }
                }
                7 => {
                    assert!(config <= 511, "8: {config} has more than 9 config bits");
                    assert!(
                        properties <= 0,
                        "7: {properties} has more than 0 properties"
                    );
                    println!("    Z=7 - ??: {config:09b}");
                }
                8 => {
                    assert!(config <= 2047, "Pad/P-Bus Output: {config} has more than 11 config bits");
                    assert!(
                        properties <= 0,
                        "Pad/P-Bus Output: {properties} has more than 0 properties"
                    );
                    const GLOBAL_BUS_0: i64 = 0b000_0000_0001;
                    const GLOBAL_BUS_1: i64 = 0b000_0000_0010;
                    const GLOBAL_BUS_2: i64 = 0b000_0000_0100;
                    const GLOBAL_BUS_3: i64 = 0b000_0000_1000;
                    const MEDIUM_BUS_0: i64 = 0b000_0001_0000;
                    const MEDIUM_BUS_1: i64 = 0b000_0010_0000;
                    const X_BUS: i64 = 0b000_0100_0000;
                    const UNKNOWN_BIT7: i64 = 0b000_1000_0000;
                    const UNKNOWN_BIT8: i64 = 0b001_0000_0000;
                    const UNKNOWN_BIT9: i64 = 0b010_0000_0000;
                    const UNKNOWN_BIT10: i64 = 0b100_0000_0000;
                    println!("    Z=8 - Pad/P-Bus Output?:");
                    if config & GLOBAL_BUS_0 == GLOBAL_BUS_0 {
                        println!("        --- ---- ---1: connect to global bus 0");
                    }
                    if config & GLOBAL_BUS_1 == GLOBAL_BUS_1 {
                        println!("        --- ---- --1-: connect to global bus 1");
                    }
                    if config & GLOBAL_BUS_2 == GLOBAL_BUS_2 {
                        println!("        --- ---- -1--: connect to global bus 2");
                    }
                    if config & GLOBAL_BUS_3 == GLOBAL_BUS_3 {
                        println!("        --- ---- 1---: connect to global bus 3");
                    }
                    if config & MEDIUM_BUS_0 == MEDIUM_BUS_0 {
                        println!("        --- ---1 ----: connect to medium bus 0");
                    }
                    if config & MEDIUM_BUS_1 == MEDIUM_BUS_1 {
                        println!("        --- --1- ----: connect to medium bus 1");
                    }
                    if config & X_BUS == X_BUS {
                        println!("        --- -1-- ----: connect to X bus");
                    }
                    if config & UNKNOWN_BIT7 == UNKNOWN_BIT7 {
                        println!("        --- 1--- ----: UNKNOWN bit 7 = 1");
                    }
                    if config & UNKNOWN_BIT8 == UNKNOWN_BIT8 {
                        println!("        --1 ---- ----: UNKNOWN bit 8 = 1");
                    }
                    if config & UNKNOWN_BIT9 == UNKNOWN_BIT9 {
                        println!("        -1- ---- ----: UNKNOWN bit 9 = 1");
                    }
                    if config & UNKNOWN_BIT10 == UNKNOWN_BIT10 {
                        println!("        1-- ---- ----: UNKNOWN bit 10 = 1");
                    }
                }
                _ => panic!(
                    "    don't know how to disassemble peripheral bus z = {z} with config {config:b} and properties {properties}"
                ),
            }
        }
    }

    pub fn disassemble(&self) {
        let mut elements = BTreeMap::<(i64, i64), Vec<(i64, i64, i64)>>::new();

        for element in &self.elements {
            let entry = elements.entry((element.x, element.y)).or_insert(vec![]);
            entry.push((element.z, element.properties, element.config));
        }

        for ((x, y), subelements) in elements {
            let standalone_core_cell = matches!(self.shape_transform, (0, 0, 0, 0, 0, 8)) && x == 0 && y == 0;
            let within_core_cell = ((4_i64..14).contains(&x)
                || (16_i64..26).contains(&x)
                || (28_i64..38).contains(&x)
                || (40_i64..50).contains(&x)
                || (52_i64..62).contains(&x)
                || (64_i64..74).contains(&x))
                && ((4_i64..14).contains(&y)
                    || (16_i64..26).contains(&y)
                    || (28_i64..38).contains(&y)
                    || (40_i64..50).contains(&y)
                    || (52_i64..62).contains(&y)
                    || (64_i64..74).contains(&y));
            let within_io_pad = (0..1).contains(&y);
            let within_peripheral_bus = (2..3).contains(&y);
            if standalone || within_core_cell {
                println!("({x}, {y}) = Core Cell:");
                self.disassemble_core_cell(&subelements);
            } else if within_io_pad {
                println!("({x}, {y}) = I/O Pad:");
                self.disassemble_io_pad(&subelements);
            } else if within_peripheral_bus {
                println!("({x}, {y}) = Peripheral Bus?:");
                self.disassemble_peripheral_bus(&subelements);
            } else {
                println!("({x}, {y}) = ???");
            }
        }
    }
}

fn main() -> io::Result<()> {
    for filename in std::env::args().skip(1) {
        let file = File::open(&filename)?;
        println!("{filename}:");

        let layout = Layout::new(file).unwrap();
        layout.disassemble();
    }

    Ok(())
}
