# bcl2fq-stats #

Illumina sequencing data is always demultiplexed with [Bcl2fastq](https://support.illumina.com/sequencing/sequencing_software/bcl2fastq-conversion-software.html),
but it's not always easy to get a table of read counts from the demultiplexed data. 
`bcl2fq-stats` is designed to give a quick overview of read count distribution over the given indices, and
identify potential index mismatches from the undetermined read counts.

## Usage ##

The program takes `Stats.json` in the bcl2fastq output folder as input:

```
bcl2fq-stats --json-file data/Stats.json
```

## Installation ## 

```
git clone https://github.com/wckdouglas/bcl2fq-stats.git
cd bcl2fq-stats
cargo install --path .
```

or using docker:

```
docker pull ghcr.io/wckdouglas/bcl2fq-stats:main
```


