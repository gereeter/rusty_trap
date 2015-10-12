extern crate rusty_trap;
use std::path::Path;
use rusty_trap::Inferior;

#[test]
fn it_can_exec() {
    let inferior = Inferior::exec(Path::new("./target/debug/twelve"), &[]).unwrap();
    assert_eq!(12, inferior.cont(&mut |_, _| {}));
}

#[test]
fn it_can_set_breakpoints() {
    let mut breakpoint_count: i32 = 0;

    let mut inferior = Inferior::exec(Path::new("./target/debug/twelve"), &[]).unwrap();
    inferior.set_breakpoint(0x00005555555585f0);
    inferior.cont(&mut |passed_inferior, passed_bp| {
        //assert_eq!(passed_inferior, inferior);
        //assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 1);
}

#[test]
fn it_can_handle_a_breakpoint_more_than_once () {
    let mut breakpoint_count: i32 = 0;

    let mut inferior = Inferior::exec(Path::new("./target/debug/loop"), &[]).unwrap();
    inferior.set_breakpoint(0x5555555585d0);
    inferior.cont(&mut |passed_inferior, passed_bp| {
        //assert_eq!(passed_inferior, inferior);
        //assert_eq!(passed_bp, bp);
        breakpoint_count += 1;
    });

    assert_eq!(breakpoint_count, 5);
}
