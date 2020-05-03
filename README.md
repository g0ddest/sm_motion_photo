SM Motion Photo
===============

[![Build Status](https://travis-ci.org/g0ddest/sm_motion_photo.svg?branch=master)](https://travis-ci.org/g0ddest/sm_motion_photo)
[![codecov](https://codecov.io/gh/g0ddest/sm_motion_photo/branch/master/graph/badge.svg)](https://codecov.io/gh/g0ddest/sm_motion_photo)

This crate can be used to extract Motion Photo taken on Samsung phone (if it provides such feature and this feature is turned on) and saves it in MP4. You can also check if the photo has video and get meta information from the video.

This feature is available on Galaxy S20, S20+, S20 Ultra, Z Flip, Note10, Note10+, S10e, S10, S10+, Fold, Note9, S9, S9+, Note8, S8, S8+, S7, and S7 edge.

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
```