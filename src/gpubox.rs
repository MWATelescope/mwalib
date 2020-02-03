use std::collections::BTreeMap;
use std::fmt::Debug;
use std::string::ToString;

use fitsio::{hdu::FitsHdu, FitsFile};
use regex::Regex;

use crate::*;

#[derive(Debug)]
pub struct ObsTimes {
    pub start_millisec: u64,
    pub end_millisec: u64,
}

lazy_static! {
    static ref RE_BATCH: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2})_(?P<batch>\d{2}).fits").unwrap();
    static ref RE_OLD_FORMAT: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2}).fits").unwrap();
    static ref RE_BAND: Regex = Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2})").unwrap();
}

/// Group input gpubox files into a vector of vectors containing their
/// batches. A "gpubox batch" refers to the number XX in a gpubox filename
/// (e.g. 1065880128_20131015134930_gpubox01_XX.fits). Fail if the number of
/// files in each batch is not equal.
///
/// Some older files might have a "batchless" format
/// (e.g. 1065880128_20131015134930_gpubox01.fits); in this case, this function
/// will just return one sub-vector for one batch.
///
/// ```
/// use mwalib::gpubox::determine_gpubox_batches;
///
/// let files = vec![
///     "/home/chj/1065880128_20131015134929_gpubox02_00.fits",
///     "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
///     "/home/chj/1065880128_20131015134929_gpubox20_01.fits",
///     "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
///     "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
///     "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
/// ];
/// let result = determine_gpubox_batches(&files);
/// assert!(result.is_ok());
/// let result = result.unwrap();
/// // Three batches (02 is the biggest number).
/// assert_eq!(result.len(), 3);
/// assert_eq!(
///     result,
///     vec![
///         vec![
///             "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
///             "/home/chj/1065880128_20131015134929_gpubox02_00.fits"
///         ],
///         vec![
///             "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
///             "/home/chj/1065880128_20131015134929_gpubox20_01.fits"
///         ],
///         vec![
///             "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
///             "/home/chj/1065880128_20131015134930_gpubox15_02.fits"
///         ],
///     ]
/// );
/// ```
pub fn determine_gpubox_batches<T: AsRef<str> + ToString + Debug>(
    gpubox_filenames: &[T],
) -> Result<Vec<Vec<String>>, ErrorKind> {
    let mut old_format: Option<bool> = None;
    let mut output = vec![vec![]];
    for g in gpubox_filenames {
        match RE_BATCH.captures(g.as_ref()) {
            Some(caps) => {
                if old_format == None {
                    old_format = Some(false);
                }
                // Check if we've already matched any files as being the old
                // format. If so, then we've got a mix, and we should exit
                // early.
                else if old_format == Some(true) {
                    return Err(ErrorKind::Custom(format!(
                        "There are a mixture of gpubox filename types in {:?}",
                        gpubox_filenames
                    )));
                }

                let batch: usize = caps["batch"].parse()?;
                // Enlarge the output vector if we need to.
                while output.len() < batch + 1 {
                    output.push(vec![]);
                }
                output[batch].push(g.to_string());
            }
            // Try to match the old format.
            None => match RE_OLD_FORMAT.captures(g.as_ref()) {
                Some(_) => {
                    if old_format == None {
                        old_format = Some(true);
                    } else if old_format == Some(false) {
                        return Err(ErrorKind::Custom(format!(
                            "There are a mixture of gpubox filename types in {:?}",
                            gpubox_filenames
                        )));
                    }

                    output[0].push(g.to_string());
                }
                None => {
                    return Err(ErrorKind::Custom(format!(
                        "Could not identify the gpubox filename structure for {:?}",
                        g
                    )))
                }
            },
        }
    }

    // Check that an equal number of files are in each batch.
    if !output.iter().all(|x| x.len() == output[0].len()) {
        return Err(ErrorKind::Custom(format!(
            "There are an uneven number of files in the gpubox batches.\n{}",
            output
                .iter()
                .enumerate()
                .map(|(i, x)| format!("Batch {}: {}", i, x.len()))
                .collect::<Vec<String>>()
                .join(", ")
        )));
    }

    // Ensure the output is properly sorted - each batch is sorted by
    // coarse-band channel.
    for v in &mut output {
        v.sort_unstable_by(|a, b| {
            let a2 = &RE_BAND.captures(a).unwrap()["band"].parse::<u32>().unwrap();
            let b2 = &RE_BAND.captures(b).unwrap()["band"].parse::<u32>().unwrap();
            a2.cmp(b2)
        });
    }

    Ok(output)
}

/// Given a FITS file pointer and HDU, determine the time in units of
/// milliseconds.
pub fn determine_hdu_time(gpubox_fptr: &mut FitsFile, hdu: FitsHdu) -> Result<u64, ErrorKind> {
    let start_time: i64 = hdu.read_key(gpubox_fptr, "TIME")?;
    let start_millitime: i64 = hdu.read_key(gpubox_fptr, "MILLITIM")?;
    Ok((start_time * 1000 + start_millitime) as u64)
}

/// Iterate over each HDU of the given gpubox file, tracking which UNIX times
/// are associated with which HDU numbers.
pub fn map_unix_times_to_hdus(
    gpubox_fptr: &mut FitsFile,
) -> Result<BTreeMap<u64, usize>, ErrorKind> {
    let mut map = BTreeMap::new();
    let last_hdu_index = gpubox_fptr.iter().count();
    // Ignore the first HDU in all gpubox files; it contains only a little
    // metadata.
    for hdu_index in 1..last_hdu_index {
        let hdu = gpubox_fptr.hdu(hdu_index)?;
        let time = determine_hdu_time(gpubox_fptr, hdu)?;
        map.insert(time, hdu_index);
    }

    Ok(map)
}

/// Determine the proper start and end times of an observation. In this context,
/// "proper" refers to a time that is common to all gpubox files. Because gpubox
/// files may not all start and end at the same time, anything "dangling" is
/// trimmed. e.g.
///
/// time:     0123456789abcdef
/// gpubox01: ################
/// gpubox02:  ###############
/// gpubox03: ################
/// gpubox04:   ##############
/// gpubox05: ###############
/// gpubox06: ################
///
/// In this example, we start collecting data from time=2, and end at time=e,
/// because these are the first and last places that all gpubox files have
/// data. All dangling data is ignored.
///
/// See tests of this function or `obs_context.rs` for examples of constructing
/// the input to this function.
pub fn determine_obs_times(
    gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
) -> Result<ObsTimes, ErrorKind> {
    // Find the maximum number of gpubox files, and assume that this is the
    // total number of input gpubox files.
    let size = match gpubox_time_map.iter().map(|(_, submap)| submap.len()).max() {
        Some(m) => m,
        None => {
            return Err(ErrorKind::Custom(
                "determine_obs_times: Input BTreeMap was empty".to_string(),
            ))
        }
    };

    // Filter the first elements that don't satisfy `submap.len() == size`. The
    // first and last of the submaps that satisfy this condition are the proper
    // start and end of the observation.

    // Is there a way to iterate only once?
    let proper_start_millisec = match gpubox_time_map
        .iter()
        .find(|(_, submap)| submap.len() == size)
        .map(|(time, _)| *time)
    {
        Some(s) => s,
        None => {
            return Err(ErrorKind::Custom(
                "determine_obs_times: proper_start_millisec was not set".to_string(),
            ))
        }
    };
    let proper_end_millisec = match gpubox_time_map
        .iter()
        .filter(|(_, submap)| submap.len() == size)
        .last()
        .map(|(time, _)| *time)
    {
        Some(s) => s,
        None => {
            return Err(ErrorKind::Custom(
                "determine_obs_times: proper_end_millisec was not set".to_string(),
            ))
        }
    };

    Ok(ObsTimes {
        start_millisec: proper_start_millisec,
        end_millisec: proper_end_millisec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fitsio::images::{ImageDescription, ImageType};
    use std::env;
    use std::fs::remove_file;
    use std::time::SystemTime;

    #[test]
    fn determine_gpubox_batches_proper_format() {
        let files = vec![
            "1065880128_20131015134930_gpubox20_01.fits",
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result,
            vec![
                vec!["1065880128_20131015134930_gpubox01_00.fits"],
                vec!["1065880128_20131015134930_gpubox20_01.fits"],
                vec!["1065880128_20131015134930_gpubox15_02.fits"],
            ]
        );
    }

    #[test]
    fn determine_gpubox_batches_proper_format2() {
        let files = vec![
            "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            "/home/gs/1065880128_20131015134930_gpubox20_01.fits",
            "/var/cache/1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result,
            vec![
                vec!["/home/chj/1065880128_20131015134930_gpubox01_00.fits"],
                vec!["/home/gs/1065880128_20131015134930_gpubox20_01.fits"],
                vec!["/var/cache/1065880128_20131015134930_gpubox15_02.fits"],
            ]
        );
    }

    #[test]
    fn determine_gpubox_batches_proper_format3() {
        let files = vec![
            "/home/chj/1065880128_20131015134930_gpubox02_00.fits",
            "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            "/home/chj/1065880128_20131015134930_gpubox20_01.fits",
            "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
            "/home/chj/1065880128_20131015134930_gpubox14_02.fits",
            "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result,
            vec![
                vec![
                    "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
                    "/home/chj/1065880128_20131015134930_gpubox02_00.fits"
                ],
                vec![
                    "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
                    "/home/chj/1065880128_20131015134930_gpubox20_01.fits"
                ],
                vec![
                    "/home/chj/1065880128_20131015134930_gpubox14_02.fits",
                    "/home/chj/1065880128_20131015134930_gpubox15_02.fits"
                ],
            ]
        );
    }

    #[test]
    fn determine_gpubox_batches_proper_format4() {
        let files = vec![
            "/home/chj/1065880128_20131015134929_gpubox02_00.fits",
            "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            "/home/chj/1065880128_20131015134929_gpubox20_01.fits",
            "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
            "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
            "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result,
            vec![
                vec![
                    "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
                    "/home/chj/1065880128_20131015134929_gpubox02_00.fits"
                ],
                vec![
                    "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
                    "/home/chj/1065880128_20131015134929_gpubox20_01.fits"
                ],
                vec![
                    "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
                    "/home/chj/1065880128_20131015134930_gpubox15_02.fits"
                ],
            ]
        );
    }

    #[test]
    fn determine_gpubox_batches_invalid_filename() {
        let files = vec!["1065880128_20131015134930_gpubox0100.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batches_invalid_filename2() {
        let files = vec!["1065880128x_20131015134930_gpubox01_00.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batches_invalid_filename3() {
        let files = vec!["1065880128_920131015134930_gpubox01_00.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batches_invalid_count() {
        // There are no gpubox files for batch "01".
        let files = vec![
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batches_invalid_count2() {
        // There are not enough gpubox files for batch "02".
        let files = vec![
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox02_00.fits",
            "1065880128_20131015134930_gpubox01_01.fits",
            "1065880128_20131015134930_gpubox02_01.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batches_old_format() {
        let files = vec![
            "1065880128_20131015134930_gpubox01.fits",
            "1065880128_20131015134930_gpubox20.fits",
            "1065880128_20131015134930_gpubox15.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result,
            vec![vec![
                "1065880128_20131015134930_gpubox01.fits",
                "1065880128_20131015134930_gpubox15.fits",
                "1065880128_20131015134930_gpubox20.fits"
            ],]
        );
    }

    #[test]
    fn determine_gpubox_batches_mix() {
        let files = vec![
            "1065880128_20131015134930_gpubox01.fits",
            "1065880128_20131015134930_gpubox15_01.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    fn helper_make_fits_file(filename: String) -> (String, FitsFile, FitsHdu) {
        // FitsFile::create() expects the filename passed in to not
        // exist. Delete it if it exists.
        let mut file = env::temp_dir();
        file.push(filename);
        if file.exists() {
            remove_file(&file).unwrap();
        }
        let mut fptr = FitsFile::create(&file)
            .open()
            .expect("Couldn't open tempfile");
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        (file.to_str().unwrap().to_string(), fptr, hdu)
    }

    #[test]
    fn determine_hdu_time_test1() {
        // Create a (temporary) FITS file with some keys to test our functions.
        let (file, mut fptr, hdu) =
            helper_make_fits_file("determine_hdu_time_test1.fits".to_string());

        // Write the TIME and MILLITIM keys. Key types must be i64 to get any
        // sort of sanity.
        hdu.write_key(&mut fptr, "TIME", 1_434_494_061)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(&mut fptr, "MILLITIM", 0)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(&mut fptr, hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_434_494_061_000);

        remove_file(&file).expect("Couldn't remove file");
    }

    #[test]
    fn determine_hdu_time_test2() {
        let (file, mut fptr, hdu) =
            helper_make_fits_file("determine_hdu_time_test2.fits".to_string());

        hdu.write_key(&mut fptr, "TIME", 1_381_844_923)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(&mut fptr, "MILLITIM", 500)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(&mut fptr, hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_381_844_923_500);

        remove_file(&file).unwrap();
    }

    #[test]
    fn determine_hdu_time_test3() {
        // Use the current UNIX time.
        let current = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Err(e) => panic!("Something is wrong with time on your system: {}", e),
            Ok(n) => n.as_secs(),
        };

        let (file, mut fptr, hdu) =
            helper_make_fits_file("determine_hdu_time_test3.fits".to_string());

        hdu.write_key(&mut fptr, "TIME", current)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(&mut fptr, "MILLITIM", 500)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(&mut fptr, hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), current * 1000 + 500);

        remove_file(&file).unwrap();
    }

    #[test]
    fn map_unix_times_to_hdus_test() {
        let (file, mut fptr, _) =
            helper_make_fits_file("map_unix_times_to_hdus_test.fits".to_string());

        let times: Vec<(u64, u64)> =
            vec![(1_381_844_923, 500), (1_381_844_924, 0), (1_381_844_950, 0)];
        let mut expected = BTreeMap::new();
        let image_description = ImageDescription {
            data_type: ImageType::Float,
            dimensions: &[100, 100],
        };
        for (i, (time, millitime)) in times.iter().enumerate() {
            let hdu = fptr
                .create_image("EXTNAME".to_string(), &image_description)
                .expect("Couldn't create image");
            hdu.write_key(&mut fptr, "TIME", *time)
                .expect("Couldn't write key 'TIME'");
            hdu.write_key(&mut fptr, "MILLITIM", *millitime)
                .expect("Couldn't write key 'MILLITIM'");

            expected.insert(time * 1000 + millitime, i + 1);
        }

        let result = map_unix_times_to_hdus(&mut fptr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);

        remove_file(&file).unwrap();
    }

    #[test]
    fn determine_obs_times_test() {
        // Create two files, with mostly overlapping times, but also a little
        // dangling at the start and end.
        let common_times: Vec<u64> = vec![
            1_381_844_923_500,
            1_381_844_924_000,
            1_381_844_924_500,
            1_381_844_925_000,
            1_381_844_925_500,
        ];

        let mut input = BTreeMap::new();
        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(0, (0, 1));
        input.insert(1_381_844_923_000, new_time_tree);

        for (i, time) in common_times.iter().enumerate() {
            let mut new_time_tree = BTreeMap::new();
            // gpubox 0.
            new_time_tree.insert(0, (0, i + 2));
            // gpubox 1.
            new_time_tree.insert(1, (0, i + 1));
            input.insert(*time, new_time_tree);
        }

        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(1, (0, common_times.len() + 1));
        input.insert(1_381_844_926_000, new_time_tree);

        let expected_start = common_times.first().unwrap();
        let expected_end = common_times.last().unwrap();

        let result = determine_obs_times(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(&result.start_millisec, expected_start);
        assert_eq!(&result.end_millisec, expected_end);
    }
}
