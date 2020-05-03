#[cfg(test)]
mod tests {
    use sm_motion_photo::SmMotion;
    use std::fs::File;
    use std::env;
    use mp4parse::TrackType::Video;

    fn get_photo_file() -> File {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        File::open(format!("{}/{}", dir, "tests/data/photo.jpg")).unwrap()
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
            Some(size) => assert_eq!(size, 3366251),
            None => panic!("No result"),
        };
    }

    #[test]
    fn test_dump_video() {
        let mut sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };
        let file_path = "tests/tmp/foo.mp4";
        let mut file = File::create(file_path).unwrap();
        let _ = sm_motion.dump_video_file(&mut file);
        let mut open_file = File::open(file_path).unwrap();
        let mut context = mp4parse::MediaContext::new();
        let _ = mp4parse::read_mp4(&mut open_file, &mut context);
        assert_eq!(context.tracks.len(), 1);
    }

    #[test]
    fn test_meta() {
        let mut sm_motion = match SmMotion::with(&get_photo_file()) {
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
        let mut sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        match sm_motion.get_video_file_duration() {
            Some(duration) => assert_eq!(duration, 2932),
            None => panic!("Not found duration"),
        }
    }

    #[test]
    fn test_check() {
        let mut sm_motion = match SmMotion::with(&get_photo_file()) {
            Some(sm) => sm,
            None => panic!("Not created motion"),
        };

        assert_eq!(sm_motion.has_video(), true)
    }

}
