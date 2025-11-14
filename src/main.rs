use std::path::PathBuf;

use doc_simfinder::{analysis::score_file, config::Config, file_walker::walk_from_root};

fn main() {
    let config = Config {
        search_path: PathBuf::from(r"D:\Noah\04_CodingStuff\RUST\doc-simfinder\testdata"),
        max_search_depth: 2,
        num_threads: 1,
        file_exts: vec![".txt".to_string(), ".md".to_string()],
        output_file: None,
        query: "purus non turpis pellentesque porttitor".to_string(),
        window_size: 500,
        threshold: Some(0.1_f64), // Check if really needed
        ..Default::default()
    };

    let walk_result = walk_from_root(&config);

    match walk_result {
        Ok(res) => {
            let res = score_file(res.files[2].as_path(), &config);
            println!("{}", res.unwrap());
        }
        Err(err) => println!("{}", err),
    };
}
