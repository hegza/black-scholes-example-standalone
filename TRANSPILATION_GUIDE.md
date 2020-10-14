## Part 0: Preparation
- Recommended: install an editor with syntax highlight, one of: vim, vscode, notepad++.
1. Make sure to have some form of a terminal, choose one of the options below:
	* (Linux) Linux machines should be able to use a system terminal.
	* (Windows): Enable and install Windows Subsystem for Linux: https://docs.microsoft.com/en-us/windows/wsl/install-win10.
	* (Windows) Install Git Bash. Untested. Dependencies may be hard to install.
	* (Windows) Install Cygwin. Untested.
	* (Windows) You might be able to use a Windows Command Prompt. Untested.
2. Install Python 3:
	* (Windows) https://www.python.org/downloads/. Make sure to have the Python executable on system path. Try `python3` on the chosen terminal to verify.
	* (Linux) from package management, eg. `sudo apt-get install python3` on most debian systems.
3. Install required Python dependencies. Python configurations easily break between machines, so we recommend one of the following. If Python dependencies break down the line, try the other one.
	* (WSL) `pip3 install numpy csv pandas scipy` on terminal
	* (Linux) `sudo apt-get install python3-pandas python3-numpy`
	* If you know what's going on, you can future proof by using a virtual environment (conda, venv, venvironment) instead of the options above.
4. Install Rust: https://rustup.rs/
5. Install libssh:
	* (WSL / Linux) `sudo apt-get install libssh-dev` on most debian systems.
	* (Git Bash) hope that libssh is already installed or try to find an installer on google.
	* (Cygwin) Use the cygwin management tool to install `libssh-dev`.
6. Optional: install valgrind:
	* (WSL / Linux) sudo apt-get install valgrind` on most debian systems.


## Part 1: Get the resources
1. Clone and compile the command line interface (CLI):
    * `git clone https://github.com/hegza/serpent-cli`
	* `cd serpent-cli`
	* Compile and install the CLI: `cargo install --path .`
2. Download or clone the example use case: https://github.com/hegza/black-scholes-example-standalone


## Part 2: Try the original code, generate the dataset and run the transpiler
1. Switch to the example repository (`cd ../black*`).
2. Run `python3 generate-dataset.py`
3. Try the original code:
	* `cd black-scholes`
	* `python3 __init__.py`, this should take a few seconds.
4. Run the tool to transpile the Python source:
	* `cd ..`
	* Run the tool with `black-scholes` directory as input, and `transpiled` as output:
	* `serpent tp black-scholes -o transpiled --emit-manifest`
	* This will create a directory "transpiled".

## Part 3: Fix the rest with help from the compiler
- Switch to the Rust project directory: `cd transpiled`
- Fix remaining compiler errors Rust project by repeatedly running `cargo check`. The errors and fixes are listed below from first to last.
- When in doubt, run `cargo check`!

1. In Cargo.toml, search for the "[package]" key, and add edition = "2018" under it
2. error[E0425]: cannot find value `si` in this scope
    * We need to replace the scipy functionality with our own. We'll use the `statrs` library.
    * Add to the top of the file in "src/black_scholes.rs":
        `use statrs::distribution::{Normal, Univariate};`
    * Then replace the 4 instances of:
        `si.norm.cdf(-d2, 0, 1)`
    * with:
        `Normal::new(0.0, 1.0).unwrap().cdf(-d2)`
    * d2 varies between the four calls to this new function: -d2, -d1, d1, d2
3. error[E0425]: cannot find value `pd` in this scope
    * The transpiler can't figure out what to do with inputs. We need to replace the input processing with our own implementation.
    1. In `src/main.rs`, remove the input re-structuring code on lines 9--22.
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
4. expected function, found macro `print`
    * Our mapping of print is partial. We can fix it by following compiler instructions.
    * Then we need to convert it to use format strings like so:
    `print!("Calculating put and call for {} options took {} seconds", n, duration);`
    * Convert the duration to seconds `duration.as_secs_f64()`
5. Then move the euro_vanilla calls into a new loop, and the timers outside of the loop, like so:
    ```
    let start_time = std::time::Instant::now();
    for i in 0..n {
        euro_vanilla_put(S[i], K[i], T[i], r[i], sigma[i]);
        euro_vanilla_call(S[i], K[i], T[i], r[i], sigma[i]);
    }
    let duration = std::time::Instant::now() - start_time;
    ```
6. error[E0121]: the type placeholder `_` is not allowed within types on item signatures
	* The Python code did not have type ascriptions, and the transpiler had to use a placeholder.
    * We replace the placeholders like `_` in "src/black_scholes.rs" function signatures with the double-precision floating point type: `f64`.
    * Let's also add the missing return value for the functions: `pub fn .. -> f64`


## Part 4: Benchmark & Profile
0. Calculate the amount of data used by the program:
	* (5x 9000000 for inputs + 2x 9000000 for outputs) * 8 bytes = 0.504 GB
1. Run Rust with `cargo run --release`. Running without `release` will be 50 times slower.
2. valgrind --tool=massif python3 __init__.py
    * runs for a minute on my computer
3. ms_print massif.out.* | less
    * shows a graph up to 1.106 GB memory consumption: https://puu.sh/GCDDo/df8e19a66a.png
4. valgrind --tool=massif target/release/main
    * runs for about 30 seconds on my computer
5. ms_print massif.out.* | less
    * shows a graph to 1.140 GB memory consumption: https://puu.sh/GCDxO/f50290774f.png