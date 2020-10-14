## Part 0: Preparation
- Install Python 3: https://www.python.org/downloads/
- Install required Python dependencies: `pip3 install numpy csv pandas scipy`
- Install Rust: https://rustup.rs/
- Consider enabling and installign Windows Subsystem for Linux: https://docs.microsoft.com/en-us/windows/wsl/install-win10
- (Linux/WSL only) Install Valgrind: `sudo apt-get install valgrind`
- Recommended: install an editor with syntax highlight, one of: vim, vscode, notepad++.

## Part 1: Get the resources
- Clone and compile the CLI:
    * `git clone https://github.com/hegza/serpent-cli`
    * `cargo install --path .`
- TODO: download / install the example use case (TODO: make it into a standalone repo)

## Part 1: Generate code and dataset
- Run `python3 generate-dataset.py`

## Part 2: Fix the rest with help from the compiler
- Run `cargo run` and fixed based on compiler output
1. Cargo.toml: edition = "2018"
2. cannot find value `si` in this scope (black_scholes.rs)
    * We need to replace the scipy functionality with our own. We'll use the `statrs` library.
    * Add to the top of the file:
        `use statrs::distribution::{Normal, Univariate};`
    * Then replace the 4 instances of:
        `si.norm.cdf(-d2, 0, 1)`
    * with:
        `Normal::new(0.0, 1.0).unwrap().cdf(-d2)`
    * d2 varies between the four calls to this new function.
3. cannot find value `pd` in this scope
    * we can't read inputs the same way, thus we must write our own
    1. remove the input re-structuring code on lines 9--21
    2. replace it with:
    ```
    let csv = std::fs::read_to_string("dataset.csv").expect("cannot input csv");
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
    ```
    3. Then move the euro_vanilla calls inside of the loop, and the timers outside of the loop, like so:
    ```
    let start_time = std::time::Instant::now();
    for .. {
        ..
        euro_vanilla_put(S, K, T, r, sigma);
        euro_vanilla_call(S, K, T, r, sigma);
    }
    let duration = std::time::Instant::now() - start_time;
    ```
4. cannot find function `time` in this scope
5. expected function, found macro `print`
    * Our mapping of print is partial. We can fix it by following compiler instructions.
    * Then we need to convert it to use format strings like so:
    `print!("Calculating put and call for {} options took {} seconds", n, duration)`
    * Convert the duration to seconds `duration.as_secs_f64()`
6. the type placeholder `_` is not allowed within types on item signatures    * The Python code did not have type ascriptions, so the transpiler had to use a placeholder.
    * We replace the placeholders with f64.
    * Let's also add the missing return value for the functions: `pub fn .. -> f64`

## Part 3: Benchmark & Profile
0. Calculate the amount of data: (5x 9000000 for inputs + 2x 9000000 for outputs) * 8 bytes = 0.504 GB
1. Run Rust with `cargo run --release`. Running without `release` will be 50 times slower.
2. valgrind --tool=massif python3 __init__.py
    * runs for a minute on my computer
3. ms_print massif.out.* | less
    * shows a graph up to 1.106 GB memory consumption: https://puu.sh/GCDDo/df8e19a66a.png
4. valgrind --tool=massif target/release/main
    * runs for about 30 seconds on my computer
5. ms_print massif.out.* | less
    * shows a graph to 1.140 GB memory consumption: https://puu.sh/GCDxO/f50290774f.png