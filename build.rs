use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=static/input.css");
    println!("cargo:rerun-if-changed=templates");

    let status = Command::new("tailwindcss")
        .args(["-i", "static/input.css", "-o", "static/style.css", "--minify"])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => eprintln!("cargo:warning=tailwindcss exited with {s}"),
        Err(e) => eprintln!("cargo:warning=tailwindcss not found, skipping CSS build: {e}"),
    }
}
