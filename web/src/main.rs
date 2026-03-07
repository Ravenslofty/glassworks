use std::{fs::File, io::Read};
use std::io::Write;

use tera::{Context, Tera};

fn main() -> tera::Result<()> {
    let tera = Tera::new("templates/*").inspect_err(|e| eprintln!("{:?}", e.kind))?;
    let mut context = Context::new();

    {
        let mut s = String::new(); 
        let mut f = File::open("rendered/tile-topzonet3t4.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("topzonet3t4", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-botzonet3t4.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("botzonet3t4", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-topzonet1t2.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("topzonet1t2", &s);
    }

    {
        let mut s = String::new(); 
        let mut f = File::open("rendered/tile-botzonet1t2.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("botzonet1t2", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-wor-imux_2.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("wor_imux_2", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-xnor1-imux_2.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("xnor1_imux_2", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-ff-imux_2.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("ff_imux_2", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-xnor2-imux_2.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("xnor2_imux_2", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-wor-imux_1.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("wor_imux_1", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-xnor1-imux_1.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("xnor1_imux_1", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-ff-imux_1.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("ff_imux_1", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-xnor2-imux_1.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("xnor2_imux_1", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-wor-botbmux.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("wor_botbmux", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-wor-topbmux.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("wor_topbmux", &s);
    }
    
    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-xnor1-botbmux.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("xnor1_botbmux", &s);
    }

    {
        let mut s = String::new();
        let mut f = File::open("rendered/tile-xnor1-topbmux.svg")?;
        f.read_to_string(&mut s)?;
        context.insert("xnor1_topbmux", &s);
    }

    let rendered = tera.render("tile.tera", &context)?;
    {
        let mut tile_html = File::create("rendered/tile.html")?;
        write!(tile_html, "{rendered}")?;
    }

    Ok(())
}
