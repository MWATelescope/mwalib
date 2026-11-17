// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Conversion of tile heights above the WGS84 ellipsoid (`HeightCode::EllipsoidHeight`) to the convention of the other
//! tiles (`HeightCode::ArrayCentreOffset`): the local "up" above the array centre's tangent plane, plus
//! `MWALIB_MWA_HEIGHT_CODE_ARRAY_CENTRE_OFFSET_METRES`.
//!
//! The MWA database stores the long baseline tiles' heights as their raw surveyed heights above the ellipsoid. These
//! differ from "up" by the Earth's curvature, about (east² + north²) / 2R: 0.08 m at 1 km and about 0.97 m for LBD7 at
//! 3.5 km. See the MWA wiki page "MWA Tile coordinate errors". The method is the same as Marlu's
//! `ENH::from_ellipsoidal_height` (Marlu PR 51).

use crate::{
    MWALIB_MWA_HEIGHT_CODE_ARRAY_CENTRE_OFFSET_METRES, MWALIB_MWA_LATITUDE_RADIANS,
    MWALIB_MWA_LONGITUDE_RADIANS,
};

/// WGS84 semi-major axis (metres)
const WGS84_SEMI_MAJOR_AXIS_M: f64 = 6_378_137.0;

/// WGS84 flattening
const WGS84_FLATTENING: f64 = 1.0 / 298.257_223_563;

/// Number of corrections of "up". The ellipsoidal height changes by about 1 m per metre of "up", so each correction
/// gains several orders of magnitude; after 4 the error is far below a micrometre.
const UP_ITERATIONS: usize = 4;

/// Number of iterations of the latitude when converting a geocentric position to a height above the ellipsoid.
/// For points near the surface this converges to below a micrometre in 3 or 4 iterations.
const LATITUDE_ITERATIONS: usize = 6;

/// Square of the WGS84 first eccentricity
fn wgs84_eccentricity_squared() -> f64 {
    WGS84_FLATTENING * (2.0 - WGS84_FLATTENING)
}

/// Convert a WGS84 geodetic position to geocentric (ECEF) coordinates.
///
/// # Arguments
///
/// * `latitude_rad` - Geodetic latitude (radians).
///
/// * `longitude_rad` - Longitude (radians).
///
/// * `height_m` - Height above the ellipsoid (metres).
///
/// # Returns
///
/// * The geocentric position [x, y, z] (metres).
///
fn geodetic_to_geocentric(latitude_rad: f64, longitude_rad: f64, height_m: f64) -> [f64; 3] {
    let e2 = wgs84_eccentricity_squared();
    let (sin_lat, cos_lat) = latitude_rad.sin_cos();
    let (sin_lon, cos_lon) = longitude_rad.sin_cos();
    let prime_vertical_radius = WGS84_SEMI_MAJOR_AXIS_M / (1.0 - e2 * sin_lat * sin_lat).sqrt();

    [
        (prime_vertical_radius + height_m) * cos_lat * cos_lon,
        (prime_vertical_radius + height_m) * cos_lat * sin_lon,
        (prime_vertical_radius * (1.0 - e2) + height_m) * sin_lat,
    ]
}

/// Get the height above the WGS84 ellipsoid of a geocentric (ECEF) position.
///
/// # Arguments
///
/// * `position` - The geocentric position [x, y, z] (metres).
///
/// # Returns
///
/// * The height above the ellipsoid (metres).
///
fn geocentric_to_ellipsoidal_height(position: [f64; 3]) -> f64 {
    let e2 = wgs84_eccentricity_squared();
    let [x, y, z] = position;
    let p = x.hypot(y);

    // Start from the latitude for a height of 0, then iterate
    let mut latitude_rad = z.atan2(p * (1.0 - e2));
    let mut height_m = 0.0;

    for _ in 0..LATITUDE_ITERATIONS {
        let sin_lat = latitude_rad.sin();
        let prime_vertical_radius = WGS84_SEMI_MAJOR_AXIS_M / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        height_m = p / latitude_rad.cos() - prime_vertical_radius;
        latitude_rad =
            z.atan2(p * (1.0 - e2 * prime_vertical_radius / (prime_vertical_radius + height_m)));
    }

    height_m
}

/// Convert a tile height above the WGS84 ellipsoid to the `ArrayCentreOffset` convention: the tile's local "up" above
/// the array centre's tangent plane (along the array centre's vertical), plus
/// `MWALIB_MWA_HEIGHT_CODE_ARRAY_CENTRE_OFFSET_METRES`.
///
/// # Arguments
///
/// * `east_m` - Tile position East of the array centre (metres).
///
/// * `north_m` - Tile position North of the array centre (metres).
///
/// * `ellipsoidal_height_m` - Tile height above the WGS84 ellipsoid (metres), as in the metafits.
///
/// # Returns
///
/// * The tile height in the `ArrayCentreOffset` convention (metres).
///
pub(crate) fn ellipsoidal_height_to_array_centre_offset_height(
    east_m: f64,
    north_m: f64,
    ellipsoidal_height_m: f64,
) -> f64 {
    let array_height_m = MWALIB_MWA_HEIGHT_CODE_ARRAY_CENTRE_OFFSET_METRES;
    let (sin_lat, cos_lat) = MWALIB_MWA_LATITUDE_RADIANS.sin_cos();
    let (sin_lon, cos_lon) = MWALIB_MWA_LONGITUDE_RADIANS.sin_cos();

    let centre = geodetic_to_geocentric(
        MWALIB_MWA_LATITUDE_RADIANS,
        MWALIB_MWA_LONGITUDE_RADIANS,
        array_height_m,
    );
    // Unit vectors of the array centre's local East, North and Up, in geocentric coordinates
    let east = [-sin_lon, cos_lon, 0.0];
    let north = [-sin_lat * cos_lon, -sin_lat * sin_lon, cos_lat];
    let up = [cos_lat * cos_lon, cos_lat * sin_lon, sin_lat];

    let position = |up_m: f64| -> [f64; 3] {
        std::array::from_fn(|i| centre[i] + east_m * east[i] + north_m * north[i] + up_m * up[i])
    };

    // Start with "up" as if the ellipsoid were flat, then correct it until the point's ellipsoidal height is right
    let mut up_m = ellipsoidal_height_m - array_height_m;
    for _ in 0..UP_ITERATIONS {
        up_m += ellipsoidal_height_m - geocentric_to_ellipsoidal_height(position(up_m));
    }

    up_m + array_height_m
}
