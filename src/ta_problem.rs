use rand::Rng;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const NUM_STUDENTS: usize = 5;
const NUM_CHAIRS: usize = 3;
const MAX_HELP_SESSIONS: usize = 2;

#[derive(Clone)]
struct TAOffice {
    waiting_students: Arc<Mutex<VecDeque<usize>>>,
    ta_sleeping: Arc<Mutex<bool>>,
    ta_helping: Arc<Mutex<bool>>,
    ta_wakeup: Arc<Condvar>,
    student_helped: Arc<Condvar>,
    students_helped: Arc<Mutex<Vec<usize>>>,
}

impl TAOffice {
    fn new() -> Self {
        TAOffice {
            waiting_students: Arc::new(Mutex::new(VecDeque::new())),
            ta_sleeping: Arc::new(Mutex::new(true)),
            ta_helping: Arc::new(Mutex::new(false)),
            ta_wakeup: Arc::new(Condvar::new()),
            student_helped: Arc::new(Condvar::new()),
            students_helped: Arc::new(Mutex::new(vec![0, NUM_STUDENTS])),
        }
    }

    fn student_arrives(&self, student_id: usize) -> bool {
        println!("Student {} arrives at TA's office", student_id);

        let mut waiting = self.waiting_students.lock().unwrap();
        let mut ta_sleeping = self.ta_sleeping.lock().unwrap();
        let ta_helping = self.ta_helping.lock().unwrap();

        if *ta_sleeping && !*ta_helping {
            println!("Student {} wakes up the sleeping TA", student_id);

            *ta_sleeping = false;
            drop(ta_helping);
            drop(waiting);

            self.ta_wakeup.notify_one();
            drop(ta_sleeping);

            self.wait_for_help(student_id);

            return true;
        }

        return if waiting.len() < NUM_CHAIRS {
            println!(
                "Student {} sits in the hallway (chair {}/{}",
                student_id,
                waiting.len() + 1,
                NUM_CHAIRS
            );

            waiting.push_back(student_id);
            drop(ta_helping);
            drop(waiting);
            drop(ta_sleeping);

            self.wait_for_help(student_id);

            true
        } else {
            false
        };
    }

    fn wait_for_help(&self, student_id: usize) {
        let mut helped = self.students_helped.lock().unwrap();

        while helped[student_id] == 0
            || (helped[student_id] < MAX_HELP_SESSIONS && !self.is_student_being_helped(student_id))
        {
            helped = self.student_helped.wait(helped).unwrap();
        }
    }

    fn is_student_being_helped(&self, student_id: usize) -> bool {
        let waiting = self.waiting_students.lock().unwrap();
        let ta_helping = self.ta_helping.lock().unwrap();

        *ta_helping && (waiting.is_empty() || waiting.front() != Some(&student_id))
    }

    fn ta_work(&self) {
        loop {
            if self.all_students_done() {
                println!("All students were helped out twice. TA is done for the day!");
                break;
            }

            let mut ta_sleeping = self.ta_sleeping.lock().unwrap();
            let waiting = self.waiting_students.lock().unwrap();

            if waiting.is_empty() {
                println!("TA has no students to help and returns to sleep");
                *ta_sleeping = true;
                drop(waiting);

                ta_sleeping = self.ta_wakeup.wait(ta_sleeping).unwrap();
                println!("TA wakes up");
            }

            drop(ta_sleeping);

            while let Some(student_id) = {
                let mut waiting = self.waiting_students.lock().unwrap();
                waiting.pop_front()
            } {
                self.help_student(student_id);
            }
        }
    }

    fn help_student(&self, student_id: usize) {
        {
            let mut ta_helping = self.ta_helping.lock().unwrap();
            *ta_helping = true;
        }

        println!("TA has started helping Student {}", student_id);

        let help_time = rand::rng().random_range(1000..3000);
        thread::sleep(Duration::from_millis(help_time));

        {
            let mut helped = self.students_helped.lock().unwrap();
            helped[student_id] += 1;
            println!(
                "TA has finished helping Student {} (help session {}/{})",
                student_id, helped[student_id], MAX_HELP_SESSIONS
            );
        }

        {
            let mut ta_helping = self.ta_helping.lock().unwrap();
            *ta_helping = false;
        }

        self.student_helped.notify_all();
    }

    fn all_students_done(&self) -> bool {
        let helped = self.students_helped.lock().unwrap();
        helped.iter().all(|&count| count >= MAX_HELP_SESSIONS)
    }

    fn get_help_count(&self, student_id: usize) -> usize {
        let helped = self.students_helped.lock().unwrap();
        helped[student_id]
    }
}
