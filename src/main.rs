use std::io::Read;

use rayon::{iter::{ParallelBridge, ParallelIterator}, slice::ParallelSliceMut};

extern crate nalgebra as na;

fn parse(buf: &[u8])->Vec<Vec<na::Point3<f32>>>{
    let n = buf.len();
    if n > 500000{
        let mut mid = n/2;
        while mid < n && buf[mid] != b'\n' {
            mid += 1;
        }

        let (mut lo, mut hi) = rayon::join(|| parse(&buf[..mid]), || parse(&buf[(mid+1)..]));
        lo.append(&mut hi);
        return  lo;
    }
    else{
        let mut pos = Vec::with_capacity(n/59);//About 59 bytes per point
        for line in  std::str::from_utf8(buf).unwrap().lines(){
            let mut iter: std::str::SplitAsciiWhitespace<'_> = line.split_ascii_whitespace();
            pos.push(na::Point3::new(
                iter.next().unwrap().parse().unwrap(),
                iter.next().unwrap().parse().unwrap(),
                iter.next().unwrap().parse().unwrap()
            ))
        }
        vec![pos]
    }
}
fn collide(pos: &Vec<na::Point3<f32>>, a:usize, b:usize)->usize{
    if (b-a) >= 100{
        let mid = (b-a)/2;
        let (lo, hi) = rayon::join(|| collide(pos, a, mid), || collide(pos, mid, b));
        return lo + hi;
    }
    else {
        let mut num = 0;
        for i in a..b{
            for j in (i+1)..pos.len(){
                if pos[j].x-pos[i].x > 0.05{
                    break;
                }
                if na::distance_squared(&pos[i],&pos[j]) <= 0.0025{
                    num += 1;
                }
            }
        }
        return num;
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
    let mut pos: Vec<na::Point3<f32>> = Vec::with_capacity(buf.len()/59);
    for mut p in parse(&buf){
        pos.append(&mut p);
    }

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

/*     let n =pos.len();
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
     */
    let num = collide(&pos, 0, pos.len());
    println!("\tCollision time \t{}s.", time.elapsed().as_secs_f64());
    println!("{} collisions found.", num);
}
//1436965 expected collisions