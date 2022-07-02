use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// 啦啦啦
struct MainArgs {
    #[clap(short, long)]
    meta_path: Option<String>,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// 扫描配置目录下所有的图片文件夹，并为其中的所有的图片建立档案
    ScanImgs(ScanImgs),
}

#[derive(Args, Debug)]
struct ScanImgs {
    #[clap(short, long)]
    worker_num: usize,
    #[clap(short, long)]
    gen_thumb: bool,
}

fn main() {
    let args = MainArgs::parse();

}
