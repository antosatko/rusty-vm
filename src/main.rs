use reader::reader::*;
extern crate runtime;
use runtime::runtime::runtime_types::*;
use std::{env, time::SystemTime};

mod reader;
mod test;
//mod writer;
fn main() {
    let mut args = env::args();
    let mut report = true;
    let mut ctx = match args.nth(1) {
        Some(src) => read_file(src, Context::new()),
        None => {
            /*println!("Path not specified. Program will terminate."); return;*/
            use test::test::*;
            let mut ctx = Context::new();
            report = test_init(None, &mut ctx);
            ctx
        }
    };
    let start_time = SystemTime::now();
    ctx.run();
    if report {
        ctx.data_report(Some(
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_millis(),
        ));
    }
    ctx.libs.clear()
}
