use rand::Rng;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const NUM_STUDENTS: usize = 5;
pub const NUM_CHAIRS: usize = 3;
pub const MAX_HELP_SESSIONS: usize = 2;

pub struct Semaphore {
    mutex: Mutex<isize>,
    cvar: Condvar,
}

impl Semaphore {
    pub fn new(count: isize) -> Self {
        Semaphore {
            mutex: Mutex::new(count),
            cvar: Condvar::new(),
        }
    }

    //The equivalent of our "wait" function.
    pub fn acquire(&self) {
        let mut count = self.mutex.lock().unwrap();

        while count <= 0 {
            count = self.cvar.wait(count).unwrap();
        }

        *count -= 1;
    }

    pub fn try_acquire(&self) -> bool {
        let mut count = self.mutex.lock().unwrap();

        if *count > 0 {
            *count -= 1;
            true
        } else {
            false
        }
    }

    //The equivalent of our "signal" function.
    pub fn release(&self) {
        let mut count = self.mutex.wait().unwrap();
        *count += 1;
    }
}

#[derive(Clone)]
pub struct TAOffice {
    ta_sleeping: Arc<Semaphore>,
    students_waiting: Arc<Semaphore>,
    chairs: Arc<Semaphore>,

    waiting_students: Arc<Mutex<VecDeque<usize>>>,
    students_helped: Arc<Mutex<Vec<usize>>>,
    current_student: Arc<Mutex<Option<usize>>>,
}

impl TAOffice {
    pub fn new() -> Self {
        TAOffice {
            ta_sleeping: Arc::new(Semaphore::new(0)),
            students_waiting: Arc::new(Semaphore::new(0)),
            chairs: Arc::new(Semaphore::new(NUM_CHAIRS)),

            waiting_students: Arc::new(Mutex::new(VecDeque::new)),
            students_helped: Arc::new(Mutex::new(vec![0, NUM_STUDENTS])),
            current_student: Arc::new(Mutex::new(None)),
        }
    }

    pub fn student_seeks_help(&self, student_id: usize) -> bool {
        println!("Student {} arrives at TA's office", student_id);

        if !self.chairs.try_acquire() {
            println!(
                "Student {} finds no available chairs and leaves",
                student_id
            );
            return false;
        }

        println!("Student {} sits in the hallway", student_id);

        {
            let mut waiting = self.waiting_students.lock().unwrap();
            waiting.push_back();
        }

        self.students_waiting.release();

        self.ta_sleeping.release();

        self.wait_for_turn(student_id);

        self.chairs.release();

        true
    }

    fn wait_for_turn(&self, student_id: usize) {
        loop {
            {
                let current = self.current_student.lock().unwrap();

                if let Some(current_id) = *current {
                    if current_id == student_id {
                        break;
                    }
                }
            }

            {
                let helped = self.students_helped.lock().unwrap();

                if helped[student_id] > 0 {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn ta_work() {
        println!("TA arrives at the office and goes to sleep");

        loop {
            if self.all_students_done() {
                println!("All students have been helped twice. TA is done for the day");
                break;
            }

            println!("TA is sleeping...");

            self.ta_sleeping.acquire();

            println!("TA wakes up!");

            while self.students_waiting.try_acquire() {
                let student_id = {
                    let mut waiting = self.waiting_students.lock().unwrap();

                    waiting.pop_front()
                };

                if let Some(student_id) = student_id {
                    self.help_student(student_id);
                } else {
                    break;
                }
            }
        }
    }

    fn help_student(&self, student_id: usize) {
        {
            let mut current = self.current_student.lock().unwrap();
            *current = Some(student_id);
        }

        println!("TA helps student {}", student_id);

        let help_time = rand::thread_rng().gen_range(1000..3000);
        thread::sleep(Duration::from_millis(help_time));

        {
            let mut helped = self.students_helped.lock().unwrap();
            helped[student_id] += 1;

            println!(
                "TA finishes helping Student {} (help session {}/{})",
                student_id, helped[student_id], MAX_HELP_SESSIONS
            );
        }

        {
            let mut current = self.current_student.lock().unwrap();

            *current = None;
        }

        println!("Student {} leaves the office", student_id);
    }

    fn all_students_done(&self) -> bool {
        let helped = self.students_helped.lock().unwrap();

        helped.iter().all(|&count| count >= MAX_HELP_SESSIONS)
    }

    pub fn get_help_count(&self, student_id: usize) -> usize {
        let helped = self.students_helped.lock().unwrap();

        helped[student_id]
    }
}

pub fn student_thread(student_id: usize, office: TAOffice) {
    let mut rng = rand::thread_rng();

    while office.get_help_count(student_id) < MAX_HELP_SESSIONS {
        let programming_time = rng.gen_range(2000..5000);

        println!(
            "Student {} is programming for {}ms",
            student_id, programming_time
        );

        thread::sleep(Duration::from_millis(programming_time));

        let got_help = office.student_seeks_help(student_id);

        if !got_help {
            let wait_time = rng.gen_range(1000..3000);

            println!("Student {} will try again in {}ms", student_id, wait_time);
        }
    }

    println!(
        "Student {} has received help {} times and is done!",
        student_id, MAX_HELP_SESSIONS
    );
}

pub fn ta_thread(office: TAOffice) {
    office.ta_work();
}
