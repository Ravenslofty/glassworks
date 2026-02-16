use std::assert_matches;
use std::fs::File;
use std::io::{self, Read};

use lexpr::Value;

#[derive(Debug, Default)]
struct Element {
    first_five: (i64, i64, i64, i64, i64),
    sixth: i64,
    net: String,
    port_name: String,
    port_index: i64,
}

#[derive(Debug, Default)]
struct E {
    transform: (i64, i64, i64, i64, i64, i64),
}

#[derive(Debug, Default)]
struct Layout {
    key_transform: (i64, i64, i64, i64, i64, i64),
    shape_transform: (i64, i64, i64, i64, i64, i64),
    elements: Vec<Element>,
    e: Vec<E>,
}

impl Layout {
    pub fn new(r: impl Read) -> Option<Self> {
        let mut this = Self::default();

        let layout = lexpr::from_reader(r).ok()?;
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
                    assert_eq!(item[2].as_str().unwrap(), "mpa1000");
                    assert!(item[3].is_cons());
                    let t = item[3].to_vec()?;
                    assert_eq!(t[0].as_symbol().unwrap(), "t");
                    this.key_transform.0 = t[1].as_number()?.as_i64()?;
                    this.key_transform.1 = t[2].as_number()?.as_i64()?;
                    this.key_transform.2 = t[3].as_number()?.as_i64()?;
                    this.key_transform.3 = t[4].as_number()?.as_i64()?;
                    this.key_transform.4 = t[5].as_number()?.as_i64()?;
                    this.key_transform.5 = t[6].as_number()?.as_i64()?;
                    assert_matches!(this.key_transform, (1, 0, 0, 1, _, _));
                    assert_matches!(
                        (this.key_transform.4, this.key_transform.5),
                        /* DFR */
                        (4, 5) | /* AND */ (10, 10) | /* DF */ (10, 11) | /* DFER */ (12, 17) | /* OR */ (16, 16) | /* DFE */ (16, 17) | /* XOR */ (17, 17)
                    );
                }
                "n" => {
                    assert_eq!(item[1].as_number()?.as_i64()?, 0);
                    assert_eq!(item[2].as_str()?, "internal");
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
                }
                "shape" => {
                    this.shape_transform.0 = item[1].as_number()?.as_i64()?;
                    this.shape_transform.1 = item[2].as_number()?.as_i64()?;
                    this.shape_transform.2 = item[3].as_number()?.as_i64()?;
                    this.shape_transform.3 = item[4].as_number()?.as_i64()?;
                    this.shape_transform.4 = item[5].as_number()?.as_i64()?;
                    this.shape_transform.5 = item[6].as_number()?.as_i64()?;
                    assert_eq!(this.shape_transform, (0, 0, 0, 0, 0, 8));
                    let item = item.to_vec()?;
                    for item in item.iter().skip(7) {
                        let symbol = item[0].as_symbol()?.to_string();
                        match symbol.as_str() {
                            "element" => {
                                let mut element = Element::default();
                                element.first_five.0 = item[1].as_number()?.as_i64()?;
                                element.first_five.1 = item[2].as_number()?.as_i64()?;
                                element.first_five.2 = item[3].as_number()?.as_i64()?;
                                element.first_five.3 = item[4].as_number()?.as_i64()?;
                                element.first_five.4 = item[5].as_number()?.as_i64()?;
                                element.sixth =
                                    i64::from_str_radix(&item[6].as_symbol()?[2..], 16).ok()?;
                                let net = item[7].to_vec()?;
                                assert_eq!(net[0].as_symbol()?, "net");
                                element.net = net[1].as_str()?.to_string();
                                let port = item[8].to_vec()?;
                                assert_eq!(port[0].as_symbol()?, "port");
                                element.port_name = port[1].as_str()?.to_string();
                                element.port_index = port[2].as_number()?.as_i64()?;
                                this.elements.push(element);
                            }
                            "e" => {
                                let mut e = E::default();
                                e.transform.0 = item[1].as_number()?.as_i64()?;
                                e.transform.1 = item[2].as_number()?.as_i64()?;
                                e.transform.2 = item[3].as_number()?.as_i64()?;
                                e.transform.3 = item[4].as_number()?.as_i64()?;
                                e.transform.4 = item[5].as_number()?.as_i64()?;
                                e.transform.5 = item[6].as_number()?.as_i64()?;
                                assert_matches!(e.transform, (0, 0, 2, _, _, 0));
                                assert_matches!(
                                    e.transform.3,
                                    /* AND/OR */ 0 | /* XOR */ 4 | /* DL */ 5
                                );
                                assert_matches!(
                                    e.transform.4,
                                    /* DF */
                                    14 | /* DFR */ 15 | /* AND/OR/XOR */ 28 | /* DFE */ 30 | /* DFER */ 31
                                );
                                this.e.push(e);
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

    pub fn element_by_port_name(&self, port_name: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.port_name == port_name)
    }
}

fn main() -> io::Result<()> {
    for filename in std::env::args().skip(1) {
        let file = File::open(&filename)?;
        println!("{filename}:");

        let layout = Layout::new(file).unwrap();
        println!("{layout:?}");
    }

    Ok(())
}
