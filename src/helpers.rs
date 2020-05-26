use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub fn human_readable(bytes: u64) -> String {
  let bytes = bytes as f64;
  let power = (bytes.log2() / 10.0).max(0.0);
  match power as usize {
    0 => format!("{} B", bytes),
    p => {
      let norm = bytes * 2.0_f64.powf(-10.0 * power.floor());
      let symbol = b" KMGTPEZY"[p] as char;
      let digits = 3 - norm.log10() as usize;
      format!("{:.2$} {}iB", norm, symbol, digits)
    }
  }
}

pub fn hash_file(file: &mut File) -> u64 {
  let mut hasher = DefaultHasher::new();
  let mut buffer = [0; 1<<17];
  loop {
    match file.read(&mut buffer[..]) {
      Ok(0) => break hasher.finish(),
      Ok(n) => hasher.write(&buffer[..n]),
      _ => panic!()
    }
  }
}

pub fn traverse<P: AsRef<Path>>(path: P, cb: &dyn Fn(PathBuf)) {
  if let Ok(dir) = path.as_ref().read_dir() {
    for entry in dir {
      let path = entry.unwrap().path();
      if path.is_dir() {
        traverse(&path, cb);
      }
      else if path.is_file() {
        cb(path);
      }
    }
  }
}
