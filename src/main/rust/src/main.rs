use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, scope};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_PATH: &str = "../../../data/measurements.txt";

struct Measurement {
    station: String,
    min: f64,
    max: f64,
    avg: f64,
}

impl Measurement {
    fn new(station: String, min: f64, max: f64, mean: f64) -> Measurement {
        Measurement {
            station,
            min,
            max,
            avg: mean,
        }
    }
}

impl Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={:.1}/{:.1}/{:.1}", self.station, self.min, self.avg, self.max)
    }
}

fn main() {
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(); 
    let num_cpu = thread::available_parallelism().unwrap().get();
    
    let mut file = File::open(Path::new(FILE_PATH)).expect("Unable to open file measurements.txt");

    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf);

    let lines: Vec<&str> = buf.lines().collect();

    let lines_chunks = lines.chunks(lines.len() / num_cpu);

    let buckets = Mutex::new(HashMap::new());

    thread::scope(|scope| {
        for chunk in lines_chunks {
            
            scope.spawn(|| {
                calculate_piece(chunk, &buckets);
            });
        }
    });

    let mut results = Vec::new();

    for (key, val) in buckets.lock().unwrap().iter() {
        let min = calculate_min(&val);
        let max = calculate_max(&val);
        let avg = calculate_avg(&val);

        results.push(Measurement::new(key.to_string(), min, max, avg));
    }

    results.sort_unstable_by(|a, b| a.station.cmp(&b.station));

    let mut output_string: String = '{'.into();
    let size = results.len() - 1;

    for (idx, elem) in results.iter().enumerate() {
        if idx != size {
            output_string
                .push_str(format!("{elem}, ").as_str());
        } else {
            output_string
                .push_str(format!("{elem}").as_str());
        }
    }

    output_string.push('}');

    println!("{}", output_string);

    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    println!("Took {} ms", end - start);
    
}

fn calculate_piece(lines: &[&str], buckets: &Mutex<HashMap<String, Vec<f64>>>) {
    for line in lines {
        match line.split_once(';') {
            Some((weather_station, temp_str)) => {
                let temp = temp_str.parse().expect("Cannot parse temperature");

                {
                    let mut write_guard = buckets.lock().unwrap();

                    match write_guard.get_mut(weather_station) {
                        Some(tmp_vec) => {
                            tmp_vec.push(temp);
                        }
                        None => {
                            let mut tmp_vec = Vec::new();
                            tmp_vec.push(temp);
                            write_guard.insert(weather_station.to_string(), tmp_vec);
                        }
                    }
                }
            }
            None => continue,
        }
    }
}

fn calculate_min(lst: &[f64]) -> f64 {
    *lst.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn calculate_max(lst: &[f64]) -> f64 {
    *lst.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn calculate_avg(lst: &[f64]) -> f64 {
    let sum: f64 = lst.iter().sum();
    sum / (lst.len() as f64)
}
