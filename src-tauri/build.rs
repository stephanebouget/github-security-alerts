fn main() {
  // Load environment variables from .env file
  // First try the current directory (src-tauri)
  if dotenv::dotenv().is_err() {
    // If not found, try the parent directory (project root)
    let _ = dotenv::from_filename("../.env");
  }
  
  tauri_build::build()
}
