use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const PACKAGES: &[PackageTarget] = &[
    PackageTarget {
        command: "pkg-linux-x86_64",
        rust_target: "x86_64-unknown-linux-gnu",
        builder: Builder::Cross,
    },
    PackageTarget {
        command: "pkg-macos-aarch64",
        rust_target: "aarch64-apple-darwin",
        builder: Builder::Cargo,
    },
];

struct PackageTarget {
    command: &'static str,
    rust_target: &'static str,
    builder: Builder,
}

#[derive(Clone, Copy)]
enum Builder {
    Cargo,
    Cross,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let command = env::args().nth(1).unwrap_or_default();
    match command.as_str() {
        "pkg-all" => {
            clean_pkg_root()?;
            for package in PACKAGES {
                package_target(package)?;
            }
        }
        command => {
            let package = PACKAGES
                .iter()
                .find(|package| package.command == command)
                .ok_or_else(|| format!("unknown xtask command: {command}"))?;
            clean_pkg_root()?;
            package_target(package)?;
        }
    }

    Ok(())
}

fn package_target(package: &PackageTarget) -> Result<(), Box<dyn std::error::Error>> {
    let root = project_root()?;
    let pkg_root = root.join("pkg");
    let work_root = env::temp_dir().join("ripeline-pkg");
    let target_dir = work_root.join("target");
    let stage_root = work_root.join("stage");
    let package_name = format!("ripeline-{}", package.rust_target);
    let package_dir = stage_root.join(&package_name);
    let archive_name = format!("{package_name}.tar.gz");
    let archive_path = pkg_root.join(&archive_name);

    fs::create_dir_all(&pkg_root)?;
    if work_root.exists() {
        fs::remove_dir_all(&work_root)?;
    }
    fs::create_dir_all(&work_root)?;

    run_release_build(&root, &target_dir, package)?;

    if stage_root.exists() {
        fs::remove_dir_all(&stage_root)?;
    }
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)?;
    }
    fs::create_dir_all(&package_dir)?;

    fs::copy(
        target_dir
            .join(package.rust_target)
            .join("release")
            .join("ripeline"),
        package_dir.join("ripeline"),
    )?;

    if archive_path.exists() {
        fs::remove_file(&archive_path)?;
    }

    let status = Command::new("tar")
        .current_dir(&stage_root)
        .args([
            "-czf",
            archive_path.to_string_lossy().as_ref(),
            &package_name,
        ])
        .status()?;
    if !status.success() {
        return Err(format!("tar failed with status {status}").into());
    }

    fs::remove_dir_all(&stage_root)?;
    remove_old_pkg_artifacts(&pkg_root)?;

    println!("built {}", archive_path.display());
    Ok(())
}

fn project_root() -> io::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "xtask has no parent directory"))
}

fn clean_pkg_root() -> io::Result<()> {
    let root = project_root()?;
    let pkg_root = root.join("pkg");
    if pkg_root.exists() {
        fs::remove_dir_all(&pkg_root)?;
    }
    fs::create_dir_all(pkg_root)
}

fn run_release_build(
    root: &Path,
    target_dir: &Path,
    package: &PackageTarget,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = match package.builder {
        Builder::Cargo => Command::new("cargo"),
        Builder::Cross => {
            let mut command = Command::new("cross");
            command.env("CROSS_CONTAINER_OPTS", "--platform linux/amd64");
            command
        }
    };

    let status = command
        .current_dir(root)
        .env("CARGO_TARGET_DIR", target_dir)
        .args(["build", "--release", "--target", package.rust_target])
        .status()?;

    if !status.success() {
        return Err(format!("build failed with status {status}").into());
    }

    Ok(())
}

fn remove_old_pkg_artifacts(pkg_root: &Path) -> io::Result<()> {
    for entry in fs::read_dir(pkg_root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
            continue;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}
