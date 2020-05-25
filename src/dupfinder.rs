use crate::helpers;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::thread;
use crossbeam::channel;

const THREADS: usize = 8;

#[derive(Default)]
pub struct DupFinder {
  hashes: HashMap<u64, bool>,
  hashed: HashMap<PathBuf, u64>,
}

impl DupFinder {
  pub fn find<P: AsRef<Path>>(&mut self, path: P) {
    let (paths_in, paths_out) = channel::bounded::<PathBuf>(THREADS);
    let (hashes_in, hashes_out) = channel::bounded::<(PathBuf, u64)>(THREADS);

    // producer
    thread::spawn({
      let path = path.as_ref().to_owned();
      move || helpers::traverse(path, &|path| {
        paths_in.send(path).unwrap();
      })
    });

    // workers
    for _ in 0..THREADS {
      let paths_out = paths_out.clone();
      let hashes_in = hashes_in.clone();
      thread::spawn(move || {
        while let Ok(path) = paths_out.recv() {
          if let Ok(mut file) = File::open(&path) {
            let hash = helpers::hash_file(&mut file);
            hashes_in.send((path, hash)).unwrap();
          }
        }
      });
    }

    // consumer
    drop(hashes_in);
    while let Ok((path, hash)) = hashes_out.recv() {
      self.insert(path, hash);
    }

    self.show_results();
  }

  fn insert(&mut self, path: PathBuf, hash: u64) {
    self.hashes.entry(hash)
      .and_modify(|is_dupe| {
        println!("\x1b[0;33m{}\x1b[0m", path.to_string_lossy());
        *is_dupe = true;
      })
      .or_insert_with(|| {
        println!("\x1b[0;34m{}\x1b[0m", path.to_string_lossy());
        false
      });

    self.hashed.insert(path, hash);
  }

  fn show_results(&self) {
    let mut dupes = self.hashes.iter()
      .filter(|(_, &is_dupe)| is_dupe)
      .map(|(hash, _)| (hash, Vec::<&PathBuf>::new()))
      .collect::<HashMap<_, _>>();

    for (path, hash) in &self.hashed {
      dupes.entry(&hash)
        .and_modify(|paths| paths.push(path));
    }

    for (hash, dupes) in &dupes {
      println!();
      println!("\x1b[0;32m{:016x}\x1b[0m", hash);
      for path in dupes {
        let name = path.file_name().unwrap().to_string_lossy();
        let path = path.parent().unwrap().to_string_lossy();
        println!("\x1b[0;33m{}\x1b[0m in \x1b[0;34m{}\x1b[0m", name, path);
      }
    }
  }
}
