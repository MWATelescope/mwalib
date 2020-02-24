// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Functions for organising and checking the consistency of gpubox files.
 */

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
    pub duration_milliseconds: u64,
}

lazy_static! {
    static ref RE_MWAX: Regex =
        Regex::new(r"\d{10}_\d{8}(.)?\d{6}_ch(?P<channel>\d{3})_(?P<batch>\d{3}).fits").unwrap();
    static ref RE_LEGACY_BATCH: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2})_(?P<batch>\d{2}).fits").unwrap();
    static ref RE_OLD_LEGACY_FORMAT: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2}).fits").unwrap();
    static ref RE_BAND: Regex = Regex::new(r"\d{10}_\d{14}_(ch|gpubox)(?P<band>\d+)").unwrap();
}

/// Group input gpubox files into a vector of vectors containing their
/// batches. A "gpubox batch" refers to the number XX in a gpubox filename
/// (e.g. `1065880128_20131015134930_gpubox01_XX.fits`). Fail if the number of
/// files in each batch is not equal.
///
/// Some older files might have a "batchless" format
/// (e.g. `1065880128_20131015134930_gpubox01.fits`); in this case, this
/// function will just return one sub-vector for one batch.
///
/// ```
/// use mwalib::{*, gpubox::determine_gpubox_batches};
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
/// let (result, corr_format) = result.unwrap();
/// // Three batches (02 is the biggest number).
/// assert_eq!(result.len(), 3);
/// assert_eq!(corr_format, CorrelatorVersion::Legacy);
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
) -> Result<(Vec<Vec<String>>, CorrelatorVersion), ErrorKind> {
    if gpubox_filenames.is_empty() {
        return Err(ErrorKind::Custom(
            "determine_gpubox_batches: gpubox / mwax fits files missing".to_string(),
        ));
    }

    let mut format = None;
    let mut output = vec![vec![]];

    for g in gpubox_filenames {
        match RE_MWAX.captures(g.as_ref()) {
            Some(caps) => {
                // Check if we've already matched any files as being the old
                // format. If so, then we've got a mix, and we should exit
                // early.
                match format {
                    None => format = Some(CorrelatorVersion::V2),
                    Some(CorrelatorVersion::V2) => (),
                    _ => {
                        return Err(ErrorKind::Custom(format!(
                            "There are a mixture of gpubox filename types in {:?}",
                            gpubox_filenames
                        )))
                    }
                }

                let batch: usize = caps["batch"].parse()?;
                // Enlarge the output vector if we need to.
                while output.len() < batch + 1 {
                    output.push(vec![]);
                }
                output[batch].push(g.to_string());
            }

            // Try to match the legacy format.
            None => match RE_LEGACY_BATCH.captures(g.as_ref()) {
                Some(caps) => {
                    match format {
                        None => format = Some(CorrelatorVersion::Legacy),
                        Some(CorrelatorVersion::Legacy) => (),
                        _ => {
                            return Err(ErrorKind::Custom(format!(
                                "There are a mixture of gpubox filename types in {:?}",
                                gpubox_filenames
                            )))
                        }
                    }

                    let batch: usize = caps["batch"].parse()?;
                    while output.len() < batch + 1 {
                        output.push(vec![]);
                    }
                    output[batch].push(g.to_string());
                }

                // Try to match the old legacy format.
                None => match RE_OLD_LEGACY_FORMAT.captures(g.as_ref()) {
                    Some(_) => {
                        match format {
                            None => format = Some(CorrelatorVersion::OldLegacy),
                            Some(CorrelatorVersion::OldLegacy) => (),
                            _ => {
                                return Err(ErrorKind::Custom(format!(
                                    "There are a mixture of gpubox filename types in {:?}",
                                    gpubox_filenames
                                )))
                            }
                        }

                        // There's only one batch.
                        output[0].push(g.to_string());
                    }
                    None => {
                        return Err(ErrorKind::Custom(format!(
                            "Could not identify the gpubox filename structure for {:?}",
                            g
                        )))
                    }
                },
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
            let a2 = &RE_BAND.captures(a).unwrap()["band"].parse::<u8>().unwrap();
            let b2 = &RE_BAND.captures(b).unwrap()["band"].parse::<u8>().unwrap();
            a2.cmp(b2)
        });
    }

    Ok((output, format.unwrap()))
}

/// Given a FITS file pointer and HDU, determine the time in units of
/// milliseconds.
pub fn determine_hdu_time(gpubox_fptr: &mut FitsFile, hdu: &FitsHdu) -> Result<u64, ErrorKind> {
    let start_time: i64 = hdu.read_key(gpubox_fptr, "TIME")?;
    let start_millitime: i64 = hdu.read_key(gpubox_fptr, "MILLITIM")?;
    Ok((start_time * 1000 + start_millitime) as u64)
}

/// Iterate over each HDU of the given gpubox file, tracking which UNIX times
/// are associated with which HDU numbers.
pub fn map_unix_times_to_hdus(
    gpubox_fptr: &mut FitsFile,
    correlator_format: &CorrelatorVersion,
) -> Result<BTreeMap<u64, usize>, ErrorKind> {
    let mut map = BTreeMap::new();
    let last_hdu_index = gpubox_fptr.iter().count();
    // The new correlator has a "weights" HDU in each alternating HDU. Skip
    // those.
    let step_size = if correlator_format == &CorrelatorVersion::V2 {
        2
    } else {
        1
    };
    // Ignore the first HDU in all gpubox files; it contains only a little
    // metadata.
    for hdu_index in (1..last_hdu_index).step_by(step_size) {
        let hdu = gpubox_fptr.hdu(hdu_index)?;
        let time = determine_hdu_time(gpubox_fptr, &hdu)?;
        map.insert(time, hdu_index);
    }

    Ok(map)
}

/// Determine the proper start and end times of an observation. In this context,
/// "proper" refers to a time that is common to all gpubox files. Because gpubox
/// files may not all start and end at the same time, anything "dangling" is
/// trimmed. e.g.
///
/// ```text
/// time:     0123456789abcdef
/// gpubox01: ################
/// gpubox02:  ###############
/// gpubox03: ################
/// gpubox04:   ##############
/// gpubox05: ###############
/// gpubox06: ################
/// ```
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
    let mut i = gpubox_time_map
        .iter()
        .filter(|(_, submap)| submap.len() == size);
    let proper_start_millisec = match i.next().map(|(time, _)| *time) {
        Some(s) => s,
        None => {
            return Err(ErrorKind::Custom(
                "determine_obs_times: proper_start_millisec was not set".to_string(),
            ))
        }
    };
    let proper_end_millisec = match i.last().map(|(time, _)| *time) {
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
        duration_milliseconds: (proper_end_millisec - proper_start_millisec) + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fitsio::images::{ImageDescription, ImageType};
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
        let (result, corr_format) = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
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
        let (result, corr_format) = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
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
        let (result, corr_format) = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
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
        let (result, corr_format) = result.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
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
        let (result, corr_format) = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(corr_format, CorrelatorVersion::OldLegacy);
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
    fn determine_gpubox_batches_new_format() {
        let files = vec![
            "1065880128_20131015134930_ch001_000.fits",
            "1065880128_20131015134930_ch002_000.fits",
            "1065880128_20131015135030_ch001_001.fits",
            "1065880128_20131015135030_ch002_001.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (result, corr_format) = result.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(corr_format, CorrelatorVersion::V2);
        assert_eq!(
            result,
            vec![
                vec![
                    "1065880128_20131015134930_ch001_000.fits",
                    "1065880128_20131015134930_ch002_000.fits",
                ],
                vec![
                    "1065880128_20131015135030_ch001_001.fits",
                    "1065880128_20131015135030_ch002_001.fits",
                ]
            ],
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

    #[test]
    fn determine_hdu_time_test1() {
        // Create a (temporary) FITS file with some keys to test our functions.
        // FitsFile::create() expects the filename passed in to not
        // exist. Delete it if it exists.
        let filename = "determine_hdu_time_test1.fits";

        if std::path::Path::new(filename).exists() {
            remove_file(filename).unwrap();
        }
        let mut fptr = FitsFile::create(filename)
            .open()
            .expect("Couldn't open tempfile");
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // Write the TIME and MILLITIM keys. Key types must be i64 to get any
        // sort of sanity.
        hdu.write_key(&mut fptr, "TIME", 1_434_494_061)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(&mut fptr, "MILLITIM", 0)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(&mut fptr, &hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_434_494_061_000);

        remove_file(&filename).expect("Couldn't remove file");
    }

    #[test]
    fn determine_hdu_time_test2() {
        // Create a (temporary) FITS file with some keys to test our functions.
        // FitsFile::create() expects the filename passed in to not
        // exist. Delete it if it exists.
        let filename = "determine_hdu_time_test2.fits";

        if std::path::Path::new(filename).exists() {
            remove_file(filename).unwrap();
        }
        let mut fptr = FitsFile::create(filename)
            .open()
            .expect("Couldn't open tempfile");
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        hdu.write_key(&mut fptr, "TIME", 1_381_844_923)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(&mut fptr, "MILLITIM", 500)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(&mut fptr, &hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_381_844_923_500);

        remove_file(&filename).unwrap();
    }

    #[test]
    fn determine_hdu_time_test3() {
        // Use the current UNIX time.
        let current = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Err(e) => panic!("Something is wrong with time on your system: {}", e),
            Ok(n) => n.as_secs(),
        };

        // Create a (temporary) FITS file with some keys to test our functions.
        // FitsFile::create() expects the filename passed in to not
        // exist. Delete it if it exists.
        let filename = "determine_hdu_time_test3.fits";

        if std::path::Path::new(filename).exists() {
            remove_file(filename).unwrap();
        }
        let mut fptr = FitsFile::create(filename)
            .open()
            .expect("Couldn't open tempfile");
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        hdu.write_key(&mut fptr, "TIME", current)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(&mut fptr, "MILLITIM", 500)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(&mut fptr, &hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), current * 1000 + 500);

        remove_file(&filename).unwrap();
    }

    #[test]
    fn map_unix_times_to_hdus_test() {
        // Create a (temporary) FITS file with some keys to test our functions.
        // FitsFile::create() expects the filename passed in to not
        // exist. Delete it if it exists.
        let filename = "map_unix_times_to_hdus_test.fits";

        if std::path::Path::new(filename).exists() {
            remove_file(filename).unwrap();
        }
        let mut fptr = FitsFile::create(filename)
            .open()
            .expect("Couldn't open tempfile");

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

        let result = map_unix_times_to_hdus(&mut fptr, &CorrelatorVersion::Legacy);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);

        remove_file(&filename).unwrap();
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
