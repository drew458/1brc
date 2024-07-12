# 1brc

My optimization journey in Rust.  
Reference: https://github.com/gunnarmorling/1brc

**Rules:**
 - No external dependencies

## Progress

#### 10/07/2024

Multithreaded chucks processing without data copies, single thread file read

File size: 1 million rows  
Code version -> `0.1.3`  
**Baseline** -> `0.74s user 0.05s system 156% cpu 0.503`  
**Rust implementation** -> `0.65s user 0.82s system 202% cpu 0.727 total`  

#### 09/07/2024

Multithreaded chucks processing with data copies, single thread file read

File size: 1 million rows  
Code version -> `0.1.1`  
**Baseline** -> `0.74s user 0.05s system 156% cpu 0.503`   
**Rust implementation** -> `2.28s user 1.41s system 306% cpu 1.204 total`

#### 08/07/2024

Base implementation, single thread everything

File size: 1 million rows  
Code version -> `0.1.0`  
**Baseline** -> `0.74s user 0.05s system 156% cpu 0.503`   
**Rust implementation** ->  `0.14s user 0.01s system 27% cpu 0.548 total`
