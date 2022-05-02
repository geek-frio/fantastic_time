use std::ffi::OsStr;
use std::ops::ControlFlow;
use std::path::Path;
use std::convert::AsRef;
use anyhow::Error as AnyError;
use std::fs;

/// 其职责是发现所有的图片&视频文件,存储其具体的位置
pub struct Scanner {
}

impl  Scanner {
    fn scan<P>(root: P) ->  Result<(), AnyError> where P: AsRef<OsStr>{
        let root = Path::new(root.as_ref());

        let mut dir_content= fs::read_dir(root)?;        
        dir_content.for_each(|item| {
            
        });
        return Ok(());
    }


}