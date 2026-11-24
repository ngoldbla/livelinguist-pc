fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let options = match ll_eval::EvalOptions::parse(&args) {
        Ok(options) => options,
        Err(error) => {
            let message = error.to_string();
            if message.starts_with("invalid argument: ll-eval") {
                println!("{}", message.trim_start_matches("invalid argument: "));
                std::process::exit(0);
            }
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    if options.asr_wer.is_some() {
        match ll_eval::run_asr_wer(&options) {
            Ok(summary) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
                println!("wrote {}", options.out.display());
                return;
            }
            Err(error) => {
                eprintln!("ll-eval failed: {error}");
                std::process::exit(1);
            }
        }
    }
    if let Some(bench) = options.bench {
        match ll_eval::bench::run(&options, bench) {
            Ok(summary) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).unwrap_or_default()
                );
                println!("wrote {}", options.out.display());
                return;
            }
            Err(error) => {
                eprintln!("ll-eval failed: {error}");
                std::process::exit(1);
            }
        }
    }
    match ll_eval::run(&options) {
        Ok(summary) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).unwrap_or_default()
            );
            println!("wrote {}", options.out.display());
        }
        Err(error) => {
            eprintln!("ll-eval failed: {error}");
            std::process::exit(1);
        }
    }
}
