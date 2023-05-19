# pgv-cli
CLI tool for encoding PGV video files

## Installation

```
cargo install pgv-cli
```

## Usage

Takes as input a folder containing named PNG files (as 001.png, 002.png, etc) and an optional WAV audio track, and produces an encoded PGV video file

```
Usage: pgv-cli.exe [OPTIONS] -i <FRAMEPATH> -n <NUMFRAMES> -f <FPS> -o <OUTPATH>

Options:
  -i <FRAMEPATH>
  -n <NUMFRAMES>
  -f <FPS>
  -a <AUDIOPATH>
  -o <OUTPATH>
  -h, --help          Print help
  -V, --version       Print version
```