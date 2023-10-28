use std::{
    path::{Path, PathBuf},
    process::Command,
};

struct Vars {
    cwd: PathBuf,
    target_dir: PathBuf,
}

fn grcov(target_dir: &Path, format: &str, build_type: &str) {
    let ret = Command::new("grcov")
        .args(["."])
        .args([
            "--binary-path",
            target_dir
                .join(format!("{build_type}/deps"))
                .to_str()
                .unwrap(),
        ])
        .args(["-s", "."])
        .args(["-t", format])
        .args(["--branch"])
        .args(["--ignore-not-existing"])
        .args(["-o", target_dir.join(format).to_str().unwrap()])
        .args(["--keep-only", "src/*"])
        .args(["--keep-only", "derive/src/*"])
        .status()
        .expect("Perhaps you need to run 'cargo install grcov'")
        .success();
    assert!(ret);
}

fn code_coverage(vars: &Vars) {
    let build_type = "debug";
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
        .args(match build_type {
            "release" => vec!["--release"],
            "debug" => vec![],
            _ => unreachable!("{build_type:?}"),
        })
        .status()
        .unwrap()
        .success();
    assert!(ret);

    grcov(&target_dir, "html", build_type);
    grcov(&target_dir, "lcov", build_type);

    let ret = Command::new("genhtml")
        .args(["-o", target_dir.join("html2").to_str().unwrap()])
        .args(["--show-details"])
        .args(["--highlight"])
        .args(["--ignore-errors", "source"])
        .args(["--legend", target_dir.join("lcov").to_str().unwrap()])
        .status()
        .unwrap()
        .success();
    assert!(ret);

    println!("Now open:");
    println!(
        "  file://{}/html/index.html",
        vars.cwd.join(&target_dir).display()
    );
    println!(
        "  file://{}/html2/index.html",
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
