#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use fantastic_time::img::*;
    #[test]
    fn test_num() {
        println!("{}", ImageFormat::Jpeg.as_ref());
        let path = OsStr::new("tests/imgs/IMG_5745.JPG");
        let r = change_img_format(path.as_ref(), ImageFormat::Png);
        println!("convert result is:{:?}", r.unwrap().len());
        let r = gen_new_format_image(path.as_ref(), "/tmp/test.png".as_ref(), ImageFormat::Png);
        println!("r is:{:?}", r);
    }
}
