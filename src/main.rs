use std::{io::Read, iter::successors};

use na::min;
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge, ParallelIterator}, slice::ParallelSliceMut, str::ParallelString};

extern crate nalgebra as na;
fn main(){
    let args: Vec<String> = std::env::args().collect();

    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    /* Load data from file. */

    println!("Loading positions from {}.", args[1].as_str());
    let time = std::time::Instant::now();
    let mut file = std::fs::File::open(args[1].as_str()).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let n = buf.len();
    let mut pos: Vec<na::Point3<f32>> = std::thread::scope(|s|{
        let mut a = 0;
        let mut threads: Vec<Option<std::thread::ScopedJoinHandle<Vec<na::Point3<f32>>>>> = Vec::new();
        while a < n{
            let mut b = min(a+ n/16, n);
            while b < n && buf[b] != b'\n' {
                b += 1;
            }
    
            let slice = &buf[a..b];
            threads.push(Some(s.spawn(move || {
                std::str::from_utf8(slice).unwrap().lines().map(|line| {
                    let mut iter = line.split_ascii_whitespace();
                    na::Point3::new(
                        iter.next().unwrap().parse().unwrap(),
                        iter.next().unwrap().parse().unwrap(),
                        iter.next().unwrap().parse().unwrap()
                    )
                }).collect()
            })));
            a=b + 1;
        }
        return threads.iter_mut().map(|thread| thread.take().unwrap()).map(|thread| {
            thread.join().unwrap()
        }).flatten().collect();
    });

    /*
    let mut buf =   String::new();
    file.read_to_string(&mut buf).unwrap();
    let mut pos: Vec<na::Point3<f32>> = buf.as_str().as_parallel_string().par_lines().map(|line|{
        let mut iter = line.split_ascii_whitespace();
        na::Point3::new(
            iter.next().unwrap().parse().unwrap(),
            iter.next().unwrap().parse().unwrap(),
            iter.next().unwrap().parse().unwrap()
        )
    }).collect();
    */

    println!("\tFile load time \t{}s.", time.elapsed().as_secs_f64());

    /* Sort points by their x position */
    let time = std::time::Instant::now();
    pos.as_mut_slice().par_sort_unstable_by(|a,b| a.x.total_cmp(&b.x));
    println!("\tSort time \t{}s.", time.elapsed().as_secs_f64());

    /* Find all point near each other */
    let time = std::time::Instant::now();
    let n =pos.len();
    let block_size: usize = n/1000;
    let num: usize = successors(Some(0), |b| if b < &n { Some(b+block_size)} else {None}).par_bridge().map(|b|{
        let mut num = 0;
        for i in b..(b+block_size).min(n-1){
            for j in (i+1)..n{
                if pos[j].x-pos[i].x > 0.05{
                    break;
                }
                if na::distance_squared(&pos[i],&pos[j]) <= 0.0025{
                    num += 1;
                }
            }
        }
        return num;
    }).sum();
    
    println!("\tCollision time \t{}s.", time.elapsed().as_secs_f64());
    println!("{} collisions found.", num);
}
//1436965 expected collisions
