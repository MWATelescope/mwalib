use fitsio::hdu::HduInfo;
use fitsio::FitsFile;

use crate::*;

/// `mwalib` buffer. Used to transport data out of gpubox files.
///
/// The name is not following the rust convention of camel case, to make it look
/// more like a C library.
#[allow(non_camel_case_types)]
pub struct mwalibBuffer {
    // Track the UNIX time (in milliseconds) that will be read next.
    pub current_time_milli: u64,

    // Keep gpubox file FITS file pointers open. Structured:
    // `gpubox_fptrs[batch][fptr]`.
    pub gpubox_fptrs: Vec<Vec<FitsFile>>,

    // These variables are provided for convenience, so a caller knows how to
    // index the data buffer.
    pub num_data_scans: usize,
    pub num_gpubox_files: usize,
    pub gpubox_hdu_size: usize,
}

fn get_hdu_image_size(fptr: &mut FitsFile) -> Result<usize, ErrorKind> {
    match fptr.hdu(1)?.info {
        HduInfo::ImageInfo { shape, .. } => Ok(shape.iter().product()),
        _ => Err(ErrorKind::Custom(
            "mwalibBuffer::read: HDU 2 of the first gpubox_fptr was not an image".to_string(),
        )),
    }
}

impl mwalibBuffer {
    /// From an obs. context, create a buffer. The buffer is primed to read
    /// gpubox files from the first appropriate HDU of the first batch. The
    /// buffer is designed to always be used alongside the obs. context, so no
    /// redundant information is needed.
    // TODO: Write tests
    pub fn new(o: &mwalibObsContext, num_scans: usize) -> Result<mwalibBuffer, ErrorKind> {
        // Keep track of the gpubox HDU size.
        let mut size = None;

        // Open all the gpubox files.
        let mut gpubox_fptrs = Vec::with_capacity(num_scans);
        let mut num_gpubox_files = 0;
        for (batch_num, batch) in o.gpubox_batches.iter().enumerate() {
            num_gpubox_files = batch.len();
            gpubox_fptrs.push(Vec::with_capacity(o.gpubox_batches.len()));
            for gpubox_file in batch {
                let mut fptr = FitsFile::open(&gpubox_file)
                    .with_context(|| format!("Failed to open {:?}", gpubox_file))?;

                // Determine the size of the gpubox HDU image. mwalib will panic
                // if this size is not consistent for all HDUs in all gpubox
                // files.
                let this_size = get_hdu_image_size(&mut fptr)?;
                if let Some(s) = size {
                    if this_size != s {
                        return Err(ErrorKind::Custom(
                            "mwalibBuffer::read: Error: HDU sizes in gpubox files are not equal"
                                .to_string(),
                        ));
                    }
                } else {
                    size = Some(this_size);
                }

                gpubox_fptrs[batch_num].push(fptr);
            }
        }

        if let Some(s) = size {
            Ok(mwalibBuffer {
                current_time_milli: o.start_time_milliseconds,
                gpubox_fptrs,
                num_data_scans: num_scans,
                num_gpubox_files,
                gpubox_hdu_size: s,
            })
        } else {
            Err(ErrorKind::Custom(
                "mwalibBuffer::read: Error: 'size' was not set. Were any files provided?".to_string(),
            ))
        }
    }

    /// The output `buffer` is structured: `buffer[scan][gpubox_index][data]`.
    pub fn read(&mut self, o: &mwalibObsContext) -> Result<Vec<Vec<Vec<f32>>>, ErrorKind> {
        // Is there enough data left to fit into the total number of scans? If
        // not, we need to resize `buffer`.
        let ct = self.current_time_milli as i64;
        let it = o.integration_time_milliseconds as i64;
        let et = o.end_time_milliseconds as i64;
        // The end time is inclusive; need to add the integration time to get
        // the last scan.
        let new_num_scans = ((et - ct + it) as f64 / it as f64) as i64;

        if new_num_scans < 0 {
            return Err(ErrorKind::Custom("mwalibBuffer::read: A negative number for `new_num_scans` was calculated; this should only happen if something has manually changed the timings.".to_string()));
        };

        self.num_data_scans = self.num_data_scans.min(new_num_scans as usize);
        // Completely reset the internal data buffer.
        let mut data = vec![vec![vec![]; self.num_gpubox_files]; self.num_data_scans];

        for scan in &mut data {
            for gpubox_index in 0..self.num_gpubox_files {
                let (batch_index, hdu_index) =
                    o.gpubox_time_map[&self.current_time_milli][&gpubox_index];
                let mut fptr = &mut self.gpubox_fptrs[batch_index][gpubox_index];
                let hdu = fptr.hdu(hdu_index)?;
                scan[gpubox_index] = hdu.read_image(&mut fptr)?;

                // We expect *all* HDUs to have the same number of floats. Error
                // if this is not true.
                if scan[gpubox_index].len() != self.gpubox_hdu_size {
                    return Err(ErrorKind::Custom(
                        "mwalibBuffer::read: Error: HDU sizes in gpubox files are not equal"
                            .to_string(),
                    ));
                }
            }
            self.current_time_milli += o.integration_time_milliseconds;
        }

        Ok(data)
    }
}
