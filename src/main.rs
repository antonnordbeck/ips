use std::{io::Read, iter::successors};

use rayon::{iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator}, slice::ParallelSliceMut, str::ParallelString};

extern crate nalgebra as na;
fn main(){
    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();
    let time = std::time::Instant::now();

    let iterations = 1;

    for n in 0..iterations{
        sorted_parallel();
    }

    println!("Avrage time {}s.", time.elapsed().as_secs_f64()/(iterations as f64));
}
//1436965 expected collisions
fn sorted_parallel(){
    let time = std::time::Instant::now();
    let mut file = std::fs::File::open("positions_large.xyz").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let mut pos: Vec<na::Point3<f32>> = buf.as_str().as_parallel_string().par_lines().map(|line|{
        let mut iter = line.split_ascii_whitespace();
        na::Point3::new(
            iter.next().unwrap().parse().unwrap(),
            iter.next().unwrap().parse().unwrap(),
            iter.next().unwrap().parse().unwrap()
        )
    }).collect();
    println!("\tFile load time \t{}s.", time.elapsed().as_secs_f64());
    let time = std::time::Instant::now();
    pos.as_mut_slice().par_sort_unstable_by(|a,b| a.x.total_cmp(&b.x));
    println!("\tSort time \t{}s.", time.elapsed().as_secs_f64());
    let time = std::time::Instant::now();

    let n =pos.len();

    let block_size = n/1000;

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


fn sorted(){
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').from_path("positions_large.xyz").unwrap();
    let mut pos: Vec<na::Point3<f32>> = rdr.deserialize().map(|pos| pos.unwrap()).collect();
    pos.sort_unstable_by(|a,b| a.x.total_cmp(&b.x));

    let n =pos.len();
    println!("{} positions.", n);

    let mut num = 0;
    for i in 0..(n-1){
        for j in (i+1)..n{
            if pos[j].x-pos[i].x > 0.05{
                break;
            }
            if na::distance_squared(&pos[i],&pos[j]) <= 0.0025{
                num += 1;
            }
        }
    }

    println!("{} collisions found.", num);
}

fn rayon_threaded(){
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').from_path("positions_large.xyz").unwrap();
    let positions: Vec<na::Point3<f32>> = rdr.deserialize().map(|pos| pos.unwrap()).collect();

    let n =positions.len();

    let num: usize = (0..n).into_par_iter().fold(|| 0_usize, |sum: usize, i| {
        let mut sum = sum;
        for j in (i+1)..n{
            if na::distance_squared(&positions[i],&positions[j]) <= 0.0025{
                sum += 1;
            }
        }
        return sum;
    }).sum();

    println!("{} collisions found.", num);
}
fn single_threaded(){
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').from_path("positions_large.xyz").unwrap();
    let positions: Vec<na::Point3<f32>> = rdr.deserialize().map(|pos| pos.unwrap()).collect();
    let mut num = 0;

    for i in 0..positions.len(){
        for j in (i+1)..positions.len(){
            if na::distance_squared(&positions[i],&positions[j]) <= 0.0025{
                num += 1;
            }
        }
    }
    println!("{} collisions found.", num);
}