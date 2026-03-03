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

    fn disassemble_core_cell(&self, kind: i8, subelements: &[(i64, i64, i64)]) {
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
                    const INPUT_NORMAL: i64 = 0b00_0000_0001;
                    const HORIZONTAL_MEDIUM_BUS: i64 = 0b10_0000_0100;
                    const VERTICAL_MEDIUM_BUS: i64 = 0b10_0000_1000;
                    const LL_LOCAL_INTERCONNECT: i64 = 0b10_0001_0000;
                    const U_LOCAL_INTERCONNECT: i64 = 0b10_0010_0000;
                    const FBB_LOCAL_INTERCONNECT: i64 = 0b10_0100_0000;
                    const FB_LOCAL_INTERCONNECT: i64 = 0b10_1000_0000;
                    const F_LOCAL_INTERCONNECT: i64 = 0b11_0000_0000;

                    println!("    Z=0 - \"imux_2\" A input mux:");
                    // Output is in order of config bits in the bitstream
                    if config & INPUT_NORMAL == INPUT_NORMAL {
                        println!("        ------- 1: input polarity normal");
                    } else {
                        println!("        ------- 0: input polarity inverted");
                    }
                    match config & !1 {
                        FBB_LOCAL_INTERCONNECT => println!("        ------1 -: input = FBB local interconnect"),
                        FB_LOCAL_INTERCONNECT  => println!("        -----1- -: input = FB local interconnect"),
                        LL_LOCAL_INTERCONNECT  => println!("        ----1-- -: input = LL local interconnect"),
                        F_LOCAL_INTERCONNECT   => println!("        ---1--- -: input = F  local interconnect"),
                        U_LOCAL_INTERCONNECT   => println!("        --1---- -: input = U  local interconnect"),
                        HORIZONTAL_MEDIUM_BUS  => println!("        -1----- -: input = horizontal medium bus"),
                        VERTICAL_MEDIUM_BUS    => println!("        1------ -: input = vertical medium bus"),
                        0                      => println!("        0000000 -: input = VCC"),
                        _                      => panic!("illegal config {config:010b} for imux_2")
                    }
                    match properties {
                        0 => println!("        properties=0: \"route\" noninverted"),
                        1 => println!("        properties=1: \"inv\" inverted"),
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
                    const INPUT_NORMAL: i64 = 0b00_0000_0001;
                    const HORIZONTAL_MEDIUM_BUS: i64 = 0b10_0000_0100;
                    const VERTICAL_MEDIUM_BUS: i64 = 0b10_0000_1000;
                    const UU_LOCAL_INTERCONNECT: i64 = 0b10_0001_0000;
                    const L_LOCAL_INTERCONNECT: i64 = 0b10_0010_0000;
                    const FF_LOCAL_INTERCONNECT: i64 = 0b10_0100_0000;
                    const FB_LOCAL_INTERCONNECT: i64 = 0b10_1000_0000;
                    const F_LOCAL_INTERCONNECT: i64 = 0b11_0000_0000;

                    println!("    Z=1 - \"imux_1\" B input mux:");
                    // Output is in order of config bits in the bitstream
                    if config & INPUT_NORMAL == INPUT_NORMAL {
                        println!("        ------- 1: input polarity normal");
                    } else {
                        println!("        ------- 0: input polarity inverted");
                    }
                    match config & !1 {
                        HORIZONTAL_MEDIUM_BUS => println!("        ------1 -: input = horizontal medium bus"),
                        FB_LOCAL_INTERCONNECT => println!("        -----1- -: input = FB local interconnect"),
                        L_LOCAL_INTERCONNECT  => println!("        ----1-- -: input = L  local interconnect"),
                        F_LOCAL_INTERCONNECT  => println!("        ---1--- -: input = F  local interconnect"),
                        UU_LOCAL_INTERCONNECT => println!("        --1---- -: input = UU local interconnect"),
                        VERTICAL_MEDIUM_BUS   => println!("        -1----- -: input = vertical medium bus"),
                        FF_LOCAL_INTERCONNECT => println!("        1------ -: input = FF local interconnect"),
                        0                     => println!("        0000000 -: input = VCC"),
                        _                     => panic!("illegal config {config:010b} for imux_1")
                    }
                    match properties {
                        0 => println!("        properties=0: \"route\" noninverted"),
                        1 => println!("        properties=1: \"inv\" inverted"),
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

                    println!("    Z=2 - \"func\" function:");

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
                        0 => println!("        properties=0: \"and\" and"),
                        1 => println!("        properties=1: \"buf\" buffer"),
                        2 => println!("        properties=2: \"tie_hi\" pull-up"),
                        3 => println!("        properties=3: \"tie_lo\" pull-down"),
                        4 => println!("        properties=4: \"xor\" (if type 2/4) / \"flipflop\" (if type 3)"),
                        5 => println!("        properties=5: \"mux\" (if type 4) / \"latch\" (if type 3)"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                3 => {
                    assert!(
                        config <= 524287,
                        "wired-OR: {config} has more than 19 config bits"
                    );
                    assert!(
                        properties <= 0,
                        "wired-OR: {properties} has more than 0 properties"
                    );
                    const WIRED_OR: i64 = 0b010_0000_0000_0000_0000;

                    println!("    Z=3 - \"omux\" wired-OR");
                    match config {
                        WIRED_OR => println!("        1: wired-OR"),
                        _       => (), // a lot of virtual cruft
                    }
                }
                4 => {
                    assert!(config <= 1023, "4: {config} has more than 10 config bits");
                    assert!(
                        properties <= 0,
                        "4: {properties} has more than 0 properties"
                    );
                    println!("    Z=4 - \"imux_c1\" Clock?: {config:010b}");
                    if config == 4 {
                        println!("        1: enabled??")
                    }
                }
                5 => {
                    assert!(config <= 1023, "5: {config} has more than 10 config bits");
                    assert!(
                        properties == 0,
                        "5: {properties} has more than 0 properties"
                    );
                    println!("    Z=5 - \"imux_rst\" Reset");
                    if config == 0x204 {
                        println!("        0: enabled??")
                    }
                }
                6 => {
                    assert!(config <= 3, "X Bus: {config} has more than 2 config bits");
                    assert!(
                        properties <= 1,
                        "X Bus: {properties} has exactly 2 properties"
                    );
                    println!("    Z=6 - \"x_switch\" X Bus:");
                    const LEFT_VERTICAL_X_BUS: i64 = 0b01;
                    const BELOW_HORIZONTAL_X_BUS: i64 = 0b10;
                    if config & (LEFT_VERTICAL_X_BUS | BELOW_HORIZONTAL_X_BUS) == (LEFT_VERTICAL_X_BUS | BELOW_HORIZONTAL_X_BUS) {
                        println!("        1: connect vertical and horizontal X bus");
                    }
                    match properties {
                        0 => println!("        properties=0: \"v2h\" [VIRTUAL] vertical-to-horizontal"),
                        1 => println!("        properties=1: \"h2v\" [VIRTUAL] horizontal-to-vertical"),
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

                    println!("    Z=7 - \"bmux\" function??:");
                    if config & BELOW_HORIZONTAL_MEDIUM_BUS == BELOW_HORIZONTAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- ---- -1--: connect to below horizontal medium bus"
                        );
                    }
                    if config & ABOVE_HORIZONTAL_MEDIUM_BUS == ABOVE_HORIZONTAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- ---1 ----: connect to above horizontal medium bus"
                        );
                    }
                    if config & LEFT_VERTICAL_MEDIUM_BUS == LEFT_VERTICAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- ---- 1---: connect to left vertical medium bus"
                        );
                    }

                    if config & RIGHT_VERTICAL_MEDIUM_BUS == RIGHT_VERTICAL_MEDIUM_BUS {
                        println!(
                            "        --- ---- ---- --1- ----: connect to right vertical medium bus"
                        );
                    }
                    if config & (UNKNOWN_BIT18 | UNKNOWN_BIT0) == (UNKNOWN_BIT18 | UNKNOWN_BIT0) {
                        println!("        1-- ---- ---- ---- ---1: ???");
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
                    assert!(
                        config <= 2047,
                        "Pad/P-Bus Output: {config} has more than 11 config bits"
                    );
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

    fn disassemble_vertical_port_cell(&self, subelements: &[(i64, i64, i64)]) {
        for &(z, properties, config) in subelements {
            match z {
                0 => {
                    assert!(config <= 31, "0: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "0: {properties} has more than 2 properties"
                    );
                    println!("    Z=0 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                1 => {
                    assert!(config <= 31, "1: {config} has more than 5 config bits");
                    assert!(
                        properties <= 2,
                        "1: {properties} has more than 2 properties"
                    );
                    println!("    Z=1 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                2 => {
                    assert!(config <= 31, "2: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "2: {properties} has more than 2 properties"
                    );
                    println!("    Z=2 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                3 => {
                    assert!(config <= 255, "3: {config} has more than 8 config bits");
                    assert!(
                        properties <= 1,
                        "3: {properties} has more than 2 properties"
                    );
                    println!("    Z=3 - ??: {config:08b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                4 => {
                    assert!(config <= 127, "4: {config} has more than 7 config bits");
                    assert!(
                        properties <= 1,
                        "4: {properties} has more than 2 properties"
                    );
                    println!("    Z=4 - ??: {config:07b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                5 => {
                    assert!(config <= 63, "5: {config} has more than 6 config bits");
                    assert!(
                        properties <= 1,
                        "5: {properties} has more than 2 properties"
                    );
                    println!("    Z=5 - ??: {config:06b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                6 => {
                    assert!(config <= 31, "6: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "6: {properties} has more than 2 properties"
                    );
                    println!("    Z=6 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                7 => {
                    assert!(config <= 31, "7: {config} has more than 5 config bits");
                    assert!(
                        properties <= 2,
                        "7: {properties} has more than 2 properties"
                    );
                    println!("    Z=7 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                8 => {
                    assert!(config <= 31, "8: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "8: {properties} has more than 2 properties"
                    );
                    println!("    Z=8 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                _ => panic!(
                    "    don't know how to disassemble vertical port cell z = {z} with config {config:b}"
                ),
            }
        }
    }

    fn disassemble_horizontal_port_cell(&self, subelements: &[(i64, i64, i64)]) {
        for &(z, properties, config) in subelements {
            match z {
                0 => {
                    assert!(config <= 31, "0: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "0: {properties} has more than 2 properties"
                    );
                    println!("    Z=0 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                1 => {
                    assert!(config <= 31, "1: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "1: {properties} has more than 2 properties"
                    );
                    println!("    Z=1 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                2 => {
                    assert!(config <= 31, "2: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "2: {properties} has more than 2 properties"
                    );
                    println!("    Z=2 - \"port_mbb_right\" ??: {config:05b}");
                    if config & 2 == 2 {
                        println!("        ---1-: ??");
                    }
                    match properties {
                        0 => println!("        properties=0: \"input\""),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                3 => {
                    assert!(config <= 255, "3: {config} has more than 8 config bits");
                    assert!(
                        properties <= 1,
                        "3: {properties} has more than 2 properties"
                    );
                    println!("    Z=3 - ??: {config:08b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                4 => {
                    assert!(config <= 255, "4: {config} has more than 8 config bits");
                    assert!(
                        properties <= 1,
                        "4: {properties} has more than 2 properties"
                    );
                    println!("    Z=4 - ??: {config:08b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                5 => {
                    assert!(config <= 63, "5: {config} has more than 6 config bits");
                    assert!(
                        properties <= 1,
                        "5: {properties} has more than 2 properties"
                    );
                    println!("    Z=5 - \"port_worb_right\" ??: {config:06b}");
                    if config & 2 == 2 {
                        println!("        ----1-: wired-OR");
                    }
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                6 => {
                    assert!(config <= 31, "6: {config} has more than 5 config bits");
                    assert!(
                        properties <= 2,
                        "6: {properties} has more than 2 properties"
                    );
                    println!("    Z=6 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                7 => {
                    assert!(config <= 31, "7: {config} has more than 5 config bits");
                    assert!(
                        properties <= 2,
                        "7: {properties} has more than 2 properties"
                    );
                    println!("    Z=7 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                8 => {
                    assert!(config <= 31, "8: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "8: {properties} has more than 2 properties"
                    );
                    println!("    Z=8 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                _ => panic!(
                    "    don't know how to disassemble horizontal port cell z = {z} with config {config:b}"
                ),
            }
        }
    }

    fn disassemble_clock_port_cell(&self, subelements: &[(i64, i64, i64)]) {
        for &(z, properties, config) in subelements {
            match z {
                0 => {
                    assert!(config <= 3, "0: {config} has more than 2 config bits");
                    assert!(
                        properties <= 1,
                        "0: {properties} has more than 2 properties"
                    );
                    println!("    Z=0 - ??: {config:02b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                1 => {
                    assert!(config <= 31, "1: {config} has more than 5 config bits");
                    assert!(
                        properties <= 0,
                        "1: {properties} has more than 0 properties"
                    );
                    println!("    Z=1 - ??: {config:05b}");
                }
                2 => {
                    assert!(config <= 1023, "2: {config} has more than 10 config bits");
                    assert!(
                        properties <= 1,
                        "2: {properties} has more than 2 properties"
                    );
                    println!("    Z=2 - ??: {config:010b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                3 => {
                    assert!(config <= 1023, "3: {config} has more than 10 config bits");
                    assert!(
                        properties <= 1,
                        "3: {properties} has more than 2 properties"
                    );
                    println!("    Z=3 - ??: {config:010b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                4 => {
                    assert!(config <= 31, "4: {config} has more than 5 config bits");
                    assert!(
                        properties <= 1,
                        "4: {properties} has more than 2 properties"
                    );
                    println!("    Z=4 - ??: {config:05b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                5 => {
                    assert!(config <= 63, "5: {config} has more than 6 config bits");
                    assert!(
                        properties <= 1,
                        "5: {properties} has more than 2 properties"
                    );
                    println!("    Z=5 - ??: {config:06b}");
                    match properties {
                        0 => println!("        properties=0: ??"),
                        1 => println!("        properties=1: ??"),
                        _ => panic!("unknown properties {properties}"),
                    }
                }
                6 => {
                    assert!(config <= 31, "6: {config} has more than 5 config bits");
                    assert!(
                        properties <= 0,
                        "6: {properties} has more than 0 properties"
                    );
                    println!("    Z=6 - ??: {config:05b}");
                }
                7 => {
                    assert!(config <= 31, "7: {config} has more than 5 config bits");
                    assert!(
                        properties <= 0,
                        "7: {properties} has more than 0 properties"
                    );
                    println!("    Z=7 - ??: {config:05b}");
                }
                8 => {
                    assert!(config <= 31, "8: {config} has more than 5 config bits");
                    assert!(
                        properties <= 0,
                        "8: {properties} has more than 0 properties"
                    );
                    println!("    Z=8 - ??: {config:05b}");
                }
                _ => panic!(
                    "    don't know how to disassemble clock port cell z = {z} with config {config:b}"
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
            let standalone_core_cell =
                matches!(self.shape_transform, (0, 0, 0, 0, 0, 8)) && x == 0 && y == 0;
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
            let within_horizontal_port_cell =
                (x == 14 || x == 26 || x == 38 || x == 50 || x == 62 || x == 74)
                    && ((4_i64..14).contains(&y)
                        || (16_i64..26).contains(&y)
                        || (28_i64..38).contains(&y)
                        || (40_i64..50).contains(&y)
                        || (52_i64..62).contains(&y)
                        || (64_i64..74).contains(&y));
            let within_vertical_port_cell = ((4_i64..14).contains(&x)
                || (16_i64..26).contains(&x)
                || (28_i64..38).contains(&x)
                || (40_i64..50).contains(&x)
                || (52_i64..62).contains(&x)
                || (64_i64..74).contains(&x))
                && (y == 14 || y == 26 || y == 38 || y == 50 || y == 62 || y == 74);
            let within_clock_port_cell =
                (x == 14 || x == 26 || x == 38 || x == 50 || x == 62 || x == 74)
                    && (y == 14 || y == 26 || y == 38 || y == 50 || y == 62 || y == 74);
            if standalone_core_cell || within_core_cell {
                println!("({x}, {y}) = Core Cell:");
                self.disassemble_core_cell(1, &subelements);
            } else if within_io_pad {
                println!("({x}, {y}) = I/O Pad:");
                self.disassemble_io_pad(&subelements);
            } else if within_peripheral_bus {
                println!("({x}, {y}) = Peripheral Bus?:");
                self.disassemble_peripheral_bus(&subelements);
            } else if within_horizontal_port_cell {
                println!("({x}, {y}) = Horizontal Port Cell:");
                self.disassemble_horizontal_port_cell(&subelements);
            } else if within_vertical_port_cell {
                println!("({x}, {y}) = Vertical Port Cell:");
                self.disassemble_vertical_port_cell(&subelements);
            } else if within_clock_port_cell {
                println!("({x}, {y}) = Clock Port Cell:");
                self.disassemble_clock_port_cell(&subelements);
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
