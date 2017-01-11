extern crate futures;
extern crate futures_cpupool;


pub use futures::*;
pub use std::thread;
pub use futures_cpupool::CpuPool;

#[macro_export]
macro_rules! async {
    ($e: expr) => ({
        let (tx, rx) = oneshot();
        thread::spawn(move || {
            tx.complete($e);
        });
        rx
    });
    ($block: block) => ({
        let (tx, rx) = oneshot();
        thread::spawn(move || {
            tx.complete($block);
        });
        rx
    });
    ($e: expr, $pool: ident) => ({
        let (tx, rx) = oneshot();
        $pool.spawn({
            tx.complete($e);
            rx
        })
    });
    ($block: block, $pool: ident) => ({
        let (tx, rx) = oneshot();
        $pool.spawn(move || {
            tx.complete($block);
            rx
        });
    });
}

#[macro_export]
macro_rules! await {
    ($f: expr) => {
        $f.wait().unwrap()
    };
    ($f: expr, $d: expr) => {
        match $f.wait() {
            Ok(e) => e,
            Err(_) => $d
        }
    }
}

#[test]
fn test_simple_async() {
    let a = async!{42};
    assert_eq!(a.wait().unwrap(), 42);
}

#[test]
fn test_complex_async() {
    let f1 = async!{42};
    let f2 = async!{18};
    let transformation = f1.map(|v| v * 2).join((f2.map(|v| v + 5)))
        .and_then(|(v1, v2)| Ok(v1 - v2));
    assert_eq!(61, await!{transformation});
}

#[test]
fn test_block() {
    let f1 = async!{{
        let f1 = async!{42};
        await!{f1.map(|v| v * 2)}
    }};
    assert_eq!(84, await!{f1})
}

#[test]
fn test_await() {
    let a = async!{42};
    assert_eq!(await!(a), 42);
}

#[test]
fn test_default() {
    let a = async!{panic!("i")};
    let res = await!(a, 9711);
    assert_eq!(res, 9711);
}

#[test]
fn test_cpupool_expression() {
    let pool = CpuPool::new(4);
    let a = async!({42}, pool);
    assert_eq!(a.wait().unwrap(), 42)
}

#[test]
fn test_cpupool_block() {
    let pool = CpuPool::new(4);
    let f1 = async!({
        let f1 = async!({42}, pool);
        await!{f1.map(|v| v * 2)}
    }, pool);
    assert_eq!(84, await!{f1})
}
