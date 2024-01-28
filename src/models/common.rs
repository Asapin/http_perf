use std::{collections::HashMap, fmt::Display, iter::Map};

use tokio::time::Instant;

use super::request::Request;

pub struct Header {
    pub name: String,
    pub value: Vec<u8>,
}

impl Header {
    pub fn new(name: String, value: Vec<u8>) -> Self {
        Self { name, value }
    }
}

pub enum Verb {
    Get,
    Head,
    Post(Option<Vec<u8>>),
    Put(Option<Vec<u8>>),
    Delete,
    Patch(Option<Vec<u8>>),
}

pub enum NextAction {
    SendRequest(Request),
    Stop,
}

pub struct Config {
    pub number_of_requests: usize,
    pub total_number_of_connections: usize,
}

pub struct BenchResult {
    pub response_times: Vec<u128>,
    pub received_data: u64,
    pub response_statuses: HashMap<u16, usize>,
    pub errors: usize,
}

impl BenchResult {
    pub fn new(expected_size: usize) -> Self {
        Self {
            response_times: Vec::with_capacity(expected_size),
            received_data: 0,
            response_statuses: HashMap::new(),
            errors: 0,
        }
    }

    pub fn count_response_status(&mut self, status: u16) {
        self.response_statuses
            .entry(status)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn count_data(&mut self, data: u64) {
        self.received_data += data;
    }

    pub fn count_errors(&mut self) {
        self.errors += 1;
    }

    pub fn register_time(&mut self, start_time: Instant) {
        let micros = start_time.elapsed().as_micros();
        self.response_times.push(micros);
    }

    pub fn merge(&self, other: &Self) -> Self {
        let received_data = self.received_data + other.received_data;
        let errors = self.errors + other.errors;
        let mut response_times = self.response_times.clone();
        response_times.extend(other.response_times.clone().iter());
        let mut response_statuses = self.response_statuses.clone();
        for (status, count) in other.response_statuses.iter() {
            response_statuses
                .entry(*status)
                .and_modify(|i| *i += *count)
                .or_insert(*count);
        }

        Self {
            response_times,
            received_data,
            response_statuses,
            errors,
        }
    }

    pub fn print_stats(&self) {
        let mut min_time = u128::MAX;
        let mut max_time = 0;
        let mut avg_time = 0.0;
        for (idx, time) in self.response_times.iter().enumerate() {
            if *time > max_time {
                max_time = *time;
            }
            if *time < min_time {
                min_time = *time;
            }
            if idx == 0 {
                avg_time = *time as f64;
            } else {
                let idx = idx as f64;
                avg_time = avg_time * idx / (idx + 1.0) + *time as f64 / (idx + 1.0);
            }
        }
        println!("Min time: {} μs", min_time);
        println!("Avg time: {} μs", avg_time);
        println!("Max time: {} μs", max_time);
        println!("HTTP Statuses:");
        for (status, count) in self.response_statuses.iter() {
            println!("    {} -> {}", status, count);
        }
        println!(
            "Transfered data: {}",
            DataSize::calculate(self.received_data)
        );
        println!("Errors: {}", self.errors);
    }
}

pub enum DataSize {
    Bytes(u64),
    KBytes(f64),
    MBytes(f64),
    GBytes(f64),
    TBytes(f64),
}

impl DataSize {
    pub fn calculate(byte_size: u64) -> DataSize {
        if byte_size <= 4096 {
            return DataSize::Bytes(byte_size);
        }

        let mut byte_size = byte_size as f64 / 1024.0;
        if byte_size <= 1024.0 {
            return DataSize::KBytes(byte_size);
        }

        byte_size /= 1024.0;
        if byte_size <= 1024.0 {
            return DataSize::MBytes(byte_size);
        }

        byte_size /= 1024.0;
        if byte_size <= 1024.0 {
            return DataSize::GBytes(byte_size);
        }

        return DataSize::TBytes(byte_size);
    }
}

impl Display for DataSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSize::Bytes(size) => write!(f, "{} Bytes", size),
            DataSize::KBytes(size) => write!(f, "{} Kb", size),
            DataSize::MBytes(size) => write!(f, "{} Mb", size),
            DataSize::GBytes(size) => write!(f, "{} Gb", size),
            DataSize::TBytes(size) => write!(f, "{} Tb", size),
        }
    }
}
