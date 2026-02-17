// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for beam metadata
use crate::{metafits_context::MetafitsContext, MWAVersion};

#[test]
fn test_populate_calibration_fits() {
    let filename = String::from("test_files/metafits_cal_sol/1111842752_metafits.fits");

    let mc = MetafitsContext::new(&filename, Some(MWAVersion::CorrLegacy));
    assert!(mc.is_ok());

    let context = mc.unwrap();

    assert_eq!(context.obs_id, 1_111_842_752);
    assert_eq!(context.best_cal_fit_id, Some(1720774022));
    assert_eq!(context.best_cal_obs_id, Some(1111842752));
    assert_eq!(context.best_cal_code_ver, Some(String::from("0.17.22")));
    assert_eq!(
        context.best_cal_fit_timestamp,
        Some(String::from("2024-07-12T08:47:02.308203+00:00"))
    );
    assert_eq!(context.best_cal_creator, Some(String::from("calvin")));
    assert_eq!(context.best_cal_fit_iters, Some(3));
    assert_eq!(context.best_cal_fit_iter_limit, Some(20));

    assert!(context.calibration_fits.is_some());
    let fits = context.calibration_fits.unwrap();
    assert_eq!(fits.len(), 256);
}
