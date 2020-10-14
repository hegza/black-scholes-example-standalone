pub mod black_scholes;
// External imports

pub use crate::black_scholes::euro_vanilla_put;
pub use crate::black_scholes::euro_vanilla_call;


pub fn main() {
    let csv = std::fs::read_to_string("../dataset.csv").expect("cannot input csv");
    let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(csv.as_bytes());
    let n = reader.records().count();
    let (mut S, mut K, mut T, mut r, mut sigma) = (vec![0.; n], vec![0.; n], vec![0.; n], vec![0.; n], vec![0.; n]);
    for record in reader.records() {
        let record = record.expect("cannot read record").iter().map(|r| r.parse::<f64>().expect("cannot parse float")).collect::<Vec<f64>>();
        S.push(record[0]);
        K.push(record[1]);
        T.push(record[2]);
        r.push(record[3]);
        sigma.push(record[4]);
    }
    
    // We time this loop
    let start_time = std::time::Instant::now();
    for i in 0..n {
        euro_vanilla_put(S[i], K[i], T[i], r[i], sigma[i]);
        euro_vanilla_call(S[i], K[i], T[i], r[i], sigma[i]);
    }
    let duration = std::time::Instant::now() - start_time;

    print!("Calculating put and call for {} options took {} seconds", n, duration.as_secs_f64());
}







