use std::fs::File;
use std::io;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossbeam_channel::{unbounded, RecvTimeoutError};

pub fn get_buf_reader(file: &str) -> BufReader<Box<dyn std::io::Read>> {
    let read: Box<dyn std::io::Read> = match file {
        "-" => Box::new(io::stdin()),
        _ => Box::new(File::open(file).expect(&(format!("Error opening {} file", file)))),
    };
    BufReader::new(read)
}

pub fn get_buf_reader_safe(file: &str) -> Result<BufReader<Box<dyn std::io::Read>>> {
    let reader: Box<dyn std::io::Read> = match file {
        "-" => Box::new(io::stdin()),
        _ => {
            if std::path::Path::new(file).is_dir() {
                return Err(anyhow!("{} is a directory, not a file", file));
            }
            Box::new(File::open(file)?)
        }
    };
    Ok(BufReader::new(reader))
}

fn lossy_read_line(r: &mut dyn std::io::BufRead, buf: &mut String) -> std::io::Result<usize> {
    let mut byte_buf = vec![];
    let num_bytes = r.read_until(b'\n', &mut byte_buf)?;
    *buf = String::from_utf8_lossy(&byte_buf).into_owned();
    Ok(num_bytes)
}

pub fn write_lines<F>(
    r: &mut dyn std::io::BufRead,
    w: &mut (dyn std::io::Write + Send),
    f: F,
) -> Result<(), io::Error>
where
    F: Fn(&mut dyn std::io::Write, i32, &String) -> Result<bool, io::Error>,
{
    let mut s = String::new();

    crossbeam::scope(|scope| -> Result<(), io::Error> {
        let (tx, rx) = unbounded();
        let writer: Arc<Mutex<&mut (dyn std::io::Write + Send)>> = Arc::new(Mutex::new(w));

        let sub_writer = writer.clone();

        let handle = scope.spawn(move |_| {
            match move || -> Result<(), io::Error> {
                let mut start = Instant::now();
                let flush_interval = Duration::from_millis(100);
                let flush_timeout_th = Duration::from_millis(10);
                loop {
                    match rx.recv_timeout(flush_timeout_th) {
                        Err(RecvTimeoutError::Timeout) => {
                            let mut sub_writer = sub_writer.lock().unwrap();
                            sub_writer.flush()?;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            break;
                        }
                        Ok(_) => { /* nothing to do */ }
                    }
                    let now = start.elapsed();
                    if now >= flush_interval {
                        start = Instant::now();
                        let mut sub_writer = sub_writer.lock().unwrap();
                        sub_writer.flush()?;
                    }
                }
                drop(rx);
                Ok(())
            }() {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => { /* ignore error */ }
                Err(err) => panic!("{}", err),
            }
        });

        let main_writer = writer.clone();

        let mut ret_val: Result<(), io::Error> = Ok(());
        let mut nr = 1;
        loop {
            // NOTE: read_line cause error when input text is invalid UTF-8
            match lossy_read_line(r, &mut s) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let mut w = &mut *(main_writer.lock().unwrap());
                    let line_func_ret = f(&mut w, nr, &s);
                    s.clear();
                    match line_func_ret {
                        Ok(false) => {
                            break;
                        }
                        Ok(true) => {}
                        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => {}
                        Err(err) => {
                            ret_val = Err(err);
                            break;
                        }
                    }
                    match tx.send(true) {
                        Ok(_) => {}
                        Err(err) => {
                            #[cfg(debug_assertions)]
                            eprintln!("{:?}", err);
                            break;
                        }
                    }
                }
                Err(err) => {
                    ret_val = Err(err);
                    break;
                }
            }
            nr += 1;
        }
        drop(tx);
        handle.join().unwrap();
        ret_val
    })
    .unwrap()
}
