use crate::helpers;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam::channel;

const THREADS: usize = 8;

#[derive(Default)]
pub struct DupFinder {
  dupes:  HashSet<u64>,
  hashes: HashSet<u64>,
  paths:  HashMap<PathBuf, u64>,
}

impl DupFinder {
  pub fn find<P: AsRef<Path>>(self, path: P) {
    let state = Arc::new(Mutex::new(self));
    let (paths_in, paths_out) = channel::bounded::<PathBuf>(THREADS);
    let (hashes_in, hashes_out) = channel::bounded::<(PathBuf, u64)>(THREADS);
    let mut threads = vec![];

    // consumer
    threads.push({
      let state = state.clone();
      thread::spawn(move || {
        while let Ok((path, hash)) = hashes_out.recv() {
          state.lock().unwrap().insert(path, hash);
        }
      })
    });

    // workers
    threads.extend((0..THREADS).map(|_| {
      let paths_out = paths_out.clone();
      let hashes_in = hashes_in.clone();
      thread::spawn(move || {
        while let Ok(path) = paths_out.recv() {
          if let Ok(mut file) = File::open(&path) {
            let hash = helpers::hash_file(&mut file);
            hashes_in.send((path, hash)).unwrap();
          }
        }
      })
    }));

    // producer
    helpers::traverse(path, &move |path| {
      paths_in.send(path).unwrap();
    });

    drop(paths_out);
    drop(hashes_in);
    for t in threads {
      t.join().unwrap();
    }

    state.lock().unwrap().show_stats();
  }

  fn insert(&mut self, path: PathBuf, hash: u64) {
    if self.hashes.contains(&hash) {
      self.dupes.insert(hash);
      println!("\x1b[0;33m{}\x1b[0m", path.to_string_lossy());
    }
    else {
      self.hashes.insert(hash);
      println!("\x1b[0;34m{}\x1b[0m", path.to_string_lossy());
    }
    self.paths.insert(path, hash);
  }

  fn show_stats(&self) {
    let mut dupes = self.dupes.iter()
      .map(|hash| (hash, Vec::<&PathBuf>::new()))
      .collect::<HashMap<_, _>>();

    for (path, hash) in &self.paths {
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