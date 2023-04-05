import json

import click

MAX_COUNT_UNDETERMINED = 10
MAX_DIFF = 1


def hamming_distance(s1: str, s2: str) -> int:
    """
    perform hamming distance on two strings with identical length

    :param s1: first string
    :param s2: second string
    :return: hamming distance
    :rtype: int
    """
    hamming = 0
    if len(s1) != len(s2):
        raise ValuError("Wrong barcode extraction")

    for i, j in zip(s1, s2):
        if i != j:
            hamming += 1

    return hamming


@click.command()
@click.option("-j", "--json-file", required=True, help="bcl2fastq stats.json file path")
@click.option(
    "-d",
    "--max-distance",
    help="How many bases difference can we tolerate to say a undetermined barcode is a SQ",
    show_default=True,
    default=MAX_DIFF,
)
@click.option(
    "-c",
    "--max-count-undetermined",
    help="How many highest count of undetermined barcode to show",
    show_default=True,
    default=MAX_COUNT_UNDETERMINED,
)
def parse(json_file: str, max_distance: int = MAX_DIFF, max_count_undetermined: int = MAX_COUNT_UNDETERMINED):
    """
    parsing the json file to restructure as a table of count
    """
    print("sq_id\tbarcode\tread_count\tpossible_sq_index")
    sq_list = {}
    with open(json_file) as json_handle:
        json_result = json.load(json_handle)
        for sample in json_result["ConversionResults"][0]["DemuxResults"]:
            # parse the demultiplexed sample with known barcode
            barcode = sample["IndexMetrics"][0]["IndexSequence"]
            line = f"{sample['SampleId']}\t{barcode}\t{sample['NumberReads']}\t"
            sq_list[sample["SampleId"]] = barcode
            print(line)

        # parse the demultiplexed sample with unknown barcode
        for i, (undertermined_barcode, barcode_count) in enumerate(
            json_result["UnknownBarcodes"][0]["Barcodes"].items()
        ):
            # identify the possible sq index by hamming distance
            list_of_possible_barcodes = [
                sq for sq, bc in sq_list.items() if hamming_distance(bc, undertermined_barcode) <= max_distance
            ]
            possible_barcodes = ",".join(list_of_possible_barcodes)
            print(f"Undetermined\t{undertermined_barcode}\t{barcode_count}\t{possible_barcodes}")
            if i == max_count_undetermined - 1:
                break


if __name__ == "__main__":
    parse()
