/// Models for deserializing bcl2fastq Stats.json
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadInfo {
    pub Number: u8,
    pub NumCycles: u32,
    pub IsIndexedRead: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadInfosForLane {
    pub LaneNumber: u8,
    pub ReadInfos: Vec<ReadInfo>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IndexMetric {
    pub IndexSequence: String,
    pub MismatchCounts: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadMetric {
    pub ReadNumber: u8,
    pub Yield: u64,
    pub YieldQ30: u64,
    pub QualityScoreSum: u64,
    pub TrimmedBases: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DemuxResult {
    pub SampleId: String,
    pub SampleName: String,
    pub IndexMetrics: Vec<IndexMetric>,
    pub NumberReads: u64,
    pub Yield: u64,
    pub ReadMetrics: Vec<ReadMetric>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConversionResult {
    pub LaneNumber: u8,
    pub TotalClustersRaw: u64,
    pub TotalClustersPF: u64,
    pub Yield: u64,
    pub DemuxResults: Vec<DemuxResult>,
}

#[derive(Deserialize, Serialize)]
pub struct UnknownBarcode {
    pub Lane: u8,
    pub Barcodes: HashMap<String, u64>,
}

#[derive(Deserialize, Serialize)]
pub struct Bcl2FqStats {
    pub Flowcell: String,
    pub RunNumber: u8,
    pub RunId: String,
    pub ReadInfosForLanes: Vec<ReadInfosForLane>,
    pub ConversionResults: Vec<ConversionResult>,
    pub UnknownBarcodes: Vec<UnknownBarcode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_model() {
        let filename = "data/Stats.json";
        let mut file = File::open(filename).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let bcl2fastq_stats: Bcl2FqStats = from_str(&data).unwrap();
        assert_eq!(bcl2fastq_stats.ReadInfosForLanes[0].LaneNumber, 1);

        let first_barcode = &bcl2fastq_stats.ConversionResults[0].DemuxResults[0];
        assert_eq!(&first_barcode.SampleId, "BARCODE1");
        assert_eq!(&first_barcode.SampleName, "BARCODE1");
        assert_eq!(
            &first_barcode.IndexMetrics[0].IndexSequence,
            "CCGCGGTT+AGCGCTAG"
        );
    }
}
