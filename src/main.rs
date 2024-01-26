mod rw_file;
mod splitter;

use clap::Parser;
use env_logger;
use log::error;
use std::fs::metadata;
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Task manager
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Task list file path
    #[arg(short, long)]
    backlog: String,

    // Number of parallel jobs
    #[arg(short, long, default_value_t = 1)]
    jobs: usize,
}

static MUTEX: Mutex<i32> = Mutex::new(0);
fn exec(task: String) {
    let mut params = splitter::split(task.to_string());
    let executable = params[0].to_string();
    params.remove(0);

    match Command::new(executable).args(params).output() {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let _ = MUTEX.lock();
            println!("-> {}", task);
            println!("{}", stdout);
            // output.status
        }
        Err(..) => {
            error!("Failed to execute command: {}", task);
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let args = Args::parse();

    let mut thread_pool: Vec<thread::JoinHandle<()>> = Vec::new();
    let mut has_tasks = true;

    while has_tasks || thread_pool.len() > 0 {
        match metadata(&args.backlog) {
            Ok(_) => (),
            Err(_) => {
                error!("File path \"{}\" does not exist", args.backlog);
                return Ok(());
            }
        }

        let mut tasks = rw_file::read(&args.backlog);

        // No more tasks need to be executed in the backlog
        if tasks.len() != 0 {
            let task = tasks[0].to_string();
            tasks.remove(0);
            rw_file::write(&args.backlog, &tasks);

            thread_pool.push(thread::spawn(|| exec(task)));
        } else {
            has_tasks = false;
        }

        // Visitor
        let mut be_visited = true;
        while be_visited {
            let mut index = 0;
            while index < thread_pool.len() {
                if thread_pool[index].is_finished() {
                    thread_pool.remove(index);
                    break;
                }
                index += 1;
            }

            if thread_pool.len() < args.jobs {
                be_visited = false;
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    return Ok(());
}
