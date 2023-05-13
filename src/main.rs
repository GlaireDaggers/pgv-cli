use std::{path::Path, fs::File};

use clap::Parser;
use pgv_rs::{enc::Encoder, def::{VideoFrame, ImageSlice}};
use image::{io::Reader as ImageReader};
use wav::{Header, WAV_FORMAT_PCM, BitDepth};

#[derive(Parser, Debug)]
#[command(author = "Hazel Stagner <glairedaggers@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Command line utility for encoding PGV video files", long_about = None)]
struct Args {
    #[arg(short = 'i')]
    framepath: String,
    
    #[arg(short = 'n')]
    numframes: u32,

    #[arg(short = 'f')]
    fps: u32,
    
    #[arg(short = 'a')]
    audiopath: Option<String>,

    #[arg(short = 'o')]
    outpath: String,
}

fn load_frame<Q: AsRef<Path>>(path: Q) -> VideoFrame {
    let src_img = ImageReader::open(path).unwrap().decode().unwrap().into_rgb8();
    
    let yuv_pixels: Vec<[u8;3]> = src_img.pixels().map(|rgb| {
        // https://en.wikipedia.org/wiki/YCbCr - "JPEG Conversion"
        let y = (0.299 * rgb.0[0] as f32) + (0.587 * rgb.0[1] as f32) + (0.114 * rgb.0[2] as f32);
        let u = 128.0 - (0.168736 * rgb.0[0] as f32) - (0.331264 * rgb.0[1] as f32) + (0.5 * rgb.0[2] as f32);
        let v = 128.0 + (0.5 * rgb.0[0] as f32) - (0.418688 * rgb.0[1] as f32) - (0.081312 * rgb.0[2] as f32);
        [y as u8, u as u8, v as u8]
    }).collect();

    // split into three planes
    let y_buffer: Vec<_> = yuv_pixels.iter().map(|x| x[0]).collect();
    let u_buffer: Vec<_> = yuv_pixels.iter().map(|x| x[1]).collect();
    let v_buffer: Vec<_> = yuv_pixels.iter().map(|x| x[2]).collect();

    let y_plane = ImageSlice::from_slice(src_img.width() as usize, src_img.height() as usize, &y_buffer);
    let u_plane = ImageSlice::from_slice(src_img.width() as usize, src_img.height() as usize, &u_buffer);
    let v_plane = ImageSlice::from_slice(src_img.width() as usize, src_img.height() as usize, &v_buffer);

    VideoFrame::from_planes(src_img.width() as usize, src_img.height() as usize, y_plane, u_plane, v_plane)
}

fn main() {
    let cli = Args::parse();

    // read first image from path
    let frame0 = load_frame(format!("{}/001.png", cli.framepath));

    let (audio_header, audio_data) = match &cli.audiopath {
        Some(audiopath) => {
            let mut inp_audio_file = File::open(audiopath).unwrap();
            wav::read(&mut inp_audio_file).unwrap()
        }
        None => {
            (Header::new(WAV_FORMAT_PCM, 2, 44100, 16),
                BitDepth::Sixteen(Vec::new()))
        }
    };

    let mut enc = Encoder::new(frame0.width, frame0.height, cli.fps, audio_header.sampling_rate, audio_header.channel_count as u32);

    // encode frames
    enc.encode_iframe(&frame0);
    println!("Encoded: 1 / {}", cli.numframes);

    for i in 1..cli.numframes {
        let framepath = format!("{}/{:0>3}.png", cli.framepath, i + 1);
        let frame = load_frame(framepath);

        if i % 30 == 0 {
            enc.encode_iframe(&frame);
        } else {
            enc.encode_pframe(&frame);
        }

        println!("Encoded: {} / {}", i + 1, cli.numframes);
    }

    // encode audio data
    if cli.audiopath.is_some() {
        let audio_data: Vec<i16> = match audio_data {
            wav::BitDepth::Eight(v) => {
                v.iter().map(|x| {
                    let f = (*x as f32 / 128.0) - 1.0;
                    (f * 32768.0) as i16
                }).collect()
            }
            wav::BitDepth::Sixteen(v) => {
                v
            }
            wav::BitDepth::ThirtyTwoFloat(v) => {
                v.iter().map(|x| {
                    (*x * 32768.0) as i16
                }).collect()
            }
            _ => {
                panic!("Not implemented")
            }
        };

        let mut audio_data_channels = Vec::new();

        for c in 0..audio_header.channel_count as usize {
            let ch: Vec<_> = audio_data.iter().enumerate().filter(|(idx, _)| {
                *idx > c && (*idx - c) % audio_header.channel_count as usize == 0
            }).collect();

            let ch_data: Vec<_> = ch.iter().map(|(_, v)| **v).collect();

            audio_data_channels.push(ch_data);
        }

        enc.encode_audio(audio_data_channels);
    }

    let mut outfile = File::create(cli.outpath).unwrap();
    enc.write(&mut outfile).unwrap();
}
