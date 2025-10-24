use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread::{self};

use crossbeam::channel::{Receiver, Sender};

const FILE_PATH: &str = "measurements.txt";

struct Measurement {
    station: String,
    min: f64,
    max: f64,
    avg: f64,
    count: f64,
}

impl Measurement {
    fn new(station: String, min: f64, max: f64, mean: f64, count: f64) -> Measurement {
        Measurement {
            station,
            min,
            max,
            avg: mean,
            count,
        }
    }
}

impl Display for Measurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={:.1}/{:.1}/{:.1}",
            self.station, self.min, self.avg, self.max
        )
    }
}

fn main() {
    let num_cpu = thread::available_parallelism().unwrap().get();

    let (tx, rx): (Sender<Vec<String>>, Receiver<Vec<String>>) =
        crossbeam::channel::bounded(100000);
    let mut handles = vec![];

    // Start the threads that process the chucks
    for _ in 0..num_cpu {
        let rx = rx.clone();

        let handle = thread::spawn(move || {
            let mut buckets: HashMap<String, Vec<f64>> = HashMap::new();

            while let Ok(chunk) = rx.recv() {
                for line in chunk {
                    match process_line(&line) {
                        Some((weather_station, temperature)) => {
                            match buckets.get_mut(&weather_station) {
                                Some(tmp_vec) => {
                                    tmp_vec.push(temperature);
                                }
                                None => {
                                    let tmp_vec = vec![temperature];
                                    buckets.insert(weather_station, tmp_vec);
                                }
                            }
                        }
                        None => continue,
                    }
                }
            }

            let mut results = HashMap::new();

            for (key, val) in buckets.iter() {
                let min = calculate_min(val);
                let max = calculate_max(val);
                let avg = calculate_avg(val);

                results.insert(
                    key.to_owned().to_owned(),
                    Measurement::new(key.to_string(), min, max, avg, val.len() as f64),
                );
            }

            results
        });

        handles.push(handle);
    }

    // Read the file
    let mut file = File::open(Path::new(FILE_PATH)).expect("Unable to open file measurements.txt");
    let mut buf = String::with_capacity(100 * 1_000_000_000);   // The file is composed of 1 Billion rows of maximum 100 byte per row
    let _ = file.read_to_string(&mut buf);
    let mut lines = buf.lines();

    // Produce the chucks
    let mut chunk: Vec<String> = vec![];
    while let Some(line) = lines.next() {
        if chunk.len() > 100000 {
            let _ = tx.send(chunk);
            chunk = vec![];
        } else {
            chunk.push(line.to_owned());
        }
    }
    drop(tx);

    let mut local_buckets: Vec<HashMap<String, Measurement>> = vec![];

    for handle in handles {
        local_buckets.push(handle.join().unwrap());
    }

    let mut global_measurement: HashMap<String, Measurement> = HashMap::new();

    for b in local_buckets {
        for (k, local_measurement) in b.iter() {
            let mut global_min;
            let mut global_max;
            let new_avg;
            let mut global_count;

            match global_measurement.get(k) {
                Some(global_measurement) => {
                    global_min = global_measurement.min;
                    global_max = global_measurement.max;
                    global_count = global_measurement.count;

                    if local_measurement.min < global_min {
                        global_min = local_measurement.min;
                    }

                    new_avg = ((global_measurement.avg * global_count) + local_measurement.avg)
                        / (global_count + 1.0); // Incremental average formula

                    global_count = global_measurement.count + 1.0;

                    if local_measurement.max > global_max {
                        global_max = local_measurement.max;
                    }
                }
                None => {
                    global_min = local_measurement.min;
                    new_avg = local_measurement.avg;
                    global_count = 1.0;
                    global_max = local_measurement.max;
                }
            }

            global_measurement.insert(
                k.to_owned(),
                Measurement {
                    station: k.to_owned(),
                    min: global_min,
                    max: global_max,
                    avg: new_avg,
                    count: global_count,
                },
            );
        }
    }

    let mut results: Vec<&Measurement> = global_measurement.values().collect();

    results.sort_unstable_by(|a, b| a.station.cmp(&b.station));

    let mut output_string: String = '{'.into();
    let size = results.len() - 1;

    for (idx, elem) in results.iter().enumerate() {
        if idx != size {
            output_string.push_str(format!("{elem}, ").as_str());
        } else {
            output_string.push_str(format!("{elem}").as_str());
        }
    }

    output_string.push('}');

    println!("{}", output_string);
}

fn process_line(line: &str) -> Option<(String, f64)> {
    match line.split_once(';') {
        Some((weather_station, temp_str)) => {
            let temp = temp_str.parse().expect("Cannot parse temperature");

            Some((weather_station.to_owned(), temp))
        }
        None => None,
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
