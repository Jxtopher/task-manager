use log::error;
use std::fs::{read_to_string, File};
use std::io::Write;

pub fn read(filename: &String) -> Vec<String> {
    let mut result = Vec::new();
    for line in read_to_string(filename).unwrap().lines() {
        result.push(line.to_string())
    }
    result
}

pub fn write(filename: &String, tasks: &Vec<String>) {
    if tasks.len() == 0 {
        // Write an empty file
        let mut data_file = File::create(filename).expect("creation failed");
        data_file.write("".as_bytes()).expect("write failed");
        return;
    } else {
        let mut data_file = File::create(filename).expect("creation failed");
        for i in 0..=(tasks.len() - 1) {
            let mut line = tasks[i].to_string();
            if i != tasks.len() - 1 {
                line.push_str("\n");
            }
            match data_file.write(line.as_bytes()) {
                Ok(_) => (),
                Err(..) => {
                    error!("Write failed");
                }
            }
        }
    }
}
