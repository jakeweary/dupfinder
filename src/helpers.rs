use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::Hasher;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

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
