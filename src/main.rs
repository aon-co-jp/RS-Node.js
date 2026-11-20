//! `rnodejs <file.js|file.ts> [args...]`: JavaScript / TypeScript を実行する(feature `boa` が必要)。
#[cfg(feature = "boa")]
fn main() {
    use rnodejs::engine::{run_file, BoaEngine};
    let a: Vec<String> = std::env::args().collect();
    let Some(file) = a.get(1) else {
        eprintln!("使い方 / usage: rnodejs <file.js|file.ts> [args...]");
        std::process::exit(2);
    };
    let src = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("rnodejs: {file}: {e}");
            std::process::exit(1);
        }
    };
    let mut argv = vec!["rnodejs".to_string(), file.clone()];
    argv.extend(a.iter().skip(2).cloned());
    let mut eng = match BoaEngine::new(argv) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("rnodejs: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = run_file(&mut eng, file, &src) {
        eprintln!("rnodejs: {e}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "boa"))]
fn main() {
    eprintln!("rnodejs: feature `boa` なしでビルドされています / built without the `boa` feature");
    std::process::exit(2);
}
