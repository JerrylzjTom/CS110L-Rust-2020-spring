use crossbeam_channel;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::fmt::Debug;
fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default + Debug + Clone, // Add Debug trait for Arc::try_unwrap. Add Clone trait for initialization of output_vec.
{
    let output_vec = Arc::new(Mutex::new(vec![U::default(); input_vec.len()])); 
    // TODO: implement parallel map!
    let (sender, receiver) = crossbeam_channel::unbounded();
    let mut threads = Vec::new();
    for _ in 0..num_threads {
        let receiver = receiver.clone();
        let output_vec = Arc::clone(&output_vec);
        threads.push(thread::spawn(move || {
            while let Ok((index, item)) = receiver.recv() {
                let index_num = (index, f(item));
                output_vec.lock().unwrap()[index_num.0] = index_num.1;
            }
        }))
    }

    while let Some(item) = input_vec.pop() {
        let index = input_vec.len();
        sender.send((index, item)).expect("Failed to send item");
    };

    drop(sender);

    for thread in threads {
        thread.join().expect("Thread failed to complete");
    }
    
    Arc::try_unwrap(output_vec)
        .expect("Failed to unwrap Arc")
        .into_inner()
        .expect("Failed to unlock Mutex")

}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
