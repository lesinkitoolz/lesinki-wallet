fn main() {
  println!("cargo:warning=Build script starting...");
  println!("cargo:warning=CWD: {:?}", std::env::current_dir());
  println!("cargo:warning=OUT_DIR: {:?}", std::env::var("OUT_DIR"));
  tauri_build::build()
}
