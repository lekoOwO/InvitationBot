use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=migrations/");

    // Install sqlx-cli if not already installed
    if let Err(e) = Command::new("cargo").args(["install", "sqlx-cli"]).status() {
        eprintln!("Failed to install sqlx-cli: {:?}", e);
        std::process::exit(1);
    }

    // Create the database
    if let Err(e) = Command::new("cargo")
        .args(["sqlx", "database", "create"])
        .status()
    {
        eprintln!("Failed to create database: {:?}", e);
        std::process::exit(1);
    }

    // Run migrations
    if let Err(e) = Command::new("cargo")
        .args(["sqlx", "migrate", "run"])
        .status()
    {
        eprintln!("Failed to run migrations: {:?}", e);
        std::process::exit(1);
    }
}
