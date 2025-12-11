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
    fn iter(&self) -> Iter<'_, u8> {
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
/// let mut video_file = File::create("tests/tmp/video.mp4").unwrap();
/// // dump mp4 from jpeg
/// sm.dump_video_file(&mut video_file).unwrap();
/// // get video duration (no dump needed)
/// println!("{:?}", sm.get_video_file_duration());
/// // get MP4 file context
/// println!("{:?}", sm.find_video_context());
/// // You can also save index and use it afterwards
/// let mut sm_cached = SmMotion::with_precalculated(&photo_file, 3366251).unwrap();
/// println!("{:?}", sm_cached.get_video_file_duration());
/// ```
pub struct SmMotion {
    mmap: Mmap,
    /// Index where starts a video
    pub video_index: Option<usize>,
}

impl SmMotion {
    ///  First things first send here a file ref
    pub fn with(file: &File) -> Option<SmMotion> {
        let mut motion = SmMotion {
            video_index: None,
            // Don't place entire file in memory, using memory efficient memory mapping
            mmap: match unsafe { Mmap::map(&file) } {
                Ok(m) => m,
                _ => return None,
            },
        };

        let _ = motion.find_video_index();

        Some(motion)
    }

    /// Initialize SmMotion with a known video index.
    /// It's handful when you are caching the results of searching.
    pub fn with_precalculated(file: &File, index: usize) -> Option<SmMotion> {
        Some(SmMotion {
            video_index: Some(index),
            // Don't place entire file in memory, using memory efficient memory mapping
            mmap: match unsafe { Mmap::map(&file) } {
                Ok(m) => m,
                _ => return None,
            },
        })
    }

    /// Look for starting MP4 index in Samsung Motion Photo JPEG (or HEIC/HEIF) file
    pub fn find_video_index(&mut self) -> Result<Option<usize>, &'static str> {
        // This line is an indicator of ending JPEG (or HEIC/HEIF) file and starting MP4 file
        let indicator: Vec<u8> = vec![
            0x4D, 0x6F, 0x74, 0x69, 0x6F, 0x6E, 0x50, 0x68, 0x6F, 0x74, 0x6F, 0x5F, 0x44, 0x61,
            0x74, 0x61,
        ];

        // Using boyer moore for faster search of vec position in a file
        let bmb = BMByte::from(&indicator).unwrap();
        let bytes = Bytes::new(&self.mmap[..]);
        // Using the first entry because it is quite unique
        let base_index = match bmb.find_first_in(bytes) {
            // Move index on the length of indicator (right after the marker)
            Some(index) => Some(index + 16),
            None => None,
        };

        // On newer Samsung devices (e.g., SG22) there may be extra bytes after the marker
        // before the actual MP4 begins. The MP4 typically starts with an 'ftyp' box,
        // which appears 4 bytes after the start of the file/box (after the 32-bit size).
        // To make detection robust, scan forward from the marker for 'ftyp' and, if found,
        // shift the starting index back by 4 bytes to the true MP4 start.
        if let Some(start_after_marker) = base_index {
            // Limit the scan window to avoid scanning the entire image. 64 KiB should be plenty.
            let scan_start = start_after_marker;
            let scan_end = (scan_start + 65536).min(self.mmap.len());
            let scan_slice = &self.mmap[scan_start..scan_end];

            // Search for 'ftyp' inside the scan window using the same Boyer-Moore engine
            let ftyp: Vec<u8> = b"ftyp".to_vec();
            let bmb_ftyp = BMByte::from(&ftyp).unwrap();
            let bytes_ftyp = Bytes::new(scan_slice);
            let adjusted = match bmb_ftyp.find_first_in(bytes_ftyp) {
                Some(rel_pos) => {
                    // Ensure we don't underflow when backing up 4 bytes for the size field
                    if rel_pos >= 4 {
                        Some(scan_start + rel_pos - 4)
                    } else {
                        Some(scan_start)
                    }
                }
                None => None,
            };
            // If we didn't find 'ftyp' right after the marker (new HEIC layout),
            // try to locate the MP4 'ftyp' elsewhere in the file. Prefer the last
            // occurrence before the marker to avoid the HEIC's own 'ftyp' at the start.
            if let Some(idx) = adjusted {
                self.video_index = Some(idx);
            } else {
                // Define a small helper to test major brand after 'ftyp'
                let majors: [&[u8]; 5] = [b"isom", b"mp42", b"mp41", b"iso4", b"avc1"];
                let mut search_pos = 0usize;
                let mmap = &self.mmap;
                let mut chosen: Option<usize> = None;
                while let Some(pos) = mmap[search_pos..].windows(4).position(|w| w == b"ftyp") {
                    let abs_pos = search_pos + pos;
                    // Check we have enough bytes to read major brand
                    if abs_pos + 8 < mmap.len() {
                        let major = &mmap[abs_pos + 4..abs_pos + 8];
                        // Filter out HEIC/HEIF brands and keep MP4-like ones
                        let is_mp4_brand = majors.iter().any(|m| *m == major);
                        if is_mp4_brand {
                            // Only consider positions before the SEF footer marker to avoid false positives
                            if abs_pos < start_after_marker {
                                chosen = Some(abs_pos);
                            }
                        }
                    }
                    // Advance search position
                    search_pos = abs_pos + 4;
                    if search_pos >= mmap.len() { break; }
                }
                if let Some(ftyp_pos) = chosen {
                    // Back up 4 bytes for size field if possible
                    let start = if ftyp_pos >= 4 { ftyp_pos - 4 } else { ftyp_pos };
                    self.video_index = Some(start);
                } else {
                    // Fallback: keep original behavior (start right after marker)
                    self.video_index = Some(start_after_marker);
                }
            }
        } else {
            self.video_index = None;
        }
        Ok(self.video_index)
    }

    /// Check if a photo has a Motion Photo feature
    pub fn has_video(&self) -> bool {
        match self.video_index {
            Some(_) => true,
            None => false,
        }
    }

    /// Get video context from mp4parse.
    pub fn find_video_context(&self) -> Option<MediaContext> {
        match self.video_index {
            Some(index) => {
                let mut video_content = &self.mmap[index..];
                let mut context = mp4parse::MediaContext::new();
                let _ = mp4parse::read_mp4(&mut video_content, &mut context);
                Some(context)
            }
            None => None,
        }
    }

    /// Gen length of video file in photo in milliseconds
    pub fn get_video_file_duration(&self) -> Option<u64> {
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
    pub fn dump_video_file(&self, file: &mut File) -> Result<(), &str> {
        match self.video_index {
            Some(index) => {
                let video_content = &self.mmap[index..];
                match file.write_all(&video_content) {
                    Ok(()) => Ok(()),
                    Err(_) => Err("Can't write to file"),
                }
            }
            None => Err("Video not found"),
        }
    }
}
