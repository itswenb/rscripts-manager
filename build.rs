use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=static/input.css");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=assets/brand/ripeline.ico");

    if let Err(err) = embed_windows_icon() {
        panic!("failed to embed windows icon: {err}");
    }

    let status = Command::new("tailwindcss")
        .args([
            "-i",
            "static/input.css",
            "-o",
            "static/style.css",
            "--minify",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => eprintln!("cargo:warning=tailwindcss exited with {s}"),
        Err(e) => eprintln!("cargo:warning=tailwindcss not found, skipping CSS build: {e}"),
    }

    if let Err(err) = embed_static_assets() {
        panic!("failed to embed static assets: {err}");
    }
}

fn embed_windows_icon() -> io::Result<()> {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let icon_path = manifest_dir.join("assets/brand/ripeline.ico");
    if !icon_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("missing windows icon at {}", icon_path.display()),
        ));
    }

    let mut resource = winres::WindowsResource::new();
    resource.set_icon(icon_path.to_string_lossy().as_ref());
    resource
        .compile()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
}

fn embed_static_assets() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let static_dir = manifest_dir.join("static");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_path = out_dir.join("static_assets.rs");
    let mut assets = Vec::new();

    collect_assets(&static_dir, &static_dir, &mut assets)?;
    assets.sort_by(|left, right| left.0.cmp(&right.0));

    let mut generated = String::from("pub static STATIC_ASSETS: &[(&str, &[u8])] = &[\n");
    for (route_path, file_path) in assets {
        generated.push_str("    (");
        generated.push_str(&format!("{route_path:?}"));
        generated.push_str(", include_bytes!(");
        generated.push_str(&format!("{:?}", file_path.to_string_lossy()));
        generated.push_str(")),\n");
    }
    generated.push_str("];\n");

    fs::write(out_path, generated)
}

fn collect_assets(root: &Path, dir: &Path, assets: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", dir.display());

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name == ".DS_Store" {
            continue;
        }

        if path.is_dir() {
            collect_assets(root, &path, assets)?;
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
            let route_path = path
                .strip_prefix(root)
                .expect("static asset must be under static root")
                .to_string_lossy()
                .replace('\\', "/");
            assets.push((route_path, path));
        }
    }

    Ok(())
}
