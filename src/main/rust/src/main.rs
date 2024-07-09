use std::{collections::HashMap, fs::File, io::Read, path::Path, sync::{Arc, Mutex, RwLock}, thread::{self, JoinHandle}};

const FILE_PATH: &str = "/Users/andrea/Documents/Code/Learning/1brc/data/measurements.txt";

/*struct Measurement<'a> {
    station: &'a str,
    min: &'a f64,
    max: &'a f64,
    mean: &'a f64
}

impl <'a> Measurement<'a> {

    fn new(station: &'a str, min: &'a f64, max: &'a f64, mean: &'a f64) -> Measurement<'a> {
        Measurement {
            station,
            min,
            max,
            mean
        }
    }
}*/

fn main() {
    let mut file =
        File::open(Path::new(FILE_PATH)).expect("Unable to open file measurements.txt");

    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf);

    let lines: Vec<&str> = buf.split('\n').collect();

    let num_cpu = thread::available_parallelism().unwrap().get();

    let lines_chunks = lines.chunks(num_cpu);
    let mut running_threads = Vec::new();

    // launch processing threads
    /*for lines_chunk in lines_chucks {
        let t = thread::spawn(|| calculate_piece(lines_chunk));
        running_threads.push(t);
    }*/

    let results = Arc::new(RwLock::new(Vec::new()));

    // launch processing threads
    for lines_chunk in lines_chunks {
        let results = results.clone();

        let t = thread::spawn(move || {
            let result = calculate_piece(lines_chunk);
            let _ = results.write().unwrap().extend(result);
        });

        running_threads.push(t);
    }

    // wait threads
    for t in running_threads {
        t.join();
    }

    let mut ordered_list: Vec<(&str, f64, f64, f64)> = Vec::new();

    //  emits the results on stdout like this (i.e. sorted alphabetically by station name, and the result values per station
    //  in the format <min>/<mean>/<max>, rounded to one fractional digit
    //  {Abha=-23.0/18.0/59.2, Abidjan=-16.2/26.0/67.3}

    ordered_list.sort_unstable_by(|a, b| a.0.cmp(b.0));

    let mut output_string: String = '{'.into();

    for (idx, elem) in ordered_list.iter().enumerate() {
        let station_name = elem.0;
        let min = elem.1;
        let max = elem.2;
        let avg = elem.3;

        if idx != ordered_list.len() - 1 {
            output_string.push_str(format!("{station_name}={min}/{avg}/{max}, ").as_str())  // there are other elements in the list
        } else {
            output_string.push_str(format!("{station_name}={min}/{avg}/{max}").as_str())
        }
    }

    output_string.push('}');

    println!("{}", output_string);
}

fn calculate_piece<'a>(lines: &'a [&'a str]) -> Vec<(&'a str, f64, f64, f64)> {

    let mut buckets: HashMap<&str, Vec<f64>> = HashMap::new();

    for line in lines {
        match line.split_once(';') {
            Some((weather_station, temp_str)) => {
    
                let temp = temp_str.parse().expect("Cannot parse temperature");
    
                if buckets.contains_key(weather_station) {
                    buckets.get_mut(weather_station).unwrap().push(temp);
                } else {
                    let temp_vec = vec![temp];
                    buckets.insert(weather_station, temp_vec);
                }
            },
            None => continue
        }
    }

    let mut return_list = Vec::new();

    for (key, val) in buckets {
        let min = calculate_min(&val);
        let max = calculate_max(&val);
        let avg = calculate_avg(&val);

        return_list.push((key, min, max, avg));
    }

    return_list
}

fn calculate_min(lst: &[f64]) -> f64 {
    let mut min = lst.first().unwrap();

    for i in lst {
        if i < min {
            min = i;
        }
    }

    *min
}

fn calculate_max(lst: &[f64]) -> f64 {
    let mut max = lst.first().unwrap();

    for i in lst {
        if i > max {
            max = i;
        }
    }

    *max
}

fn calculate_avg(lst: &[f64]) -> f64 {
    let mut acc: f64 = 0.0;

    for i in lst {
        acc += i;
    }

    acc / (lst.len() as f64)
}