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

#[test]
fn sd_api_test()
{
    use bpx::sd::Value;
    use std::convert::TryInto;

    let v = Value::from(None as Option<i32>);
    let v1 = Value::from("test");
    let v2 = Value::from(Some(0));
    let vu: Option<i32> = v.try_into().unwrap();
    let v1u: String = v1.try_into().unwrap();
    let v2u: Option<i32> = v2.try_into().unwrap();

    assert_eq!(vu, None);
    assert_eq!(v1u, String::from("test"));
    assert_eq!(v2u, Some(0));
}
