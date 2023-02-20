# Rust CLI Term Frequency Analyzer Project

TFA is a Rust command-line interface (CLI) tool for finding the frequency of words in files. This tool provides a simple and efficient way to analyze the frequency of words in a given text file.
Getting Started

To get started with TFA, first, clone the repository and navigate to the root directory of the project. Then, run the following command to build the project:

```sh

cargo build --release
```
After the build is successful, you can move release binary to your bin folder adn run the tool using the following command:

```sh
hapax [OPTIONS] <COMMAND>
```
## Usage

TFA provides the following options and commands:
### Options:
```sh
  -o, --output <OUTPUT>  type of the output file: json/csv/txt [default: json]
  -p, --path <PATH>      path to the output folder [default: ./]
  -h, --help             Print help
  -V, --version          Print version
```
## Commands:
  tf    provides term frequency
  help  Print this message or the help of the given subcommand(s)

To get the frequency of words in a text file, you can use the tf command followed by the path to the file. For example:

```sh

hapax tf -f /path/to/file.txt
```
This will print the frequency of each word in the file to the console.

You can also specify the output format and the output folder using the -o and -p options, respectively. For example:

```sh

hapax -o csv -p /output/folder/ tf -f /path/to/file.txt 
```
This will save the frequency of words in the file to a CSV file in the specified output folder.

## Output Formats:

TFA supports three output formats: txt, json, and csv.
The tfa tool currently supports three output formats: JSON, CSV, and plain text.

- JSON: The output will be a JSON file containing the file name, length, and term frequency in a key-value format.

- CSV: The output will be a CSV file containing the file name, length, and term frequency in a comma-separated format.

- Plain text: The output will be a plain text file containing the file name, length, and term frequency in a table format.

Below are the examples of how each format looks like for the same input file:
### TXT Output


```sh
FILE: ga.srt           LENGTH: 10094

WORD:                  FREQUENCY:            PERCENT:
-----------------------------------------------------
you                    387                    3.83%
i                      370                    3.67%
the                    282                    2.79%
to                     228                    2.26%
a                      219                    2.17%
and                    142                    1.41%
of                     135                    1.34%
it                     129                    1.28%
what                   120                    1.19%
is                     120                    1.19%
...
```

### JSON Output

```sh
json

{
  "file_name": "ga.srt",
  "length": 10094,
  "term_frequency": {
    "want": [
      17,
      0.16841688131563304
    ],
    "dude": [
      8,
      0.07925500297206262
    ],
    "discussion": [
      1,
      0.009906875371507827
    ],
    "gasps": [
      12,
      0.11888250445809392
    ],
	}
    ...
 }

```
### CSV Output

```sh
css

FILE,LENGTH
ga.srt,10094

WORD,FREQUENCY,PERCENT
you,387,3.8339607687735286
i,370,3.6655438874578956
what,120,1.1888250445809392
that,109,1.079849415494353
me,106,1.0501287893798297
my,100,0.9906875371507826
all,95,0.9411531602932435
this,81,0.802456905092134
are,74,0.7331087774915792
your,72,0.7132950267485635
...
```
