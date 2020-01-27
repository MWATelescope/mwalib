use std::fmt::Debug;
use std::string::ToString;

// use fitsio::FitsFile;
use fitsio::{hdu::FitsHdu, FitsFile};
use regex::Regex;

use crate::fits_read::get_fits_key;
use crate::*;

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

/// Given file pointers to gpubox files arranged into batches, determine the
/// proper start and end times. In this context, "proper" refers to a time that
/// is common to all gpubox files. Because gpubox files may not all start and
/// end at the same time, anything "dangling" is trimmed. e.g.
///
/// time:     0123456789abcdef
/// gpubox01: ################
/// gpubox02:  ###############
/// gpubox03: ################
/// gpubox04:   ##############
/// gpubox05: ###############
/// gpubox06: ################
///
/// Here, we start collecting data from time=2, and end at time=e, because these
/// are the first and last places that all gpubox files have data. All dangling
/// data is ignored.
// TODO: Make this signature generic.
pub fn determine_obs_times(gpubox_fptrs: &mut Vec<Vec<FitsFile>>) -> Result<ObsTimes, ErrorKind> {
    // Assume that start times are much bigger than 0 (pretty safe).
    let mut proper_start_millisec = 0;
    if let Some(v) = gpubox_fptrs.first_mut() {
        for mut fptr in v.iter_mut() {
            let hdu = fptr.hdu(0)?;
            let mut start_time: i64 = hdu.read_key(&mut fptr, "TIME")?;
            let start_millitime: i64 = hdu.read_key(&mut fptr, "MILLITIM")?;
            start_time = start_time * 1000 + start_millitime;
            if start_time > proper_start_millisec {
                proper_start_millisec = start_time
            };
        }
    }

    let mut proper_end_millisec = 0;
    if let Some(v) = gpubox_fptrs.last_mut() {
        for mut fptr in v.iter_mut() {
            let hdu = fptr
                .iter()
                .last()
                .expect("determine_obs_times: Could not move to last HDU");
            let mut end_time: i64 = hdu.read_key(&mut fptr, "TIME")?;
            let end_millitime: i64 = hdu.read_key(&mut fptr, "MILLITIM")?;
            end_time = end_time * 1000 + end_millitime;
            if proper_end_millisec == 0 || end_time < proper_end_millisec {
                proper_end_millisec = end_time;
            };
        }
    }

    Ok(ObsTimes {
        start_millisec: proper_start_millisec as u64,
        end_millisec: proper_end_millisec as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_gpubox_batch_count_proper_format() {
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
    fn determine_gpubox_batch_count_proper_format2() {
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
    fn determine_gpubox_batch_count_proper_format3() {
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
    fn determine_gpubox_batch_count_proper_format4() {
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
    fn determine_gpubox_batch_count_invalid_filename() {
        let files = vec!["1065880128_20131015134930_gpubox0100.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batch_count_invalid_filename2() {
        let files = vec!["1065880128x_20131015134930_gpubox01_00.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batch_count_invalid_filename3() {
        let files = vec!["1065880128_920131015134930_gpubox01_00.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batch_count_invalid_count() {
        // There are no gpubox files for batch "01".
        let files = vec![
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn determine_gpubox_batch_count_invalid_count2() {
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
    fn determine_gpubox_batch_count_old_format() {
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
    fn determine_gpubox_batch_count_mix() {
        let files = vec![
            "1065880128_20131015134930_gpubox01.fits",
            "1065880128_20131015134930_gpubox15_01.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }
}
