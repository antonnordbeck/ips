use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Pos{
    x: f32,
    y: f32,
    z: f32
}

fn main(){
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').from_path("positions.xyz").unwrap();
    let positions: Vec<Pos> = rdr.deserialize().map(|pos| pos.unwrap()).collect();

    let mut num = 0;
    for i in 0..positions.len(){
        for j in (i+1)..positions.len(){
            let dist = f32::sqrt((positions[i].x+positions[j].x).powi(2)+(positions[i].y+positions[j].y).powi(2)+(positions[i].z+positions[j].z).powi(2));
            if dist <= 0.05{
                num += 1;
            }
        }
    }
    println!("{}", num)
}