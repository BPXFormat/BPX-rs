use std::{fs::File, path::Path};
use bpx::container::Container;
use bpx::header::{BPX_CURRENT_VERSION, MainHeader, Struct};

/*use bpx::{decoder::Decoder, encoder::Encoder, header::BPX_CURRENT_VERSION, Interface};*/

#[test]
fn attempt_write_empty_bpxp()
{
    {
        let mut file = File::create(Path::new("./the_very_first_bpx.bpx")).unwrap();
        let mut container = Container::create(file, MainHeader::new());
        //let mut encoder = Encoder::new(&mut file).unwrap();
        container.save().unwrap();
    }
    {
        let mut file = File::open(Path::new("./the_very_first_bpx.bpx")).unwrap();
        let mut container = Container::open(file).unwrap();
        assert_eq!(container.get_main_header().section_num, 0);
        assert_eq!(container.get_main_header().version, BPX_CURRENT_VERSION);
        assert_eq!(container.get_main_header().file_size, 40);
    }
}

#[test]
fn sd_api_test()
{
    use std::convert::TryInto;

    use bpx::sd::Value;

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
