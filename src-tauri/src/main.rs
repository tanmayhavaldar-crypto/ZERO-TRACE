#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // This simply tells the app to run the brain we just built in lib.rs!
    app_lib::run();
}