pub mod cli;
pub mod distance;
pub mod models;
pub mod utils;

use crate::cli::{Command, Parser};
use crate::distance::hamming_distance;
use crate::models::{Bcl2FqStats, ConversionResult};
use crate::utils::sort_hashmap;
use log::info;
use models::UnknownBarcode;
use serde_json::from_str;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const HEADER: &str = "sq_id\tbarcode\tread_count\tpossible_origin_index";
const MAX_COUNT_UNDETERMINED: usize = 10;

/// Parsing ConversionResult (per lane) from the bcl2fastq json file
///
/// # Arguments
/// - `conversion_result`: ConversionResult object
/// - `barcode_counter`: a hash map collecting the count for each sample
/// - `barcode_list`: a hash map collecting the conversion between sample id and barcode
///
/// # Returns
/// - None
///
/// # Example
/// ```
/// use bcl2fq_stats::models::{ConversionResult, DemuxResult};
/// use std::collections::HashMap;
/// ```
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

/// Parsing the Undetermined barcode section
///
/// # Arguments
/// - unknown_barcode_lane: the unknown barcode object for each lane
/// - undetermined_barcode_counter: a hashmap collecting the count for each undetermined barcodes
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

/// Printing out the top count undetermined barcode, with their
/// possible mismatched originated-barcode ID
///
/// # Arguments
/// - `undetermined_barcode_counter`: a hashmap storing the undetermined barcode sequence and their
/// counts
/// - `barcode_list`: a barcode ID to barcode sequence hash map
/// - `max_distance`: how many mismatch to tolerate before calling a barcode match
fn print_undetermined_barcode(
    undetermined_barcode_counter: &mut HashMap<String, u64>,
    barcode_list: &HashMap<String, String>,
    max_distance: &u8,
) -> Result<(), String> {
    let sorted_undetermined_barcode_count = sort_hashmap(undetermined_barcode_counter)?;

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
            .filter(|(_barcode_id, score)| score < max_distance)
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
/// - `barcode_counter`: hash map of sample id/count
/// - `barcode_list`: hash map of sample id/barcode
fn print_barcode_count(
    barcode_counter: &HashMap<String, u64>,
    barcode_list: &HashMap<String, String>,
) -> Result<(), String> {
    // print out the barcode counts
    let sorted_barcode_count = sort_hashmap(barcode_counter)?;
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
    let mut file =
        File::open(args.json_file).map_err(|_| "The given json file is not found".to_string())?;
    let mut data = String::new();
    file.read_to_string(&mut data).map_err(|e| e.to_string())?;
    let bcl2fastq_stats: Bcl2FqStats =
        from_str(&data).map_err(|_| "Is this a bcl2fastq Stats.json file?".to_string())?;

    println!("{}", HEADER);
    let mut barcode_list: HashMap<String, String> = HashMap::new();
    let mut barcode_counter: HashMap<String, u64> = HashMap::new();

    // parse demux result from all lanes
    let _ = &bcl2fastq_stats
        .ConversionResults
        .into_iter()
        .map(|conversion_result| {
            collect_lane_barcode_count(&conversion_result, &mut barcode_counter, &mut barcode_list)
        })
        .collect::<Vec<Result<(), String>>>();

    // and print out the demuxed counts
    print_barcode_count(&barcode_counter, &barcode_list)?;

    // now look at undetermined barcode section
    let mut undetermined_barcode_counter: HashMap<String, u64> = HashMap::new();

    // first collecting the counts
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

    // and match with the known barcodes, and print them out
    print_undetermined_barcode(
        &mut undetermined_barcode_counter,
        &barcode_list,
        &args.max_distance,
    )?;

    Ok(())
}
