#[macro_use]
extern crate gotham_derive;

use std::{process, thread};

use crate::repository::CommentRepository;
use crate::notifier::Notifier;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::iterator::Signals;
use signal_hook::consts::SIGHUP;

pub mod comment;
pub mod repository;
pub mod utils;
pub mod webapi;
pub mod importer;

mod gotham_json;
mod gotham_cors;
mod gravatar;
mod markdown;
mod notifier;
mod sendmail;


pub fn run(repo_path: String, repo_reset: bool, app_path: String, bind_addr: String,
           cors_origin: Option<String>, notify_addr: Option<String>)
{
    let mut repository = CommentRepository::new(&repo_path, repo_reset);

    let reload_flag = Arc::new(AtomicBool::new(true));
    run_signal_handler(&reload_flag);
    repository.set_reload_flag(&reload_flag);
    repository.all_comments();

    if let Some(addr) = notify_addr {
        repository.set_notifier(Notifier::new(&addr))
    }

    webapi::run(repository, &app_path, &bind_addr, &cors_origin);
}


pub fn import(repo_path: String, repo_reset: bool, filename: String)
{
    let repository = CommentRepository::new(&repo_path, repo_reset);
    let result = importer::run(&filename, repository);
    if let Err(message) = result {
        println!("Error during import: {}", message);
        process::exit(1);
    }
}

fn run_signal_handler(reload_flag: &Arc<AtomicBool>)
{
    let reload_flag = Arc::clone(reload_flag);
    let mut signals = Signals::new(&[SIGHUP]).expect("Failed to create signal handler");
    thread::spawn(move || {
        for _ in signals.forever() {
            println!("Will reload comments on next request");
            reload_flag.store(true, Ordering::Relaxed)
        }
    });
}
