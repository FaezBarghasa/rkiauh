use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn get_cpu_usage() -> f64 {
    if let Ok(file) = File::open("/proc/loadavg") {
        let reader = BufReader::new(file);
        if let Some(Ok(line)) = reader.lines().next() {
            if let Some(load) = line.split_whitespace().next() {
                if let Ok(val) = load.parse::<f64>() {
                    return val * 10.0;
                }
            }
        }
    }
    0.0
}

pub fn get_mem_usage() -> f64 {
    if let Ok(file) = File::open("/proc/meminfo") {
        let reader = BufReader::new(file);
        let mut total = 0.0;
        let mut free = 0.0;
        let mut buffers = 0.0;
        let mut cached = 0.0;

        for line in reader.lines() {
            if let Ok(l) = line {
                if l.starts_with("MemTotal:") {
                    total = l.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0.0);
                } else if l.starts_with("MemFree:") {
                    free = l.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0.0);
                } else if l.starts_with("Buffers:") {
                    buffers = l.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0.0);
                } else if l.starts_with("Cached:") {
                    cached = l.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0.0);
                }
            }
        }

        if total > 0.0 {
            let used = total - free - buffers - cached;
            return (used / total) * 100.0;
        }
    }
    0.0
}
