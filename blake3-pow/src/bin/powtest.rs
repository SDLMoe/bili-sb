use blake3_pow::*;

const N: usize = 128;
const COST: u32 = 22;

#[cfg(not(target_os = "wasi"))]
fn main() {
  use std::{
    sync::{
      atomic::{AtomicUsize, Ordering},
      Arc,
    },
    time::Duration,
  };

  let counter = Arc::new(AtomicUsize::new(0));
  let mut vec: Vec<std::thread::JoinHandle<()>> = Vec::with_capacity(17);

  for _ in 0..16 {
    let counter = Arc::clone(&counter);
    let handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
      let mut salt = [0; N];
      loop {
        if round(&mut salt, COST) {
          counter.fetch_add(1, Ordering::AcqRel);
        }
      }
    });
    vec.push(handle);
  }
  let counter = Arc::clone(&counter);
  vec.push(std::thread::spawn(move || {
    let mut total = 0;
    loop {
      std::thread::sleep(Duration::from_secs(1));
      let last_sec = counter.swap(0, Ordering::AcqRel);
      total += last_sec;
      println!("{last_sec}/s ({total})");
    }
  }));
  for ele in vec {
    ele.join().unwrap()
  }
}

#[cfg(target_os = "wasi")]
fn main() {
  use std::time::Instant;
  let mut salt = [0; N];
  loop {
    let instant = Instant::now();
    round(&mut salt, COST);
    println!("{:?}", instant.elapsed());
  }
}

fn round<const N: usize>(salt: &mut [u8; N], cost: u32) -> bool {
  use rand::{thread_rng, RngCore};

  thread_rng().fill_bytes(salt);
  let now_ts = epoch_sec();
  let key = search(salt, cost, now_ts, usize::MAX).unwrap();
  verify(salt, cost, now_ts, 60, key)
}
