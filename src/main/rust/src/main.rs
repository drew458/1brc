use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;

const FILE_PATH: &str = "../../../data/measurements.txt";

struct Measurement {
    station: String,
    min: f64,
    max: f64,
    avg: f64
}

impl Measurement {

    fn new(station: String, min: f64, max: f64, mean: f64) -> Measurement {
        Measurement {
            station,
            min,
            max,
            avg: mean
        }
    }
}

fn main() {
    let mut file = File::open(Path::new(FILE_PATH)).expect("Unable to open file measurements.txt");

    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf);

    let lines: Vec<String> = buf.lines().map(|line| line.to_string()).collect();

    let num_cpu = thread::available_parallelism().unwrap().get();   // we need to take ownership of every string in the file

    let lines_chunks: Vec<_> = lines
        .chunks(lines.len() / num_cpu)
        .map(|chunk| chunk.to_vec())
        .collect();
    let mut running_threads = Vec::new();

    let results = Arc::new(RwLock::new(Vec::new()));

    for lines_chunk in lines_chunks {
        let results = results.clone();

        let t = thread::spawn(move || {
            let result = calculate_piece(&lines_chunk);
            let _ = results.write().unwrap().extend(result);
        });

        running_threads.push(t);
    }

    for t in running_threads {
        t.join().unwrap();
    }

    results.write().unwrap().sort_unstable_by(|a, b| a.station.cmp(&b.station));

    let mut output_string: String = '{'.into();
    let size = results.read().unwrap().len() - 1;

    for (idx, elem) in results.read().unwrap().iter().enumerate() {
        let station_name = &elem.station;
        let min = &elem.min;
        let max = &elem.max;
        let avg = &elem.avg;

        if idx != size {
            output_string.push_str(format!("{station_name}={min:.1}/{avg:.1}/{max:.1}, ").as_str());
        } else {
            output_string.push_str(format!("{station_name}= {min:.1}/{avg:.1}/{max:.1}").as_str());
        }
    }

    output_string.push('}');

    println!("{}", output_string);
}

fn calculate_piece(lines: &[String]) -> Vec<Measurement> {
    let mut buckets: HashMap<String, Vec<f64>> = HashMap::new();

    for line in lines {
        match line.split_once(';') {
            Some((weather_station, temp_str)) => {
                let temp = temp_str.parse().expect("Cannot parse temperature");

                buckets
                    .entry(weather_station.to_string())
                    .or_insert_with(Vec::new)
                    .push(temp);
            }
            None => continue,
        }
    }

    let mut return_list = Vec::new();

    for (key, val) in buckets {
        let min = calculate_min(&val);
        let max = calculate_max(&val);
        let avg = calculate_avg(&val);

        return_list.push(Measurement::new(key, min, max, avg));
    }

    return_list
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
