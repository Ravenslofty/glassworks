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
                    assert_matches!(
                        this.shape_transform,
                        (0, 0, 0, 0, 0, 8) | (0, 0, 0, 78, 78, 8)
                    );
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

    pub fn disassemble(&self) {
        let mut elements = BTreeMap::<(i64, i64), Vec<(i64, i64, i64)>>::new();

        for element in &self.elements {
            let entry = elements.entry((element.x, element.y)).or_insert(vec![]);
            entry.push((element.z, element.properties, element.config));
        }

        for ((x, y), subelements) in elements {
            let standalone = matches!(self.shape_transform, (0, 0, 0, 0, 0, 8)) && x == 0 && y == 0;
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
            if standalone || within_core_cell {
                println!("({x}, {y}) = Core Cell:");
                let mut should_break = false;
                let mut should_break_2 = true;
                for (z, properties, config) in subelements {
                    match z {
                        0 => {
                            assert!(config <= 1023, "A input mux has more than 10 config bits");

                            /// This *might* be input inversion, but it only ever appears on buffers.
                            const UNKNOWN_BIT0: i64 = 0b00_0000_0001;
                            /// Never set?
                            const UNKNOWN_BIT1: i64 = 0b00_0000_0010;
                            const HORIZONTAL_MEDIUM_BUS: i64 = 0b10_0000_0100;
                            const VERTICAL_MEDIUM_BUS: i64 = 0b10_0000_1000;
                            const LL_LOCAL_INTERCONNECT: i64 = 0b10_0001_0000;
                            const U_LOCAL_INTERCONNECT: i64 = 0b10_0010_0000;
                            const FBB_LOCAL_INTERCONNECT: i64 = 0b10_0100_0000;
                            const FB_LOCAL_INTERCONNECT: i64 = 0b10_1000_0000;
                            const F_LOCAL_INTERCONNECT: i64 = 0b11_0000_0000;
                            /// Possibly a mux-enable bit
                            const UNKNOWN_BIT9: i64 = 0b10_0000_0000;

                            println!("    Z=0 - A input mux:");
                            if config & UNKNOWN_BIT0 == UNKNOWN_BIT0 {
                                println!("        -- ---- ---1: UNKNOWN bit 0 = 1 (buffer?)");
                            }
                            if config & UNKNOWN_BIT1 == UNKNOWN_BIT1 {
                                println!("        -- ---- --1-: UNKNOWN bit 1 = 1");
                                should_break = true;
                            }
                            if config & HORIZONTAL_MEDIUM_BUS == HORIZONTAL_MEDIUM_BUS {
                                println!("        1- ---- -1--: horizontal medium bus");
                            }
                            if config & VERTICAL_MEDIUM_BUS == VERTICAL_MEDIUM_BUS {
                                println!("        1- ---- 1---: vertical medium bus");
                            }
                            if config & LL_LOCAL_INTERCONNECT == LL_LOCAL_INTERCONNECT {
                                println!("        1- ---1 ----: LL local interconnect");
                            }
                            if config & U_LOCAL_INTERCONNECT == U_LOCAL_INTERCONNECT {
                                println!("        1- --1- ----: U local interconnect");
                            }
                            if config & FBB_LOCAL_INTERCONNECT == FBB_LOCAL_INTERCONNECT {
                                println!("        1- -1-- ----: FBB local interconnect");
                            }
                            if config & FB_LOCAL_INTERCONNECT == FB_LOCAL_INTERCONNECT {
                                println!("        1- 1--- ----: FB local interconnect");
                            }
                            if config & F_LOCAL_INTERCONNECT == F_LOCAL_INTERCONNECT {
                                println!("        11 ---- ----: F local interconnect");
                            }
                            if config & UNKNOWN_BIT9 == 0 {
                                println!("        0- ---- ----: mux enable? = 0");
                                should_break = true;
                            }
                        }
                        1 => {
                            assert!(config <= 1023, "B input mux has more than 10 config bits");

                            /// This *might* be input inversion, but it only ever appears on buffers.
                            const UNKNOWN_BIT0: i64 = 0b00_0000_0001;
                            /// Never set?
                            const UNKNOWN_BIT1: i64 = 0b00_0000_0010;
                            const HORIZONTAL_MEDIUM_BUS: i64 = 0b10_0000_0100;
                            const VERTICAL_MEDIUM_BUS: i64 = 0b10_0000_1000;
                            const UU_LOCAL_INTERCONNECT: i64 = 0b10_0001_0000;
                            const L_LOCAL_INTERCONNECT: i64 = 0b10_0010_0000;
                            const FF_LOCAL_INTERCONNECT: i64 = 0b10_0100_0000;
                            const FB_LOCAL_INTERCONNECT: i64 = 0b10_1000_0000;
                            const F_LOCAL_INTERCONNECT: i64 = 0b11_0000_0000;
                            /// Possibly a mux-enable bit
                            const UNKNOWN_BIT9: i64 = 0b10_0000_0000;

                            println!("    Z=1 - B input mux:");
                            if config & UNKNOWN_BIT0 == UNKNOWN_BIT0 {
                                println!("        -- ---- ---1: UNKNOWN bit 0 = 1 (buffer?)");
                            }
                            if config & UNKNOWN_BIT1 == UNKNOWN_BIT1 {
                                println!("        -- ---- --1-: UNKNOWN bit 1 = 1");
                                should_break = true;
                            }
                            if config & HORIZONTAL_MEDIUM_BUS == HORIZONTAL_MEDIUM_BUS {
                                println!("        1- ---- -1--: horizontal medium bus");
                            }
                            if config & VERTICAL_MEDIUM_BUS == VERTICAL_MEDIUM_BUS {
                                println!("        1- ---- 1---: vertical medium bus");
                            }
                            if config & UU_LOCAL_INTERCONNECT == UU_LOCAL_INTERCONNECT {
                                println!("        1- ---1 ----: UU local interconnect");
                            }
                            if config & L_LOCAL_INTERCONNECT == L_LOCAL_INTERCONNECT {
                                println!("        1- --1- ----: L local interconnect");
                            }
                            if config & FF_LOCAL_INTERCONNECT == FF_LOCAL_INTERCONNECT {
                                println!("        1- -1-- ----: FF local interconnect");
                            }
                            if config & FB_LOCAL_INTERCONNECT == FB_LOCAL_INTERCONNECT {
                                println!("        1- 1--- ----: FB local interconnect");
                            }
                            if config & F_LOCAL_INTERCONNECT == F_LOCAL_INTERCONNECT {
                                println!("        11 ---- ----: F local interconnect");
                            }
                            if config & UNKNOWN_BIT9 == 0 {
                                println!("        0- ---- ----: mux enable? = 0");
                                should_break = true;
                            }
                        }
                        2 => {
                            assert!(config <= 1023, "2: {config} has more than 10 config bits");
                            println!("    Z=2 - ??: {config:010b}");
                            match properties {
                                0 => println!("        properties=0 - logic element"),
                                1 => println!("        properties=1 - buffer"),
                                2 => println!("        properties=2 - pull-up"),
                                3 => println!("        properties=3 - pull-down"),
                                _ => panic!("unknown properties {properties}"),
                            }
                            if properties == 1 {
                                //should_break_2 = false;
                            }
                        }
                        3 => {
                            assert!(
                                config <= 524287,
                                "Q output mux has more than 19 config bits"
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

                            println!("    Z=3 - Q output mux??:");
                            if config & UNKNOWN_BIT0 == UNKNOWN_BIT0 {
                                println!("        --- ---- ---- ---- ---1: UNKNOWN bit 0 = 1 (buffer?)");
                            }
                            if config & LOCAL_INTERCONNECT_CONNECTED_TO_MEDIUM_BUS
                                == LOCAL_INTERCONNECT_CONNECTED_TO_MEDIUM_BUS
                            {
                                println!(
                                    "        --- ---- ---- ---- --1-: local interconnect connected to medium bus"
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
                                println!(
                                    "        --- ---- ---- -1-- ----: UU A local interconnect"
                                );
                            }
                            if config & LL_B_LOCAL_INTERCONNECT == LL_B_LOCAL_INTERCONNECT {
                                println!(
                                    "        --- ---- ---- 1--- ----: LL B local interconnect"
                                );
                            }
                            if config & U_B_LOCAL_INTERCONNECT == U_B_LOCAL_INTERCONNECT {
                                println!("        --- ---- ---1 ---- ----: U B local interconnect");
                            }
                            if config & L_A_LOCAL_INTERCONNECT == L_A_LOCAL_INTERCONNECT {
                                println!("        --- ---- --1- ---- ----: L A local interconnect");
                            }
                            if config & FBB_A_LOCAL_INTERCONNECT == FBB_A_LOCAL_INTERCONNECT {
                                println!(
                                    "        --- ---- -1-- ---- ----: FBB A local interconnect"
                                );
                            }
                            if config & UNKNOWN_BIT11 == UNKNOWN_BIT11 {
                                println!("        --- ---- 1--- ---- ----: UNKNOWN bit 11 = 1");
                                should_break = true;
                            }
                            if config & FF_B_LOCAL_INTERCONNECT == FF_B_LOCAL_INTERCONNECT {
                                println!(
                                    "        --- ---1 ---- ---- ----: FF B local interconnect"
                                );
                            }
                            if config & FB_A_LOCAL_INTERCONNECT == FB_A_LOCAL_INTERCONNECT {
                                println!(
                                    "        --- --1- ---- ---- ----: FB A local interconnect"
                                );
                            }
                            if config & FB_B_LOCAL_INTERCONNECT == FB_B_LOCAL_INTERCONNECT {
                                println!(
                                    "        --- -1-- ---- ---- ----: FB B local interconnect"
                                );
                            }
                            if config & F_A_LOCAL_INTERCONNECT == F_A_LOCAL_INTERCONNECT {
                                println!("        --- 1--- ---- ---- ----: F A local interconnect");
                            }
                            if config & F_B_LOCAL_INTERCONNECT == F_B_LOCAL_INTERCONNECT {
                                println!("        --1 ---- ---- ---- ----: F B local interconnect");
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
                            println!("    Z=4 - ??: {config:010b}");
                            should_break = true;
                        }
                        5 => {
                            assert!(config <= 1023, "5: {config} has more than 10 config bits");
                            println!("    Z=5 - ??: {config:010b}");
                            should_break = true;
                        }
                        6 => {
                            assert!(config <= 3, "X Bus: {config} has more than 2 config bits");
                            println!("    Z=6 - X Bus:");
                            const VERTICAL_MEDIUM_BUS: i64 = 0b01;
                            const HORIZONTAL_MEDIUM_BUS: i64 = 0b10;
                            if config & VERTICAL_MEDIUM_BUS == VERTICAL_MEDIUM_BUS {
                                println!("        -1: vertical medium bus");
                            }
                            if config & HORIZONTAL_MEDIUM_BUS == HORIZONTAL_MEDIUM_BUS {
                                println!("        1-: horizontal medium bus");
                            }
                        }
                        7 => {
                            assert!(config <= 524287, "function: {config} has more than 19 config bits");
                            println!("    Z=7 - function??: {config:019b}");
                        }
                        8 => {
                            assert!(config <= 1023, "8: {config} has more than 10 config bits");
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
        println!("{layout:?}");

        layout.disassemble();
    }

    Ok(())
}
