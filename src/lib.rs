pub mod cli;
pub mod distance;
pub mod models;

use crate::cli::{Command, Parser};
use crate::distance::hamming_distance;
use crate::models::Bcl2FqStats;
use log::info;
use serde_json::from_str;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const HEADER: &str = "sq_id\tbarcode\tread_count\tpossible_sq_index";
const MAX_COUNT_UNDETERMINED: usize = 10;

pub fn run() -> Result<(), String> {
    let args = Command::parse();

    info!("Reading {}", &args.json_file);
    let mut file = File::open(args.json_file).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    println!("{}", HEADER);
    let mut barcode_list: HashMap<String, String> = HashMap::new();
    let bcl2fastq_stats: Bcl2FqStats = from_str(&data).unwrap();
    for sample in &bcl2fastq_stats.ConversionResults[0].DemuxResults {
        let barcode = &sample.IndexMetrics[0].IndexSequence;
        barcode_list.insert(sample.SampleId.clone(), barcode.to_string());
        println!(
            "{}\t{}\t{}\t{}",
            sample.SampleId, barcode, sample.NumberReads, ""
        );
    }

    for it in bcl2fastq_stats.UnknownBarcodes[0]
        .Barcodes
        .iter()
        .enumerate()
    {
        let (i, (undetermined_barcode, barcode_count)) = it;
        let list_of_possible_barcodes: Vec<String> = barcode_list
            .iter()
            .map(|(barcode_id, barcode)| {
                (
                    barcode_id,
                    hamming_distance(barcode.as_bytes(), undetermined_barcode.as_bytes()).unwrap(),
                )
            })
            .filter(|(_barcode_id, score)| score < &args.max_distance)
            .map(|(barcode_id, _score)| barcode_id.clone())
            .collect();
        let possible_barcodes: String = list_of_possible_barcodes.join(",");

        println!(
            "Undetermined\t{}\t{}\t{}",
            undetermined_barcode, barcode_count, possible_barcodes
        );
        if i == MAX_COUNT_UNDETERMINED - 1 {
            return Ok(());
        }
    }
    Ok(())
}
