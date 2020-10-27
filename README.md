SM Motion Photo
===============

[![Build Status](https://travis-ci.org/g0ddest/sm_motion_photo.svg?branch=master)](https://travis-ci.org/g0ddest/sm_motion_photo)
[![codecov](https://codecov.io/gh/g0ddest/sm_motion_photo/branch/master/graph/badge.svg)](https://codecov.io/gh/g0ddest/sm_motion_photo)
[![docs](https://docs.rs/sm_motion_photo/badge.svg)](https://docs.rs/sm_motion_photo/)
[![crates](https://img.shields.io/crates/v/sm_motion_photo.svg)](https://crates.io/crates/sm_motion_photo)

This crate provides functions for extracting video from [Motion Photos](https://www.samsung.com/global/galaxy/what-is/motion-photo/) and getting meta-information from the video. It is a feature of Samsung phones, a JPEG file with a video file embedded.

This feature is available on Galaxy S20, S20+, S20 Ultra, Z Flip, Note10, Note10+, S10e, S10, S10+, Fold, Note9, S9, S9+, Note8, S8, S8+, S7, and S7 edge.

Supports photos saved in JPEG and [HEIF (HEIC)](https://ru.wikipedia.org/wiki/HEIF) format.

## Usage
```rust
use std::fs::File;
use sm_motion_photo::SmMotion;

// open file
let photo_file = File::open("photo.jpg").unwrap();
let mut sm = SmMotion::with(&photo_file).unwrap();
println!("JPEG file contains video? {:?}", sm.has_video());
let mut video_file = File::create("video.mp4").unwrap();
// dump mp4 from jpeg
sm.dump_video_file(&mut video_file).unwrap();
// get video duration (no dump needed)
println!("{:?}", sm.get_video_file_duration());
// get MP4 file context
println!("{:?}", sm.find_video_context());
// You can also save index and use it afterwards
let mut sm_cached = SmMotion::with_precalculated(&photo_file, 3366251).unwrap();
println!("{:?}", sm_cached.get_video_file_duration());
```