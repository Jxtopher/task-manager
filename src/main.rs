mod rw_file;
mod splitter;

use chrono::Utc;
use clap::Parser;
use env_logger;
use log::error;
use std::fs::metadata;
use std::path::Path;
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

    /// run N jobs in parallel
    #[arg(short, long, default_value_t = 1)]
    jobs: usize,

    /// Demon mode
    #[arg(short, long, default_value_t = false)]
    demon: bool,
}

static MUTEX: Mutex<i32> = Mutex::new(0);
fn exec(task: String) {
    let mut params = splitter::split(task.to_string());
    let executable = params[0].to_string();
    let start_time = Utc::now();
    params.remove(0);

    match Command::new(executable).args(params).output() {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let _ = MUTEX.lock();
            println!(
                "-> {} {}",
                start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                task
            );
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
    let args: Args = Args::parse();

    let mut has_tasks: bool = true;
    let mut is_locked: bool;
    let mut thread_pool: Vec<thread::JoinHandle<()>> = Vec::new();

    while args.demon || has_tasks || thread_pool.len() > 0 {
        match metadata(&args.backlog) {
            Ok(_) => (),
            Err(_) => {
                error!("File path \"{}\" does not exist", args.backlog);
                return Ok(());
            }
        }

        // Verify that the backlog file is not being edited
        let path = Path::new(&args.backlog);
        let parent = path.parent().unwrap().to_str().unwrap();
        let swapfile_filename = path.file_name().unwrap().to_str().unwrap();

        let swapfile = format!("{parent}/.{swapfile_filename}.swp");
        match metadata(&swapfile) {
            Ok(_) => is_locked = true,
            Err(_) => is_locked = false,
        }

        // Read the backlog
        let mut tasks = rw_file::read(&args.backlog);

        if tasks.len() == 0 {
            // No more tasks need to be executed in the backlog
            has_tasks = false;
        } else if !is_locked && thread_pool.len() < args.jobs {
            let task = tasks[0].to_string();
            tasks.remove(0);
            rw_file::write(&args.backlog, &tasks);
            thread_pool.push(thread::spawn(|| exec(task)));
        }

        // Thread visitor
        let mut index = 0;
        while index < thread_pool.len() {
            if thread_pool[index].is_finished() {
                thread_pool.remove(index);
            } else {
                index += 1;
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
