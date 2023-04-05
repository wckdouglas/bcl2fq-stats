use bcl2fq_stats::run;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let result = run();
    match result {
        Ok(_) => (),
        Err(err_string) => println!("{}", err_string),
    };
}
