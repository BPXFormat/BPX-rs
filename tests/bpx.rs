use bpx::encoder::Encoder;
use bpx::decoder::Decoder;
use bpx::Interface;

#[test]
fn attempt_write_empty_bpxp()
{
    let mut encoder = Encoder::new(std::path::Path::new("./the_very_first_bpx.bpx")).unwrap();
    encoder.save().unwrap();
    let decoder = Decoder::new(std::path::Path::new("./the_very_first_bpx.bpx")).unwrap();
    assert_eq!(decoder.get_main_header().section_num, 0);
    assert_eq!(decoder.get_main_header().version, 1);
    assert_eq!(decoder.get_main_header().file_size, 40);
}
