mod helpers;
mod dupfinder;

use dupfinder::DupFinder;

fn main() {
  let mut args = std::env::args().skip(1);
  let path = args.next().unwrap_or_else(|| ".".into());

  DupFinder::default().find(path);
}
