
// #[cfg(unix)]
// use zerocopy::AsBytes;
// 


// fn main() {
//     println!("Connecting to hello world server...");
//     let context = zmq::Context::new();
//     let requester = context.socket(zmq::REQ).unwrap();
//     assert!(requester.connect("tcp://localhost:5555").is_ok());

//     for request_nbr in 0..10 {
//         let buffer = &mut [0; 10];
//         println!("Sending Hello {:?}...", request_nbr);
//         requester.send("Hello".as_bytes(), 0).unwrap();
//         requester.recv_into(buffer, 0).unwrap();
//         println!("Received World {:?}", request_nbr);
//     }
// }


#[cfg(windows)]
use std::ptr::write_volatile;
use std::{path::PathBuf, ptr::write};

#[cfg(unix)]
use zmq;
use zerocopy::AsBytes;

#[cfg(feature = "tui")]
use libafl::monitors::tui::{ui::TuiUI, TuiMonitor};
#[cfg(not(feature = "tui"))]
use libafl::monitors::SimpleMonitor;
use libafl::{
    corpus::{InMemoryCorpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::RandPrintablesGenerator,
    inputs::{BytesInput, HasTargetBytes},
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
};
use libafl_bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice};

/// Coverage map with explicit assignments due to the lack of instrumentation
static mut SIGNALS: [u8; 16] = [0; 16];
static mut SIGNALS_PTR: *mut u8 = unsafe { SIGNALS.as_mut_ptr() };

/// Assign a signal to the signals map
fn signals_set(idx: usize) {
    unsafe { write(SIGNALS_PTR.add(idx), 1) };
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[allow(clippy::similar_names, clippy::manual_assert)]
pub fn main() {

    // Set up the message queue to send the input.
    let context = zmq::Context::new();
    let requester = context.socket(zmq::REQ).unwrap();
    assert!(requester.connect("tcp://localhost:5555").is_ok());

    let mut harness = |input: &BytesInput| {
        let mut buf:Vec<u8>= vec![0;6];
        requester.send(input.target_bytes().as_slice(), 0).unwrap();
        buf = requester.recv_bytes(0).unwrap();
        // println!("Received Bytes {:?}", buf);
        // println!("buf[0] == 1u8 {:?}", buf[0] == 1u8);
        // println!("buf[0] = {:?}", buf[0]);
        // print_type_of(&buf[0]);
        // print_type_of(&1u8);
        if buf[1] == 1u8 {
            signals_set(1);
        }
        if buf[2] == 1u8 {
            signals_set(2);
        }
        if buf[3] == 1u8 {
            signals_set(3);
        }
        if buf[4] == 1u8 {
            signals_set(4);
        }
        if buf[5] == 1u8 {
            signals_set(5);
            #[cfg(unix)]
            panic!("Artificial bug triggered =)");
            // panic!() raises a STATUS_STACK_BUFFER_OVERRUN exception which cannot be caught by the exception handler.
            // Here we make it raise STATUS_ACCESS_VIOLATION instead.
            // Extending the windows exception handler is a TODO. Maybe we can refer to what winafl code does.
            // https://github.com/googleprojectzero/winafl/blob/ea5f6b85572980bb2cf636910f622f36906940aa/winafl.c#L728
            #[cfg(windows)]
            unsafe {
                write_volatile(0 as *mut u32, 0);
            }
        }
        ExitKind::Ok
    };
    // Create an observation channel using the signals map
    let observer = unsafe { StdMapObserver::from_mut_ptr("signals", SIGNALS_PTR, SIGNALS.len()) };

    // Feedback to rate the interestingness of an input
    let mut feedback = MaxMapFeedback::new(&observer);

    // A feedback to choose if an input is a solution or not
    let mut objective = CrashFeedback::new();

    // create a State from scratch
    let mut state = StdState::new(
        // RNG (current_nanos is a function to get the epoch time (nanoseconds) in unix system. )
        StdRand::with_seed(current_nanos()),
        // Corpus that will be evolved, we keep it in memory for performance
        InMemoryCorpus::new(),
        // Corpus in which we store solutions (crashes in this example),
        // on disk so the user can get them after stopping the fuzzer
        OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
        // States of the feedbacks.
        // The feedbacks can report the data that should persist in the State.
        &mut feedback,
        // Same for objective feedbacks
        &mut objective,
    )
    .unwrap();

    // The Monitor trait define how the fuzzer stats are displayed to the user
    #[cfg(not(feature = "tui"))]
    let mon = SimpleMonitor::new(|s| println!("{s}"));
    #[cfg(feature = "tui")]
    let ui = TuiUI::with_version(String::from("Baby Fuzzer"), String::from("0.0.1"), false);
    #[cfg(feature = "tui")]
    let mon = TuiMonitor::new(ui);

    // The event manager handle the various events generated during the fuzzing loop
    // such as the notification of the addition of a new item to the corpus
    let mut mgr = SimpleEventManager::new(mon);

    // A queue policy to get testcasess from the corpus
    let scheduler = QueueScheduler::new();

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // Create the executor for an in-process function with just one observer
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .expect("Failed to create the Executor");

    // Generator of printable bytearrays of max size 32
    let mut generator = RandPrintablesGenerator::new(32);
    // I tried to use the default corpus but I failed
    // let seed_dir = PathBuf::from("test");

    // Generate 8 initial inputs
    state
        .generate_initial_inputs(&mut fuzzer, &mut executor, &mut generator, &mut mgr, 1000)
        .expect("Failed to generate the initial corpus");

    // Setup a mutational stage with a basic bytes mutator
    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in the fuzzing loop");
}