# 1brc

My optimization journey in Go/Rust.  
Reference: https://github.com/gunnarmorling/1brc

The text file contains temperature values for a range of weather stations.
Each row is one measurement in the format `<string: station name>;<double: measurement>`, with the measurement value having exactly one fractional digit.
The following shows ten rows as an example:

```
Hamburg;12.0
Bulawayo;8.9
Palembang;38.8
St. John's;15.2
Cracow;12.6
Bridgetown;26.9
Istanbul;6.2
Roseau;34.4
Conakry;31.2
Istanbul;23.0
```

The task is to write a Java program which reads the file, calculates the min, mean, and max temperature value per weather station, and emits the results on stdout like this
(i.e. sorted alphabetically by station name, and the result values per station in the format `<min>/<mean>/<max>`, rounded to one fractional digit):

```
{Abha=-23.0/18.0/59.2, Abidjan=-16.2/26.0/67.3, Abéché=-10.0/29.4/69.0, Accra=-10.1/26.4/66.4, Addis Ababa=-23.7/16.0/67.0, Adelaide=-27.8/17.3/58.5, ...}
```

**Rules:**
 - No external dependencies

## Progress

### 21/10/2025

Go: Multithreaded chucks processing, single thread file read that passes chucks of lines through channels
Rust: Multithreaded chucks processing without data copies, single thread file read

File size: 1 billion rows

**Baseline** -> `77.25s user 3.99s system 101% cpu 1:20.07 total`  
**Rust** ->`129.28s user 1393.87s system 643% cpu 3:56.70 total`  
**Go** -> `111.80s user 4.62s system 409% cpu 28.461 total`


### 15/10/2025

Go: basic implementation  
Rust: Multithreaded chucks processing without data copies, single thread file read

File size: 1 billion rows

**Baseline** -> `77.25s user 3.99s system 101% cpu 1:20.07 total`  
**Rust** ->`132.20s user 1440.23s system 749% cpu 3:29.66 total`  
**Go** -> `78.95s user 5.09s system 98% cpu 1:24.99 total`

#### 10/07/2024

Multithreaded chucks processing without data copies, single thread file read

File size: 1 million rows  
Code version -> `0.1.3`  
**Baseline** -> `0.74s user 0.05s system 156% cpu 0.503`  
**Rust** -> `0.65s user 0.82s system 202% cpu 0.727 total`  

#### 09/07/2024

Multithreaded chucks processing with data copies, single thread file read

File size: 1 million rows  
Code version -> `0.1.1`  
**Baseline** -> `0.74s user 0.05s system 156% cpu 0.503`   
**Rust** -> `2.28s user 1.41s system 306% cpu 1.204 total`

#### 08/07/2024

Base implementation, single thread everything

File size: 1 million rows  
Code version -> `0.1.0`  
**Baseline** -> `0.74s user 0.05s system 156% cpu 0.503`   
**Rust** ->  `0.14s user 0.01s system 27% cpu 0.548 total`


## Running the Challenge

This repository contains two programs:

* `dev.morling.onebrc.CreateMeasurements` (invoked via _create\_measurements.sh_): Creates the file _measurements.txt_ in the root directory of this project with a configurable number of random measurement values
* `dev.morling.onebrc.CalculateAverage` (invoked via _calculate\_average\_baseline.sh_): Calculates the average values for the file _measurements.txt_

Execute the following steps to run the challenge:

1. Build the project using Apache Maven:

    ```
    ./mvnw clean verify
    ```

2. Create the measurements file with 1B rows (just once):

    ```
    ./create_measurements.sh 1000000000
    ```

    This will take a few minutes.
    **Attention:** the generated file has a size of approx. **12 GB**, so make sure to have enough diskspace.

    If you're running the challenge with a non-Java language, there's a non-authoritative Python script to generate the measurements file at `src/main/python/create_measurements.py`. The authoritative method for generating the measurements is the Java program `dev.morling.onebrc.CreateMeasurements`.

3. Calculate the average measurement values:

    ```
    ./calculate_average_baseline.sh
    ```

    The provided naive example implementation uses the Java streams API for processing the file and completes the task in ~2 min on environment used for [result evaluation](#evaluating-results).
    It serves as the base line for comparing your own implementation.

4. Optimize the heck out of it:

    Adjust the `CalculateAverage` program to speed it up, in any way you see fit (just sticking to a few rules described below).
    Options include parallelizing the computation, using the (incubating) Vector API, memory-mapping different sections of the file concurrently, using AppCDS, GraalVM, CRaC, etc. for speeding up the application start-up, choosing and tuning the garbage collector, and much more.

## Rules and limits

* No external library dependencies may be used
* Implementations must be provided as a single source file
* The computation must happen at application _runtime_, i.e. you cannot process the measurements file at _build time_
(for instance, when using GraalVM) and just bake the result into the binary
* Input value ranges are as follows:
    * Station name: non null UTF-8 string of min length 1 character and max length 100 bytes, containing neither `;` nor `\n` characters. (i.e. this could be 100 one-byte characters, or 50 two-byte characters, etc.)
    * Temperature value: non null double between -99.9 (inclusive) and 99.9 (inclusive), always with one fractional digit
* There is a maximum of 10,000 unique station names
* Line endings in the file are `\n` characters on all platforms
* Implementations must not rely on specifics of a given data set, e.g. any valid station name as per the constraints above and any data distribution (number of measurements per station) must be supported
* The rounding of output values must be done using the semantics of IEEE 754 rounding-direction "roundTowardPositive"