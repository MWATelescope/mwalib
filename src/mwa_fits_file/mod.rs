// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Encapsulating filename and FitsFile

// This struct encapsulates the fitsio::FitsFile struct and adds back
// the 'filename' property that was removed in v0.21.0 of the fitsio crate
//
use fitsio::FitsFile;
use std::path::PathBuf;

#[cfg(test)]
mod test;

pub struct MWAFitsFile {
    /// FitsFile struct
    pub fits_file: FitsFile,

    /// Filename (filename, including
    /// the full or relative path to the file)
    pub filename: PathBuf,
}

impl MWAFitsFile {
    /// Creates an encapsulating struct to hold a FitsFile object and
    /// it's filename. Filename was removed from fitsio in 0.21, so
    /// this is a way to retain that property since we use it in error
    /// messages and debug.
    ///
    /// # Arguments
    ///
    /// * `filename` - filename of FITS file as a path or string.    
    ///
    /// * `fits_file` - an already created fitsio::FitsFile struct
    ///
    /// # Returns
    ///
    /// * A populated MWAFits object
    ///    
    pub fn new(filename: PathBuf, fits_file: FitsFile) -> Self {
        MWAFitsFile {
            fits_file,
            filename,
        }
    }
}
