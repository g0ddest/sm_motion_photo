#[cfg(test)]
mod tests {
    use mp4parse::TrackType::Video;
    use sm_motion_photo::SmMotion;
    use std::env;
    use std::fs::File;

    const VIDEO_INDEX: usize = 3366251;
    const VIDEO_INDEX_HEIC: usize = 2749488;
    const VIDEO_DURATION: u64 = 2932;
    const TMP_VIDEO: &str = "tests/tmp/foo.mp4";
    const TMP_VIDEO_HEIC: &str = "tests/tmp/foo.mp4";
    // for parallel test execution
    const TMP_VIDEO_: &str = "tests/tmp/foo_heic_.mp4";

    fn get_photo_file() -> File {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        File::open(format!("{}/{}", dir, "tests/data/photo.jpg")).unwrap()
    }

    fn get_photo_file_heic() -> File {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        File::open(format!("{}/{}", dir, "tests/data/photo.heic")).unwrap()
    }

    fn get_wrong_photo_file() -> File {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        File::open(format!("{}/{}", dir, "tests/data/blank.jpg")).unwrap()
    }

    fn get_empty_file() -> File {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        File::open(format!("{}/{}", dir, "tests/data/empty")).unwrap()
    }

    fn create_video_file(file_path: &str) -> File {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        File::create(format!("{}/{}", dir, file_path)).unwrap()
    }

    #[test]
    fn test_search_index() {
        let mut sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        match sm_motion.find_video_index() {
            Err(e) => panic!(e),
            _result => {}
        };

        match sm_motion.video_index {
            Some(size) => assert_eq!(size, VIDEO_INDEX),
            None => panic!("No result"),
        };
    }

    #[test]
    fn test_search_index_heic() {
        let mut sm_motion = match SmMotion::with(&get_photo_file_heic()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        match sm_motion.find_video_index() {
            Err(e) => panic!(e),
            _result => {}
        };

        match sm_motion.video_index {
            Some(size) => assert_eq!(size, VIDEO_INDEX_HEIC),
            None => panic!("No result"),
        };
    }

    #[test]
    fn test_dump_video() {
        let sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };
        let mut file = create_video_file(TMP_VIDEO);
        let _ = sm_motion.dump_video_file(&mut file);
        let mut open_file = File::open(TMP_VIDEO).unwrap();
        let mut context = mp4parse::MediaContext::new();
        let _ = mp4parse::read_mp4(&mut open_file, &mut context);
        assert_eq!(context.tracks.len(), 1);
    }

    #[test]
    fn test_dump_video_heic() {
        let sm_motion = match SmMotion::with(&get_photo_file_heic()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };
        let mut file = create_video_file(TMP_VIDEO_HEIC);
        let _ = sm_motion.dump_video_file(&mut file);
        let mut open_file = File::open(TMP_VIDEO_HEIC).unwrap();
        let mut context = mp4parse::MediaContext::new();
        let _ = mp4parse::read_mp4(&mut open_file, &mut context);
        assert_ne!(context.tracks.len(), 0);
    }

    #[test]
    fn test_meta() {
        let sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        match sm_motion.find_video_context() {
            Some(context) => {
                assert_eq!(context.tracks.len(), 1);
                assert_eq!(context.tracks[0].track_type, Video);
            }
            None => panic!("No media context found"),
        };
    }

    #[test]
    fn test_duration() {
        let sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        match sm_motion.get_video_file_duration() {
            Some(duration) => assert_eq!(duration, VIDEO_DURATION),
            None => panic!("Not found duration"),
        }
    }

    #[test]
    fn test_duration_cached_index() {
        let sm_motion = match SmMotion::with_precalculated(&get_photo_file(), VIDEO_INDEX) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        match sm_motion.get_video_file_duration() {
            Some(duration) => assert_eq!(duration, VIDEO_DURATION),
            None => panic!("Not found duration"),
        }
    }

    #[test]
    fn test_check() {
        let sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        assert_eq!(sm_motion.has_video(), true)
    }

    #[test]
    fn test_fail_open_video() {
        let _ = match SmMotion::with(&get_wrong_photo_file()) {
            Some(sm) => {
                assert!(sm.find_video_context().is_none());
            }
            None => panic!("Not created motion"),
        };
    }

    #[test]
    fn test_wrong_photo_no_video() {
        let _ = match SmMotion::with(&get_wrong_photo_file()) {
            Some(sm) => {
                assert_eq!(sm.has_video(), false);
            }
            None => panic!("Not created motion"),
        };
    }

    #[test]
    fn test_fail_dump_video() {
        let _ = match SmMotion::with(&get_wrong_photo_file()) {
            Some(sm) => {
                assert!(sm
                    .dump_video_file(&mut create_video_file(TMP_VIDEO_))
                    .is_err());
            }
            None => panic!("Not created motion"),
        };
    }

    #[test]
    pub fn test_mmap() {
        let _ = match SmMotion::with(&get_empty_file()) {
            Some(_) => panic!("Should not mmap"),
            None => {}
        };

        let _ = match SmMotion::with_precalculated(&get_empty_file(), 0) {
            Some(_) => panic!("Should not mmap"),
            None => {}
        };
    }

    #[test]
    fn test_fail_dump_video_file_write() {
        let _ = match SmMotion::with(&get_wrong_photo_file()) {
            Some(sm) => {
                assert!(sm.dump_video_file(&mut get_empty_file()).is_err());
            }
            None => panic!("Not created motion"),
        };
    }
}
