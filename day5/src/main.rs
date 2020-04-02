use intcode;

#[tokio::main]
async fn main() {
    let start_time = std::time::Instant::now();

    let program = intcode::load_program("day5/input.txt").unwrap_or_else(|err| {
        println!("Could not load input file!\n{:?}", err);
        std::process::exit(1);
    });

    let program_clone = program.clone();
    let outputs_part_1 = tokio::spawn(async move {
        intcode::run_async_computer(&program_clone, &[1]).await
    });
    let outputs_part_2 = tokio::spawn(async move {
        intcode::run_async_computer(&program, &[5]).await
    });

    println!(
        "Part 1: {}\nPart 2: {}\nTime: {}us",
        outputs_part_1.await.unwrap().last().unwrap(),
        outputs_part_2.await.unwrap().first().unwrap(),
        start_time.elapsed().as_micros()
    );
}
