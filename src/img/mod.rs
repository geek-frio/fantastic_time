use anyhow::Error as AnyError;
use chrono::{DateTime, Local, TimeZone};
use std::io::Write;
use std::{fs::OpenOptions, path::Path, sync::Once};

use magick_rust::{magick_wand_genesis, MagickWand};
use strum_macros::{AsRefStr, EnumString};

// 图片中关于图像的属性
const DATETIME_PROP: [&'static str; 5] = [
    "exif:DateTime",         // 2016:03:17 12:43:55
    "exif:DateTimeOriginal", // 2016:03:17 12:43:55
    "exif:GPSDateStamp",     // 2016:03:17
    "date:create",           // 2022-05-05T03:48:40+00:00
    "date:modify",           // 2022-05-05T03:48:40+00:00
];
const SIGNATURE_PROP: [&'static str; 1] = ["signature"];

// 只执行一次，初始化image magic
static START: Once = Once::new();

#[derive(Debug)]
pub struct ImageMeta {
    create_time: Option<DateTime<Local>>,
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
) -> Result<(DateTime<Local>, InfoValidScore), AnyError> {
    return Err(AnyError::msg("无法判断图片具体的创建时间"));
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
