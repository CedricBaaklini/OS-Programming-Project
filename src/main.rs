mod ta_problem;
use crate::ta_problem::TAOffice;
use std::thread;
use std::thread::JoinHandle;
use ta_problem::{student_thread, ta_thread, MAX_HELP_SESSIONS, NUM_CHAIRS, NUM_STUDENTS};

fn main() {
    println!("Sleeping Teaching Assistant Problem");
    println!(
        "Students: {}, Chairs: {}, Max help sessions per student: {}",
        NUM_STUDENTS, NUM_CHAIRS, MAX_HELP_SESSIONS
    );

    let office = TAOffice::new();
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let ta_office = office.clone();
    let ta_handle: JoinHandle<()> = thread::spawn(move || ta_thread(ta_office));

    for i in 0..NUM_STUDENTS {
        let student_office = office.clone();
        let handle: JoinHandle<()> = thread::spawn(move || student_thread(i, student_office));

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nAll students have finished. Waiting for TA");

    ta_handle.join().unwrap();
}
