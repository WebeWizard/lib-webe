extern crate chrono;
use chrono::prelude::*;
use std::io::{self, BufWriter, Write};
use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
pub enum LogLevel {
  TRACE, DEBUG, ERROR, WARN, INFO
}

pub struct WebeLogger {
  sinks: Vec<Box<dyn Sink>>,
  monitor: JoinHandle<()>, // a separate thread used to push logging status messages to console
  mon_sender: mpsc::Sender<String>,
}

impl WebeLogger {
  pub fn new() -> WebeLogger {
    let (mon_send,mon_recv) = mpsc::channel();
    let monitor = thread::spawn(move || {
      loop {
        // block while waiting for new messages
        match mon_recv.recv() {
          Ok(status_msg) => {}
          Err(recv_err) => {}
        }
      }
    });
    WebeLogger { 
      sinks:  Vec::new(),
      monitor: monitor,
      mon_sender: mon_send, }
  }

  pub fn log(&mut self, level: &LogLevel, msg: &str) {
    /* TODO: filter sinks by what log levels they are listening to */
    for sink in &mut self.sinks {
      sink.write(level, msg);
    }
  }

  pub fn add_sink(&mut self, sink: Box<dyn 'static+Sink>) {
    self.sinks.push(sink);
  }
}

pub trait Sink {
  /* TODO - return an error if for some reason we can't write */
  fn write(&mut self, level: &LogLevel, msg: &str) {}
}


pub struct ConsoleLogger {
  queue: Arc<Mutex<Vec<String>>>,
  scheduler: JoinHandle<()>,
}

impl ConsoleLogger {
  pub fn new(mon_send: mpsc::Sender<String>) -> ConsoleLogger {
    /* TODO - use a Locked stdout handle for better performance - once the api is stablized.
    https://doc.rust-lang.org/std/io/struct.Stdout.html#method.into_locked */
    let stdout = io::stdout();
    let queue = Arc::new(Mutex::new(Vec::new()));
    let thread_queue = queue.clone();
    let scheduler = thread::spawn(move || {
      let std_handle = io::BufWriter::new(stdout);
      loop {
        match thread_queue.try_lock() {
          Ok(guard) => {
            for msg in guard.iter() {
              writeln!(std_handle, "{}", msg);
            }
          }
          Err(lock_err) => {
            mon_send.send(format!("{}",lock_err));
          }
        }
        for msg in thread_queue.try_lock().iter() {
        }
      }
    });
    ConsoleLogger { 
      queue: queue,
      scheduler: scheduler
    }
  }
}

impl Sink for ConsoleLogger {
  // Add message to the queue.  It'll get picked up by the next write timer.
  fn write(&mut self, level: &LogLevel, msg: &str) {
    let cur_timestamp = Local::now();
    match thread_
    self.handle, "[{}] - [{:?}] {}", cur_timestamp, level, msg);
  }
}