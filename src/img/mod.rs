mod hash;

use anyhow::Error as AnyError;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, TimeZone};
use regex::Regex;
use std::io::Write;
use std::ops::ControlFlow;
use std::vec;
use std::{fs::OpenOptions, path::Path, sync::Once};

use lazy_static::lazy_static;
use magick_rust::{magick_wand_genesis, MagickWand};
use strum_macros::{AsRefStr, EnumString};

const CORRECT_DATETIME_PROP: [&'static str; 3] = [
    "exif:DateTime",         // format: 2016:03:17 12:43:55
    "exif:DateTimeOriginal", // format: 2016:03:17 12:43:55
    "exif:GPSDateStamp",     // format: 2016:03:17
];
// 2022-05-04T12:40:18+00:00 (RFC3339 format)
const IMG_FILE_DATETIME_DROP: [&'static str; 2] = ["date:create", "date:modify"];

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

// 信息获取的有效性
#[derive(Debug, PartialEq)]
pub enum InfoValidScore {
    High,
    Middle,
    Low,
}

#[derive(Debug)]
pub struct ImgMeta {
    pub sig: Option<String>,
    pub time: Option<NaiveDateTime>,
    pub score: Option<InfoValidScore>,
}

impl ImgMeta {}

// 获取图片的时间信息
// 1: 图片中含有extif: DateTime或者GPS datetime信息
// 2: 文件名中含有时间信息
// 3: 使用文件的创建时间
pub fn retrive_img_datetime(path: &Path) -> Result<ImgMeta, AnyError> {
    let path_str = path.as_os_str().to_str();
    let mut date_time = None;

    if let Some(s) = path_str {
        date_time = retrieve_filename_datetime(s);
    }

    let wand = MagickWand::new();
    wand.read_image(
        path.as_os_str()
            .to_str()
            .ok_or(AnyError::msg("path to str failed!"))?,
    )?;

    let mut img_meta = ImgMeta {
        sig: None,
        time: None,
        score: None,
    };

    let sign = wand.get_image_property("signature");
    // let _ = sign.map(|r| {
    //     img_meta.sig = Some(r);
    // });
    sign.into_iter().for_each(|item| {
        img_meta.sig = Some(item);
    });

    if img_meta.sig.is_none() {}

    let res = retrieve_meta_datetime(&wand);
    if let Some((d, score)) = res {
        if score == InfoValidScore::High || score == InfoValidScore::Middle {
            img_meta.time = Some(d);
            img_meta.score = Some(score);
        } else if score == InfoValidScore::Low && date_time.is_some() {
            img_meta.time = Some(date_time.unwrap().0);
            img_meta.score = Some(score);
        } else {
            img_meta.time = Some(date_time.unwrap().0);
            img_meta.score = Some(InfoValidScore::Low);
        }
    }

    Ok(img_meta)
}

// 获取文件名中的年月日
pub fn retrieve_filename_datetime(name: &str) -> Option<(NaiveDateTime, InfoValidScore)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"((((19|20)\d{2})(0?[13578]|1[02])(0?[1-9]|[12]\d|3[01]))|(((19|20)\d{2})(0?[469]|11)(0?[1-9]|[12]\d|30))|(((19|20)\d{2})0?2(0?[1-9]|1\d|2[0-8]))|((((19|20)([13579][26]|[2468][048]|0[48]))|(2000))0?2(0?[1-9]|[12]\d)))$").unwrap();
    }
    RE.captures(name).and_then(|el| {
        let r = el.get(0);
        r.map_or(None, |el| {
            let mut date_time_str = String::new();
            date_time_str.push_str(el.as_str());
            date_time_str.push_str(" 00:00:00");
            let date_time =
                NaiveDateTime::parse_from_str(date_time_str.as_str(), "%Y%m%d %H:%M:%S");
            println!("parse result is:{:?}", date_time);
            match date_time {
                Ok(d) => Some((d, InfoValidScore::Middle)),
                Err(_) => None,
            }
        })
    })
}

fn parse_in_multi_formats(time: &str, formats: Vec<&'static str>) -> Option<NaiveDateTime> {
    for fmt in formats.into_iter() {
        let r = NaiveDateTime::parse_from_str(time, fmt);
        if r.is_ok() {
            return Some(r.unwrap());
        }
    }
    None
}

// 读取图片meta信息
pub fn retrieve_meta_datetime(img: &MagickWand) -> Option<(NaiveDateTime, InfoValidScore)> {
    // let mut s = None;
    let r = CORRECT_DATETIME_PROP.iter().try_for_each(|el| {
        let res = img.get_image_property(*el);
        if let Ok(time_str) = res {
            return if (*el).contains("DateTime") {
                let date_time = parse_in_multi_formats(
                    time_str.as_ref(),
                    vec!["%Y-%m-%d %H:%M:%S", "%Y:%m:%d %H:%M:%S"],
                );

                match date_time {
                    Some(s) => ControlFlow::Break((s, InfoValidScore::High)),
                    None => ControlFlow::Continue(()),
                }
            } else {
                let date_time = NaiveDateTime::parse_from_str(time_str.as_ref(), "%Y-%m-%d");
                match date_time {
                    Ok(s) => ControlFlow::Break((s, InfoValidScore::Middle)),
                    Err(_) => ControlFlow::Continue(()),
                }
            };
        }
        ControlFlow::Continue(())
    });
    return match r {
        ControlFlow::Break(s) => Some(s),
        ControlFlow::Continue(_) => {
            let r = IMG_FILE_DATETIME_DROP.iter().try_for_each(|el| {
                let res = img.get_image_property(*el);
                if let Ok(time_str) = res {
                    let date_time = DateTime::parse_from_rfc3339(time_str.as_ref());
                    return match date_time {
                        Ok(s) => ControlFlow::Break((s.naive_local(), InfoValidScore::Low)),
                        Err(_) => ControlFlow::Continue(()),
                    };
                }
                ControlFlow::Continue(())
            });
            match r {
                ControlFlow::Break(s) => Some(s),
                ControlFlow::Continue(_) => None,
            }
        }
    };
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
    return match res {
        Ok(_) => {
            let blob = wand.write_images_blob(format.as_ref()).unwrap();
            Ok(blob)
        }
        Err(s) => Err(AnyError::msg(s)),
    };
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
    use magick_rust::MagickWand;

    use crate::img::InfoValidScore;

    use super::retrieve_meta_datetime;

    #[test]
    fn test_retrive_meta() {
        // rust png don't have DateTime info
        let img = MagickWand::new();
        let _ = img.read_image("tests/imgs/rust.png");
        let r = retrieve_meta_datetime(&img);
        assert!(r.unwrap().1 == InfoValidScore::Low);
    }
}
