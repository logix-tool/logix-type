use std::{path::PathBuf, process::Command};

struct Vars {
    cwd: PathBuf,
    target_dir: PathBuf,
}

fn code_coverage(vars: &Vars) {
    let target_dir = vars.target_dir.join("coverage");

    std::fs::remove_dir_all(&target_dir).unwrap();

    let ret = Command::new("cargo")
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env(
            "LLVM_PROFILE_FILE",
            target_dir.join("cargo-test-%p-%m.profraw"),
        )
        .arg("test")
        .arg("--workspace")
        .status()
        .unwrap()
        .success();
    assert!(ret);

    let ret = Command::new("grcov")
        .args(["."])
        .args([
            "--binary-path",
            target_dir.join("debug/deps").to_str().unwrap(),
        ])
        .args(["-s", "."])
        .args(["-t", "html"])
        .args(["--branch"])
        .args(["--ignore-not-existing"])
        .args(["-o", target_dir.join("html").to_str().unwrap()])
        .args(["--ignore", "xtask/src/*"])
        .status()
        .expect("Perhaps you need to run 'cargo install grcov'")
        .success();
    assert!(ret);

    println!(
        "Now open file://{}/html/index.html",
        vars.cwd.join(&target_dir).display()
    );
}

fn main() {
    let vars = Vars {
        cwd: std::env::current_dir().unwrap().canonicalize().unwrap(),
        target_dir: std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| "target".into())
            .canonicalize()
            .unwrap(),
    };

    let mut task = None;

    for arg in std::env::args().skip(1) {
        if task.is_some() {
            eprintln!("Invalid argument {arg:?}");
            std::process::exit(1);
        }
        task = Some(arg);
    }

    match task.as_deref() {
        Some("coverage") => code_coverage(&vars),
        Some(unk) => {
            eprintln!("Unknown command {unk:?}");
            std::process::exit(1);
        }
        None => {
            eprintln!("Not enough arguments");
            std::process::exit(1);
        }
    }
}
