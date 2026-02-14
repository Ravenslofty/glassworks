use glassworks::Bitstream;

fn main() {
    let bytes = include_bytes!("../bitstreams/and/mpa1036.bit");
    let _ = Bitstream::new(bytes.as_slice()).unwrap();
    //let bytes = include_bytes!("../src/and_mpa1036.bit");
    //let _ = Bitstream::new(bytes.as_slice()).unwrap();
}
