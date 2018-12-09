mod vrm;

use self::vrm::*;
use std::fs::create_dir;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str), help = "VRM file exported by VRoid Studio.")]
    path: PathBuf,
    #[structopt(
        short = "f",
        long = "force",
        help = "Overwrite file if already exists same file."
    )]
    force: bool,
}

fn main() {
    let opt = Opt::from_args();
    let path = opt.path;
    println!("{:?}", path);

    // vrm読み込み
    let vrm = Vrm::load(path.as_path())
        .unwrap_or_else(|e| panic!("Failed to parse file {:?}: {:?}", path, e));
    let save_dir = path.parent().unwrap_or(Path::new(".")).join("result");
    if !save_dir.exists() {
        create_dir(&save_dir)
            .unwrap_or_else(|e| panic!("failed to create dir '{:?}': {:?}", save_dir, e));
    }
    let save_path = save_dir.join(
        path.file_name()
            .unwrap_or_else(|| panic!("path '{:?}' doesn't have file component", path)),
    );
    if !opt.force && save_path.exists() {
        println!("Already exists file. Overwrite?(y/N): ");
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .expect("Failed to read answer from stdin");
        if !["y", "yes"].contains(&line.trim().to_lowercase().as_str()) {
            return;
        }
    }

    vrm.save(&save_path).expect("Failed to save");
    println!("saved.");
}
