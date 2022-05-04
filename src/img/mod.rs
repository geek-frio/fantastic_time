use anyhow::Error as AnyError;
use std::io::Write;
use std::{fs::OpenOptions, path::Path, sync::Once};

use magick_rust::{magick_wand_genesis, MagickWand};
use strum_macros::{AsRefStr, EnumString};

// 只执行一次，初始化image magic
static START: Once = Once::new();

#[derive(Debug, PartialEq, EnumString, AsRefStr)]
pub enum ImageFormat {
    #[strum(serialize = "jpeg")]
    Jpeg,
    #[strum(serialize = "jpg")]
    Jpg,
    #[strum(serialize = "png")]
    Png,
}

pub fn change_img_format(path: &Path, format: ImageFormat) -> Result<Vec<u8>, AnyError> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand = MagickWand::new();
    let res = wand.read_image(
        path.as_os_str()
            .to_str()
            .ok_or(AnyError::msg("Path osstr to str failed"))?,
    );
    match res {
        Ok(_) => {
            let blob = wand.write_images_blob(format.as_ref()).unwrap();
            return Ok(blob);
        }
        Err(s) => {
            return Err(AnyError::msg(s));
        }
    }
}

pub fn gen_new_format_image(
    src_path: &Path,
    dest_path: &Path,
    dest_format: ImageFormat,
) -> Result<(), AnyError> {
    let dest_img = change_img_format(src_path, dest_format)?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        // either use ? or unwrap since it returns a Result
        .open(dest_path)?;
    return Ok(file.write_all(&dest_img)?);
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    #[test]
    fn test_num() {
        println!("{}", ImageFormat::Jpeg.as_ref());
        let path = OsStr::new("/home/frio/workspace/rust/fantastic_time/src/img/data/IMG_5745.JPG");
        let r = change_img_format(path.as_ref(), ImageFormat::Png);
        println!("convert result is:{:?}", r.unwrap().len());
        let r = gen_new_format_image(path.as_ref(), "/tmp/test.png".as_ref(), ImageFormat::Png);
        println!("r is:{:?}", r);
    }
}
