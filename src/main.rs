use std::io::Read;

use na::min;
use rayon::{iter::{ParallelBridge, ParallelIterator}, slice::ParallelSliceMut};

extern crate nalgebra as na;

static THREAD_N: usize = 8;

fn parse(buf: &Vec<u8>, n:usize , a:usize)->Vec<na::Point3<f32>>{
    if a < n{
        let mut b = min(a+ n/THREAD_N, n);
        while b < n && buf[b] != b'\n' {
            b += 1;
        }
        let slice = &buf[a..b];
        let (mut lo, mut hi) = rayon::join(|| {
            let mut pos = Vec::with_capacity(n/59/THREAD_N);//About 59 bytes per point
            for line in  std::str::from_utf8(slice).unwrap().lines(){
                let mut iter: std::str::SplitAsciiWhitespace<'_> = line.split_ascii_whitespace();
                pos.push(na::Point3::new(
                    iter.next().unwrap().parse().unwrap(),
                    iter.next().unwrap().parse().unwrap(),
                    iter.next().unwrap().parse().unwrap()
                ))
            }
            pos
        }, || {
            parse(buf, n, b+1)
        });
        lo.append(&mut hi);
        lo
    }
    else{
        return Vec::with_capacity(n/59);
    }
}
fn main(){
    let args: Vec<String> = std::env::args().collect();

    rayon::ThreadPoolBuilder::new().num_threads(16).build_global().unwrap();

    /* Load data from file.
     * Uses pararell parsing to parse the file, it is sligthly slower for smaller data sets but scales better with big datasets.
     */
    let path = if args.len() > 1 { args[1].as_str() } else {"positions_large.xyz"};

    println!("Loading positions from {}.", path);
    let time = std::time::Instant::now();
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let mut pos: Vec<na::Point3<f32>> = parse(&buf, buf.len(), 0);

    println!("\tFile load time \t{}s.", time.elapsed().as_secs_f64());

    /* Sort points by their x position using parallel sort*/
    let time = std::time::Instant::now();
    pos.as_mut_slice().par_sort_unstable_by(|a,b| a.x.total_cmp(&b.x));
    println!("\tSort time \t{}s.", time.elapsed().as_secs_f64());

    /* Find all point near each other.
     * Uses the fact that the point are sorted by x and that the distance between the points a and b
     * can never be less than |a_x - b_x|. This means we only need to check point near each other
     * in the vector. 
     * For each point we only need to check points on one side.
     * We seperare the points in to blocks an prosess each in parallel.
     * 
     * In the worst case where all points are "near" each other the algorithm will do an exhaustive search in O(n^2) time, n^2/2 to be exact.
     * but the more the points are spread out the nearer linear time the algorithm will become.
     */
    let time = std::time::Instant::now();
    //let num = collide(&pos, 0, pos.len());

    let n =pos.len();
    let block_size: usize = n/1000;
    let num: usize = std::iter::successors(Some(0), |b| if b < &n { Some(b+block_size)} else {None}).par_bridge().map(|b|{
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