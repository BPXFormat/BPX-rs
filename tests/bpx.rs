use bpx::encoder::Encoder;
use bpx::decoder::Decoder;
use bpx::Interface;
use std::fs::File;
use std::path::Path;

#[test]
fn attempt_write_empty_bpxp()
{
    {
        let mut file = File::create(Path::new("./the_very_first_bpx.bpx")).unwrap();
        let mut encoder = Encoder::new(&mut file).unwrap();
        encoder.save().unwrap();
    }
    {
        let mut file = File::open(Path::new("./the_very_first_bpx.bpx")).unwrap();
        let decoder = Decoder::new(&mut file).unwrap();
        assert_eq!(decoder.get_main_header().section_num, 0);
        assert_eq!(decoder.get_main_header().version, 1);
        assert_eq!(decoder.get_main_header().file_size, 40);
    }
}
