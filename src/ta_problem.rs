use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

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
    students_helped: Arc<Mutex<usize>>,
}
