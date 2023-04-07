pub mod cli;
pub mod distance;
pub mod models;

use crate::cli::{Command, Parser};
use crate::distance::hamming_distance;
use crate::models::{Bcl2FqStats, ConversionResult, DemuxResult};
use log::info;
use serde_json::from_str;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const HEADER: &str = "sq_id\tbarcode\tread_count\tpossible_sq_index";
const MAX_COUNT_UNDETERMINED: usize = 10;

fn collect_lane_barcode_count(
    conversion_result: &ConversionResult,
    barcode_counter: &mut HashMap<String, u64>,
    barcode_list: &mut HashMap<String, String>,
) -> Result<(), String> {
    for demux_sample in &conversion_result.DemuxResults {
        let sample_id: &String = &demux_sample.SampleId;
        let barcode: &String = &demux_sample.IndexMetrics[0].IndexSequence;
        let read_count: u64 = demux_sample.NumberReads;

        match barcode_counter.get(sample_id) {
            Some(count) => {
                barcode_counter.insert(sample_id.clone(), count + read_count);
            }
            None => {
                barcode_counter.insert(sample_id.clone(), read_count);
            }
        };

        barcode_list.insert(sample_id.clone(), barcode.to_string());
    }
    Ok(())
}

pub fn run() -> Result<(), String> {
    // Read in cli arguments
    let args = Command::parse();

    info!("Reading {}", &args.json_file);
    // Read json file content as string
    // TODO: can we stream it?
    let mut file = File::open(args.json_file).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    println!("{}", HEADER);
    let mut barcode_list: HashMap<String, String> = HashMap::new();
    let mut barcode_counter: HashMap<String, u64> = HashMap::new();
    let bcl2fastq_stats: Bcl2FqStats = from_str(&data).unwrap();

    // parse demux result from all lanes
    for conversion_result in &bcl2fastq_stats.ConversionResults {
        // this is running for each lane
        collect_lane_barcode_count(conversion_result, &mut barcode_counter, &mut barcode_list)?;
    }

    for (sample_id, barcode_count) in barcode_counter.iter() {
        let barcode: &String = barcode_list
            .get(sample_id)
            .ok_or("No barcode collected".to_string())?;
        println!("{}\t{}\t{}\t{}", sample_id, barcode, barcode_count, "");
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
