use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

extern crate nalgebra as na;
fn main(){
    let time = std::time::Instant::now();

    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    let mut rdr = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').from_path("positions_large.xyz").unwrap();
    let positions: Vec<na::Point3<f32>> = rdr.deserialize().map(|pos| pos.unwrap()).collect();

    let N =positions.len();

    let num: usize = (0..N).into_par_iter().fold(|| 0_usize, |sum: usize, i| {
        let mut sum = sum;
        for j in (i+1)..N{
            if na::distance_squared(&positions[i],&positions[j]) <= 0.0025{
                sum += 1;
            }
        }
        return sum;
    }).sum();

    /*
    let mut num = 0;

    for i in 0..positions.len(){
        for j in (i+1)..positions.len(){
            if na::distance_squared(&positions[i],&positions[j]) <= 0.0025{
                num += 1;
            }
        }
    }
    */
    println!("Calculated collisions in {}s.", time.elapsed().as_secs_f64());
    println!("{} collisions found.", num);
}