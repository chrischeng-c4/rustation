//! rstn - rustation v3 Tauri application entry point.
//!
//! This is the main entry point for the native application.

// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    rstn_lib::run()
}
