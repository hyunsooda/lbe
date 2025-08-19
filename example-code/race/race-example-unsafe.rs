use std::thread;

static mut COUNTER: i32 = 0;

fn main() {
    let mut handles = vec![];

    for _ in 0..2 {
        let handle = thread::spawn(|| unsafe {
            for _ in 0..20000 {
                COUNTER += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    unsafe {
        println!("Final counter value: {}", COUNTER);
    }
}
