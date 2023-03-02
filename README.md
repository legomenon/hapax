# Rust CLI Term Frequency Analyzer 

Hapax is a Rust command-line interface (CLI) tool for finding the frequency of words in files. This tool provides a simple and efficient way to analyze the frequency of words in a given text file.
Getting Started

## Performance 
The app processed approximately **6 million** words per second, while without logging, the app processed around **10 million** words per second. This indicates that logging has a significant impact on the app's performance, as expected. Furthermore, the app's performance is influenced by various factors, such as the input data size, system processing speed,logging settings, lemmatization, junk words, ouput type. Therefore, users can optimize the app's performance by adjusting these factors based on their specific requirements.
The performance of the CLI app was evaluated on a Ryzen 4500U 6 core 4GHz processor.

## To get started 
first, clone the repository and navigate to the root directory of the project. Then, run the following command to build the project:

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
Commands:
  tf    provides term frequency TF
  tft   provides term frequency for all documents words comb
  
  Options:
  -o, --output <OUTPUT>  type of the output file: [json|csv|txt] [default: json]
  -p, --path <PATH>      path to the folder where result will be saved [default: ./]
      --log <LOG>        log filter [info|warn|error] [default: info]
  -j, --junk             if set skip junk words
  -l, --lemma            if set skip lemmatization
  -h, --help             Print help
  -V, --version          Print version
```
## Commands:
  ## tf    provides term frequency
  ```sh
  Options:
  -f, --file <FILE>...  files for parsing
  -d, --dir <DIR>       dir for parsing
  -h, --help            Print help
  ```

To get the frequency of words in a text file, you can use the tf command followed by the path to the file. For example:

```sh

hapax tf -f /path/to/file.txt
```
This will output the frequency of each word in the file to path output [default: ./].

You can also specify the output format and the output folder using the -o and -p options, respectively. For example:

```sh

hapax -o csv -p /output/folder/ tf -f /path/to/file.txt 
```
This will save the frequency of words in the file to a CSV file in the specified output folder.

## tft    provides term frequency for all words in directory combined in one output
```sh
Options:
  -d, --dir <DIR>  dir for parsing
  -h, --help       Print help
```

## Output Formats:

TFA supports three output formats: txt, json, and csv.
The tfa tool currently supports three output formats: JSON, CSV, and plain text.

- JSON: The output will be a JSON file containing the file name, length, and term frequency in a key-value format.

- CSV: The output will be a CSV file containing the file name, length, and term frequency in a comma-separated format.

- Plain text: The output will be a plain text file containing the file name, length, and term frequency in a table format.

Below are the examples of how each format looks like for the same input file:
### TXT Output
```sh
hapax -o text tf -f ./subtitles/ga.srt
```

```
FILE: ga.srt           LENGTH: 10094         TOTAL:293434

WORD:                  FREQUENCY:            PERCENT:
-----------------------------------------------------
you                    387                    3.83%
i                      370                    3.67%
the                    282                    2.79%
to                     228                    2.26%
a                      219                    2.17%
...
```

### JSON Output
```sh
hapax -o json tf -f ./subtitles/ga.srt
```
```
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
	}
    ...
 }

```
### CSV Output
```sh
hapax -o csv tf -f ./subtitles/ga.srt
```
```
css

FILE,LENGTH
ga.srt,10094

WORD,FREQUENCY,PERCENT
you,387,3.8339607687735286
i,370,3.6655438874578956
what,120,1.1888250445809392
that,109,1.079849415494353
...
```
## 10k film's subtitles files analysis: 
```sh
hapax_cli -o text tft -d films/
```
## 50 most popular words:
```sh
cat | head -n 53  ./result/total.stats.txt
```

total.stats.txt:
```
FILE: total            UNIQUE: 177480        TOTAL:20525222

WORD:                  FREQUENCY:            PERCENT:
-----------------------------------------------------
yeah                   248976                 1.21%
look                   187791                 0.91%
okay                   181334                 0.88%
tell                   157088                 0.77%
gonna                  146881                 0.72%
time                   142885                 0.70%
hey                    142145                 0.69%
fuck                   124926                 0.61%
thank                  112856                 0.55%
mean                   104192                 0.51%
love                   90873                  0.44%
please                 88952                  0.43%
guy                    87982                  0.43%
little                 87758                  0.43%
call                   85377                  0.42%
talk                   84739                  0.41%
sorry                  83662                  0.41%
leave                  82682                  0.40%
day                    76148                  0.37%
people                 75338                  0.37%
god                    73811                  0.36%
wait                   73303                  0.36%
help                   69718                  0.34%
try                    69097                  0.34%
stop                   66958                  0.33%
happen                 63814                  0.31%
hear                   63714                  0.31%
shit                   58637                  0.29%
sir                    58457                  0.28%
kill                   55687                  0.27%
win                    55246                  0.27%
life                   54589                  0.27%
night                  54383                  0.26%
girl                   52709                  0.26%
maybe                  52289                  0.25%
home                   51528                  0.25%
name                   50460                  0.25%
boy                    49420                  0.24%
live                   48654                  0.24%
kid                    47555                  0.23%
friend                 47449                  0.23%
stay                   46618                  0.23%
play                   44919                  0.22%
move                   44866                  0.22%
start                  44013                  0.21%
listen                 43242                  0.21%
...
```
## Lemmatization and stop words(junk words)
- List of lemmas was sourced from [here](https://github.com/skywind3000/lemma.en)
84466 words
- List of stop words was sourced from [here](https://countwordsfree.com/stopwords)
853 words

## License

Licensed under the MIT License

