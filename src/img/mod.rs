use anyhow::Error as AnyError;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, TimeZone};
use std::io::Write;
use std::ops::ControlFlow;
use std::{fs::OpenOptions, path::Path, sync::Once};

use magick_rust::{magick_wand_genesis, MagickWand};
use strum_macros::{AsRefStr, EnumString};

// 图片中关于图像的属性
// NaiveDateTime format
const CORRECT_DATETIME_PROP: [&'static str; 3] = [
    "exif:DateTime",         // format: 2016:03:17 12:43:55
    "exif:DateTimeOriginal", // format: 2016:03:17 12:43:55
    "exif:GPSDateStamp",     // format: 2016:03:17
];

// 2022-05-04T12:40:18+00:00 (RFC3339 format)
const IMG_FILE_DATETIME_DROP: [&'static str; 2] = ["date:create", "date:modify"];

const SIGNATURE_PROP: [&'static str; 1] = ["signature"];

// 只执行一次，初始化image magic
static START: Once = Once::new();

#[derive(Debug)]
pub struct ImageMeta {
    create_time: Option<DateTime<FixedOffset>>,
    gps: (f64, f64),
}

#[derive(Debug, PartialEq, EnumString, AsRefStr)]
pub enum ImageFormat {
    #[strum(serialize = "jpeg")]
    Jpeg,
    #[strum(serialize = "jpg")]
    Jpg,
    #[strum(serialize = "png")]
    Png,
}

pub fn retrive_img_meta(path: &Path) -> Result<ImageFormat, AnyError> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand = MagickWand::new();
    wand.read_image(
        path.to_str()
            .ok_or(AnyError::msg("Convert path to str failed!"))?,
    )?;
    return Err(AnyError::msg("error"));
}

// 信息获取的有效性
enum InfoValidScore {
    High,
    Middle,
    Low,
}

// 1. 尝试从图片中获取时间相关信息
// 2. 从文件名中获取时间相关信息
// 3. 根据图片的创建时间获取
fn try_retrieve_img_create_time_info(
    img: &MagickWand,
) -> Option<(DateTime<FixedOffset>, InfoValidScore)> {
    // let mut s = None;
    let r = CORRECT_DATETIME_PROP.iter().try_for_each(|el| {
        let res = img.get_image_property(*el);
        if let Ok(time_str) = res {
            if (*el).contains("DateTime") {
                let date_time = DateTime::parse_from_str(time_str.as_ref(), "%Y-%m-%d %H:%M:%S");
                match date_time {
                    Ok(s) => {
                        return ControlFlow::Break((s, InfoValidScore::High));
                    }
                    Err(_) => {
                        return ControlFlow::Continue(());
                    }
                }
            } else {
                let date_time = DateTime::parse_from_str(time_str.as_ref(), "%Y-%m-%d");
                match date_time {
                    Ok(s) => return ControlFlow::Break((s, InfoValidScore::Middle)),
                    Err(_) => {
                        return ControlFlow::Continue(());
                    }
                }
            }
        }
        ControlFlow::Continue(())
    });
    match r {
        ControlFlow::Break(s) => return Some(s),
        ControlFlow::Continue(_) => {
            let r = IMG_FILE_DATETIME_DROP.iter().try_for_each(|el| {
                let res = img.get_image_property(*el);
                if let Ok(time_str) = res {
                    let date_time = DateTime::parse_from_rfc3339(time_str.as_ref());
                    match date_time {
                        Ok(s) => {
                            return ControlFlow::Break((s, InfoValidScore::Low));
                        }
                        Err(_) => return ControlFlow::Continue(()),
                    }
                }
                ControlFlow::Continue(())
            });
            match r {
                ControlFlow::Break(s) => return Some(s),
                ControlFlow::Continue(_) => return None,
            }
        }
    }
}

pub fn change_img_format(path: &Path, format: ImageFormat) -> Result<Vec<u8>, AnyError> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand = MagickWand::new();
    let res = wand.read_image(
        path.to_str()
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
    use chrono::DateTime;

    #[test]
    fn test_parse_date_time() {
        let rfc3339 = DateTime::parse_from_rfc3339("1996-12-19T16:39:57+00:00");
        println!("rfc3339 is :{:?}", rfc3339);
    }
}
