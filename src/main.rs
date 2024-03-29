mod rw_file;
mod splitter;

use chrono::Utc;
use clap::Parser;
use env_logger;
use log::error;
use std::fs;
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
            let mutex = MUTEX.lock();
            println!(
                "-> {} {}",
                start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                task
            );
            println!("{}", stdout);
            // output.status
            drop(mutex);
        }
        Err(..) => {
            error!("Failed to execute command: {}", task);
        }
    }
}

fn thread_visitor(thread_pool: &mut Vec<thread::JoinHandle<()>>) {
    let mut index = 0;
    while index < thread_pool.len() {
        if thread_pool[index].is_finished() {
            thread_pool.remove(index);
        } else {
            index += 1;
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let args: Args = Args::parse();

    let mut has_tasks: bool = true;
    let mut is_locked: bool;
    let mut thread_pool: Vec<thread::JoinHandle<()>> = Vec::new();

    let backlog = fs::metadata(&args.backlog);
    let backlog_path: String = args.backlog.clone();
    let mut backlog_file: String = args.backlog.clone();
    match backlog {
        Ok(_) => (),
        Err(_) => {
            error!("File or directory path \"{}\" does not exist", args.backlog);
            return Ok(());
        }
    }

    let is_dir = backlog.unwrap().is_dir();
    if is_dir {
        backlog_file.clear();
    }

    while args.demon || has_tasks || thread_pool.len() > 0 {
        if is_dir && backlog_file.is_empty() {
            let paths = fs::read_dir(&backlog_path);
            let extension: String = String::from("bl");
            for entry in paths.unwrap() {
                let path = entry?.path();
                if path.extension() != None
                    && path.extension().unwrap().to_string_lossy() == extension
                {
                    let start_time = Utc::now();
                    let mutex = MUTEX.lock();
                    println!(
                        "-> {} backlog: {}",
                        start_time.format("%Y-%m-%d %H:%M:%S%.3f%z"),
                        path.display()
                    );
                    drop(mutex);
                    backlog_file = path.display().to_string();
                    break;
                }
            }
        }

        if backlog_file.is_empty() {
            if !args.demon {
                has_tasks = false;
            }
            thread_visitor(&mut thread_pool);
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        // Read the backlog
        let mut tasks = rw_file::read(&backlog_file);

        // Verify that the backlog file is not being edited
        let path = Path::new(&backlog_file);
        let parent = path.parent().unwrap().to_str().unwrap();
        let swapfile_filename = path.file_name().unwrap().to_str().unwrap();

        match fs::metadata(format!("{parent}/.{swapfile_filename}.swp")) {
            Ok(_) => is_locked = true,
            Err(_) => is_locked = false,
        }

        if tasks.len() == 0 {
            // No more tasks need to be executed in the backlog
            if is_dir {
                match fs::remove_file(&backlog_file) {
                    Ok(_) => (),
                    Err(_) => (),
                }
                backlog_file.clear();
            } else {
                has_tasks = false;
            }
        } else if !is_locked && thread_pool.len() < args.jobs {
            while thread_pool.len() < args.jobs {
                let task = tasks[0].to_string();
                tasks.remove(0);
                thread_pool.push(thread::spawn(|| exec(task)));
            }
            rw_file::write(&backlog_file, &tasks);
        }

        thread_visitor(&mut thread_pool);

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
