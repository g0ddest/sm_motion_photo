use boyer_moore_magiclen::{BMByte, BMByteSearchable};
use core::slice::Iter;
use memmap::Mmap;
use mp4parse::MediaContext;
use std::fs::File;
use std::io::Write;

// Implementing Bytes struct to use boyer moore search in [u8]
pub struct Bytes<'a> {
    bytes: &'a [u8],
}

impl<'a> Bytes<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Bytes { bytes }
    }
}

impl<'a> BMByteSearchable for Bytes<'a> {
    fn len(&self) -> usize {
        self.bytes.len()
    }
    fn value_at(&self, index: usize) -> u8 {
        self.bytes[index]
    }
    fn iter(&self) -> Iter<u8> {
        self.bytes.iter()
    }
}

/// Simple extractor of Motion Photo taken on Samsung phone
/// (if it provides such feature and this feature is turned on) and saves it in MP4.
/// It is available on Galaxy S20, S20+, S20 Ultra, Z Flip, Note10, Note10+, S10e, S10, S10+,
/// Fold, Note9, S9, S9+, Note8, S8, S8+, S7, and S7 edge.
///
/// Example of usage:
/// ```
/// use std::fs::File;
/// use sm_motion_photo::SmMotion;
///
/// // open file
/// let photo_file = File::open("tests/data/photo.jpg").unwrap();
/// let mut sm = SmMotion::with(&photo_file).unwrap();
/// println!("JPEG file contains video? {:?}", sm.has_video());
/// let mut video_file = File::create("tests/data/video.mp4").unwrap();
/// // dump mp4 from jpeg
/// sm.dump_video_file(&mut video_file).unwrap();
/// // get video duration (no dump needed)
/// println!("{:?}", sm.get_video_file_duration());
/// // get MP4 file context
/// println!("{:?}", sm.find_video_context());
/// ```
pub struct SmMotion {
    mmap: Mmap,
    /// Index where starts a video
    pub video_index: Option<usize>,
}

impl SmMotion {
    ///  First things first send here a file ref
    pub fn with(file: &File) -> Option<SmMotion> {
        Some(SmMotion {
            video_index: None,
            // Don't place entire file in memory, using memory efficient memory mapping
            mmap: match unsafe { Mmap::map(&file) } {
                Ok(m) => m,
                _ => return None,
            },
        })
    }

    /// Look for starting MP4 index in Samsung Motion Photo JPEG file
    pub fn find_video_index(&mut self) -> Result<Option<usize>, &'static str> {
        // This line is an indicator of ending JPEG file and starting MP4 file
        let indicator: Vec<u8> = vec![
            0x4D, 0x6F, 0x74, 0x69, 0x6F, 0x6E, 0x50, 0x68, 0x6F, 0x74, 0x6F, 0x5F, 0x44, 0x61,
            0x74, 0x61,
        ];

        // Using boyer moore for faster search of vec position in a file
        let bmb = BMByte::from(&indicator).unwrap();
        let bytes = Bytes::new(&self.mmap[..]);
        // Using the first entry because it is quite unique
        self.video_index = match bmb.find_first_in(bytes) {
            // Move index on the length of indicator
            Some(index) => Some(index + 16),
            None => None,
        };
        Ok(self.video_index)
    }

    /// Check if a photo has a Motion Photo feature
    pub fn has_video(&mut self) -> bool {
        match self.video_index {
            Some(_) => true,
            None => match self.find_video_index() {
                Ok(_) => self.has_video(),
                Err(_) => false,
            },
        }
    }

    /// Get video context from mp4parse.
    pub fn find_video_context(&mut self) -> Option<MediaContext> {
        match self.video_index {
            Some(index) => {
                let mut video_content = &self.mmap[index..];
                let mut context = mp4parse::MediaContext::new();
                let _ = mp4parse::read_mp4(&mut video_content, &mut context);
                Some(context)
            }
            None => match &self.find_video_index() {
                Ok(_) => self.find_video_context(),
                Err(_) => None,
            },
        }
    }

    /// Gen length of video file in photo in milliseconds
    pub fn get_video_file_duration(&mut self) -> Option<u64> {
        let context = self.find_video_context()?;
        if context.tracks.len() != 1 {
            return None;
        }
        match context.tracks[0].tkhd.as_ref() {
            Some(tkhd) => Some(tkhd.duration),
            None => None,
        }
    }

    // Save video file from image
    pub fn dump_video_file(&mut self, file: &mut File) -> Result<(), &str> {
        match self.video_index {
            Some(index) => {
                let video_content = &self.mmap[index..];
                match file.write_all(&video_content) {
                    Ok(()) => Ok(()),
                    Err(_) => Err("Can't write to file"),
                }
            }
            None => match self.find_video_index() {
                Ok(_) => self.dump_video_file(file),
                Err(e) => Err(e),
            },
        }
    }
}
