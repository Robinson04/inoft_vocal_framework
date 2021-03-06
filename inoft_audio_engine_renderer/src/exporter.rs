use lame::Lame;
use claxon::FlacReader;
use std::fs::File;
use std::time::Instant;
use std::path::Path;
use std::io::{Write, Read};
use crate::models::ReceivedTargetSpec;

use tokio;
use tokio::prelude::*;
use tokio::time::{Duration};
use std::collections::HashMap;
use std::error::Error;

use serde::Deserialize;
use serde::Serialize;
use std::mem::{size_of, size_of_val};
use reqwest::Url;

#[derive(Debug, Deserialize, Serialize)]
struct GeneratePresignedUploadUrlRequestData {
    filename: String,
    filesize: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct Fields {
    acl: String,
    key: String,
    policy: String,
    #[serde(default, rename="x-amz-algorithm")] x_amz_algorithm: String,
    #[serde(default, rename="x-amz-credential")] x_amz_credential: String,
    #[serde(default, rename="x-amz-date")] x_amz_date: String,
    #[serde(default, rename="x-amz-signature")] x_amz_signature: String,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    fields: Option<Fields>,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct GeneratePresignedUploadUrlResponse {
    success: bool,
    errorKey: Option<String>,
    expectedS3FileUrl: Option<String>,
    data: Option<ResponseData>
}


pub async fn post_mp3_buffer_to_s3_with_presigned_url(mp3_buffer: Vec<u8>, presigned_url_response_data: GeneratePresignedUploadUrlResponse) {
    let jsonified_data = presigned_url_response_data.data.expect("Error in retrieving the data item from the response data");
    let s3_target_url = jsonified_data.url;
    let s3_fields = jsonified_data.fields.unwrap();

    let mp3_file_part = reqwest::multipart::Part::bytes(mp3_buffer)
        .mime_str("application/octet-stream").unwrap();

    let form = reqwest::multipart::Form::new()
        .text("key", s3_fields.key)
        .text("acl", s3_fields.acl)
        .text("policy", s3_fields.policy)
        .text("x-amz-algorithm", s3_fields.x_amz_algorithm)
        .text("x-amz-credential", s3_fields.x_amz_credential)
        .text("x-amz-date", s3_fields.x_amz_date)
        .text("x-amz-signature", s3_fields.x_amz_signature)
        .part("file", mp3_file_part);

    let client = reqwest::ClientBuilder::new().build().unwrap();
    let submission_response = client
        .post(Url::parse(&*s3_target_url).unwrap())
        .multipart(form)
        .send().await.unwrap()
        .text().await.unwrap();

    println!("S3 response:\n {}", submission_response);
}


pub async fn get_upload_url(filename: String, filesize: usize) -> Result<(Option<GeneratePresignedUploadUrlResponse>), Box<dyn Error + Send + Sync>> {
    let mut jsonified_output_data: Option<GeneratePresignedUploadUrlResponse> = None;
    let client = reqwest::ClientBuilder::new().build().unwrap();

    let request_data = GeneratePresignedUploadUrlRequestData { filename, filesize };
    let request_jsonified_data = serde_json::to_string(&request_data).unwrap();
    // todo: make account name/id and project name/id dynamic
    let request = client
        .post(Url::parse("http://127.0.0.1:5000/api/v1/@robinsonlabourdette/livetiktok/resources/project-audio-files/generate-presigned-upload-url").unwrap())
        .header("content-type", "application/json")
        .body(reqwest::Body::from(request_jsonified_data));

    match request.send().await {
        Ok(res) => {
            let response_text_content = res.text().await.unwrap();
            jsonified_output_data = Some(serde_json::from_str(&*response_text_content).expect("Error in json"));
        },
        Err(err) => println!("Error: {}", err),
    }
    Ok((jsonified_output_data))
}

// flac_song: &std::fs::File
pub async fn from_flac_to_mp3() -> Vec<u8> {
    let file = File::open("F:/Sons utiles/compressed-music.flac").expect("Error opening file");
    let mut flac_reader = FlacReader::new(file).expect("FlacReader error");
    let mut lame = Lame::new().expect("Coudn't create Lame");

    lame.set_channels(1).expect("Couldn't set num channels");
    /*lame.set_sample_rate(flac_reader.streaminfo().sample_rate as _)
        .expect("Couldn't set up sample rate");
     */
    lame.set_sample_rate(16000 as u32).expect("Couldn't set up sample rate");
    lame.set_channels(1).expect("Coudn't set up channels");
    lame.set_kilobitrate(48).expect("Coudn't set up kilobitrate");

    let mut left_samples: Vec<i16> = Vec::new();
    let mut right_samples: Vec<i16> = Vec::new();

    let mut frame_reader = flac_reader.blocks();
    loop {
        let frame: Vec<i32> = Vec::new();
        let result = frame_reader
            .read_next_or_eof(frame)
            .expect("error to read frame");

        match result {
            None => {
                break;
            }
            Some(block) => {
                let iter = block.stereo_samples();
                for (left, right) in iter {
                    left_samples.push(left as _);
                    right_samples.push(right as _);
                }
            }
        };
    }

    let num_samples = left_samples.len() as f64;
    let mp3_buffer_size = (1.25 * num_samples + 7200.0) as usize;

    let mut mp3_buffer = vec![0; mp3_buffer_size];

    lame.set_quality(4).expect("Set quality error");
    lame.init_params().expect("init parametrs error");

    println!("lame start");
    let start = Instant::now();
    let _ = lame.encode(
        left_samples.as_slice(),
        right_samples.as_slice(),
        mp3_buffer.as_mut_slice(),
    );
    println!("\nFinished lame.\n  --execution_time:{}ms", start.elapsed().as_millis());

    // tokio::spawn(get_upload_url());
    /*let mut rt = tokio::runtime::Runtime::new().unwrap();
    let future = get_upload_url();
    rt.block_on(future);
     */

    mp3_buffer
}

pub fn from_samples_to_mono_mp3(samples: Vec<i16>, target_spec: &ReceivedTargetSpec) -> Vec<u8> {
    let start = Instant::now();

    let mut lame = Lame::new().expect("Coudn't create Lame");
    lame.set_channels(1).expect("Couldn't set num channels");
    lame.set_sample_rate(target_spec.sample_rate as u32).expect("Couldn't set up sample rate");
    lame.set_kilobitrate(48).expect("Coudn't set up kilobitrate");
    // todo: make kilobitrate parametrable
    lame.set_quality(9).expect("Set quality error");
    // We set the quality to the 'worst' and fastest quality possible. With the formats used
    // by the inoft_vocal_framework, we do not hear the difference between the qualities.
    lame.init_params().expect("init parameters error");

    let num_samples = samples.len() as f64;
    let mp3_buffer_size = (num_samples + 7200.0) as usize;
    let mut mp3_buffer = vec![0; mp3_buffer_size];
    // todo: find a way to have the size of the mp3_buffer exactly to the num of samples that lame
    //  will encode. Because, currently, the num_samples of the source samples is way too big,
    //  and if we initialize the vector as an empty vector, sometime lame will correctly fill it
    //  to the appropriate size, and other times, Windows will cause raise a STATUS_ACCESS_VIOLATION
    //  error. So, currently, the mp3_buffer will always be way too big, and have a ton of trailing
    //  zeros, which we remove right after encoding with lame, by using a simple loop with a step size.

    let samples_slice = samples.as_slice();
    let _ = lame.encode(
        samples_slice,
        samples_slice,
        mp3_buffer.as_mut_slice(),
    );

    let precision_10ms_samples_step = (target_spec.sample_rate / 100) as usize;
    // We need a custom sample_step to iterate over the mp3_buffer, because if we iterate over every single sample
    // in the mp3_buffer we would waste a lot of calculation, for a granularity level that's not worth the cost.
    // A divider of 100 on the sample_rate give us a precision of 10ms, a divider of 1000 would be 1ms, and 10 would be 100ms.
    println!(
        "\nRemoving trailing empty data from the mp3_buffer.\
        \n  --precision_samples_step:{}", precision_10ms_samples_step
    );
    let start_removing_trailing_empty_data = Instant::now();
    let mut mp3_buffer_inverted_index = mp3_buffer.len() - 1;
    while mp3_buffer_inverted_index > 0 {
        if mp3_buffer.get(mp3_buffer_inverted_index).unwrap() > &u8::MIN {
            mp3_buffer = mp3_buffer[0..mp3_buffer_inverted_index].to_owned();
            break;
        }
        mp3_buffer_inverted_index -= precision_10ms_samples_step;
    }
    println!("Finished removing of trailing empty data from the mp3_buffer.\n  --execution_time:{}ms", start_removing_trailing_empty_data.elapsed().as_millis());

    /*tokio::runtime::Handle::try_current()
        // tokio::spawn(get_upload_url());
    }*/

    /*
    let mut rt2 = tokio::runtime::Runtime::new().unwrap();
    let future = get_upload_url();
    rt2.block_on(future);
     */
    // tokio::spawn(get_upload_url());

    println!("\nFinished MP3 conversion using Lame.\n  --execution_time:{}ms", start.elapsed().as_millis());
    mp3_buffer
}

pub fn write_mp3_buffer_to_file(mp3_buffer: Vec<u8>, filepath: &str) {
    let path = Path::new(filepath);
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {:?}: {}", path, why),
        Ok(file) => file,
    };
    match file.write_all(mp3_buffer.as_slice()) {
        Err(why) => panic!("couldn't write to {:?}: {}", path, why),
        Ok(_) => println!("successfully wrote to {:?}", path),
    }
}
