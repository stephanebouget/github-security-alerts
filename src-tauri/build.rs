fn main() {
  // Load environment variables from .env file
  // First try the current directory (src-tauri)
  if dotenv::dotenv().is_err() {
    // If not found, try the parent directory (project root)
    let _ = dotenv::from_filename("../.env");
  }
  
  // Pass environment variables to the build
  if let Ok(client_id) = std::env::var("CLIENT_ID") {
    println!("cargo:rustc-env=CLIENT_ID={}", client_id);
  }
  if let Ok(client_secret) = std::env::var("CLIENT_SECRET") {
    println!("cargo:rustc-env=CLIENT_SECRET={}", client_secret);
  }
  
  tauri_build::build()
}
