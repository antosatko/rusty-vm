use reader::reader::*;
extern crate runtime;
use runtime::runtime_types::*;
use std::{env, time::SystemTime, mem, fs::read};
mod stringify;

mod reader;
mod test;
//mod writer;
fn main() {
    let mut args = env::args();
    let mut report = false;
    /*let mut ctx = match args.nth(1) {
        Some(src) => read_file(src, Context::new()),
        None => {
            /*println!("Path not specified. Program will terminate."); return;*/
            use test::test::*;
            let mut ctx = Context::new();
            report = test_init(None, &mut ctx);
            ctx
        }
    };*/
    let mut ctx = Context::new();
    report = test::test::test_init(None, &mut ctx);
    let start_time = SystemTime::now();
    ctx.run();
    if report {
        data_report(&ctx, Some(
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_millis(),
        ));
    }
    println!("original code: {:?}", ctx.code.data);
    let str = stringify::stringify(&ctx);
    println!("stringified: {:?}", str);
    let retrieved = stringify::parse(&str);
    println!("parsed: {:?}", retrieved);

    // print path to Rudastd from environment variable
    println!("Rudastd path: {:?}", env::var("RUDA_PATH"));
}

fn data_report(ctx: &Context, runtime: Option<u128>) {
    use colored::Colorize;
    use enable_ansi_support::enable_ansi_support;
    match enable_ansi_support() {
        Ok(_) => {
            print!("\n");
            println!("{}", "Post-process data report.".yellow());
            if let Some(time) = runtime {
                println!("\x1b[90mTotal run time: {} ms\x1b[0m", time);
            }
            println!("{} {:?}", "Heap:".magenta(), ctx.memory.heap.data);
            println!("{} {:?}", "Stack:".magenta(), ctx.memory.stack.data);
            println!("{} {:?}", "Registers:".magenta(), ctx.memory.registers);
            println!("{} {:?}", "Strings:".magenta(), ctx.memory.strings.pool);
        }
        Err(_) => {
            print!("\n");
            println!("{}", "Post-process data report.");
            if let Some(time) = runtime {
                println!("Total run time: {} ms", time);
            }
            println!("{} {:?}", "Heap:", ctx.memory.heap.data);
            println!("{} {:?}", "Stack:", ctx.memory.stack.data);
            println!("{} {:?}", "Registers:", ctx.memory.registers);
            println!("{} {:?}", "Strings:", ctx.memory.strings.pool);
        }
    }
    println!("size in bytes: {}", mem::size_of::<Context>());
    println!("real size in bytes: {}", ctx.size());
    /*let mut ctx = Context::new();
    let time = SystemTime::now();
    for _ in 0..100000 {
        ctx.memory.strings.from_str("Hello World!");
    }
    ctx.memory.gc_sweep();
    println!("time taken: {}", SystemTime::now().duration_since(time).unwrap().as_millis());
    // stop the program from exiting
    //std::io::stdin().read_line(&mut String::new()).unwrap();
    println!("gc: {:?}", ctx.memory.gc.memory_swept);*/
}