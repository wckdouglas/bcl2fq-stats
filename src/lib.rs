pub mod cli;
pub mod distance;
pub mod models;

use crate::cli::{Command, Parser};
use crate::distance::hamming_distance;
use crate::models::{Bcl2FqStats, ConversionResult};
use log::info;
use models::UnknownBarcode;
use serde_json::from_str;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const HEADER: &str = "sq_id\tbarcode\tread_count\tpossible_sq_index";
const MAX_COUNT_UNDETERMINED: usize = 10;

/// Parsing ConversionResult (per lane) from the bcl2fastq json file
///
/// # Arguments
/// - conversion_result: ConversionResult object
/// - barcode_counter: a hash map collecting the count for each sample
/// - barcode_list: a hash map collecting the conversion between sample id and barcode
///
/// # Returns
/// - None
fn collect_lane_barcode_count(
    conversion_result: &ConversionResult,
    barcode_counter: &mut HashMap<String, u64>,
    barcode_list: &mut HashMap<String, String>,
) -> Result<(), String> {
    info!(
        "Collecting sample barcode count from lane {}",
        conversion_result.LaneNumber
    );
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

fn collect_lane_undetermined_barcode(
    unknown_barcode_lane: &UnknownBarcode,
    undetermined_barcode_counter: &mut HashMap<String, u64>,
) {
    info!(
        "Collecting undetermined barcode from lane {}",
        unknown_barcode_lane.Lane
    );
    for (undetermined_barcode, read_count) in unknown_barcode_lane.Barcodes.iter() {
        match undetermined_barcode_counter.get(undetermined_barcode) {
            Some(count) => {
                undetermined_barcode_counter
                    .insert(undetermined_barcode.clone(), count + read_count);
            }
            None => {
                undetermined_barcode_counter.insert(undetermined_barcode.clone(), *read_count);
            }
        };
    }
}

fn print_undetermined_barcode(
    undetermined_barcode_counter: &mut HashMap<String, u64>,
    barcode_list: &HashMap<String, String>,
    max_distance: &u8,
) -> Result<(), String> {
    let mut sorted_undetermined_barcode_count = Vec::from_iter(undetermined_barcode_counter);
    sorted_undetermined_barcode_count.sort_by(|(_, a), (_, b)| b.cmp(&a));

    for it in sorted_undetermined_barcode_count.iter().enumerate() {
        let (i, (undetermined_barcode, barcode_count)) = it;
        let list_of_possible_barcodes: Vec<String> = barcode_list
            .iter()
            .map(|(barcode_id, barcode)| {
                (
                    barcode_id,
                    hamming_distance(barcode.as_bytes(), undetermined_barcode.as_bytes()).unwrap(),
                )
            })
            .filter(|(_barcode_id, score)| score < &max_distance)
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

/// printing out the barcode count
///
/// # Arguments
/// - barcode_counter: hash map of sample id/count
/// - barcode_list: hash map of sample id/barcode
fn print_barcode_count(
    barcode_counter: &HashMap<String, u64>,
    barcode_list: &HashMap<String, String>,
) -> Result<(), String> {
    // print out the barcode counts
    let mut sorted_barcode_count = Vec::from_iter(barcode_counter);
    // sort the barcode by counts
    sorted_barcode_count.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
    for (sample_id, barcode_count) in sorted_barcode_count.iter() {
        let barcode: &String = barcode_list
            .get(&(*sample_id).clone())
            .ok_or(format!("No barcode collected for {}", sample_id))?;
        println!("{}\t{}\t{}\t", sample_id, barcode, barcode_count);
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
    let _ = &bcl2fastq_stats
        .ConversionResults
        .into_iter()
        .map(|conversion_result| {
            collect_lane_barcode_count(&conversion_result, &mut barcode_counter, &mut barcode_list)
        })
        .collect::<Vec<Result<(), String>>>();

    print_barcode_count(&barcode_counter, &barcode_list)?;

    let mut undetermined_barcode_counter: HashMap<String, u64> = HashMap::new();

    let _ = bcl2fastq_stats
        .UnknownBarcodes
        .iter()
        .map(|unknown_barcode_for_lane| {
            collect_lane_undetermined_barcode(
                unknown_barcode_for_lane,
                &mut undetermined_barcode_counter,
            )
        })
        .collect::<Vec<_>>();
    print_undetermined_barcode(
        &mut undetermined_barcode_counter,
        &barcode_list,
        &args.max_distance,
    )?;

    Ok(())
}
