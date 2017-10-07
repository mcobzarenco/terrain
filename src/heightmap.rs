use std::f32::consts::{FRAC_1_PI, PI};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use image;
use nalgebra::{FloatPoint, Origin, Point2, Point3, Vector2};

use errors::{ChainErr, ErrorKind, Result};
use math::{CpuScalar, ScalarField3, ScalarField2};

pub struct Heightmap {
    radius: CpuScalar,
    height: Vec<CpuScalar>,
    x_max: usize,
    y_max: usize,
}

impl Heightmap {
    pub fn from_pds<P>(
        radius: CpuScalar,
        x_samples: usize,
        y_samples: usize,
        path: P,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Debug,
    {
        let file = try!(File::open(path).chain_err(
            || "Falied opening heightmap file.",
        ));
        let mut reader = BufReader::new(file);
        let num_samples = x_samples * y_samples;
        let mut height = Vec::with_capacity(num_samples);

        let mut min_height: CpuScalar = 0.0;
        let mut max_height: CpuScalar = 0.0;

        while height.len() < num_samples {
            let value = try!(reader.read_i16::<BigEndian>().chain_err(
                || "Heightmap creation failed! Could not read value from file.",
            )) as CpuScalar;
            min_height = min_height.min(value);
            max_height = max_height.max(value);
            height.push(value);
        }
        let remaining = try!(reader.read_to_end(&mut Vec::new()).chain_err(
            || "Heightmap creation failed! Could not read value from file.",
        ));
        if remaining > 0 {
            error!(
                "Found unexpected data in heightmap file; expected {} ({} x {}) values)",
                num_samples,
                x_samples,
                y_samples
            );
            Err(ErrorKind::UnexhaustedHeightmapFile.into())
        } else {
            info!(
                "Heightmap len: {} [{}, {}]",
                height.len(),
                min_height,
                max_height
            );

            Ok(Heightmap {
                height: height,
                radius: radius,
                x_max: x_samples - 1,
                y_max: y_samples - 1,
            })
        }
    }

    pub fn from_image<P>(radius: CpuScalar, path: P) -> Result<Self>
    where
        P: AsRef<Path> + Debug,
    {
        let image = try!(image::open(path.as_ref()).chain_err(|| {
            format!("Could not open heightmap image at {:?}", path)
        })).to_luma();

        let (x_samples, y_samples) = image.dimensions();
        let num_samples = (x_samples * y_samples) as usize;
        let mut height = vec![0.0; num_samples];
        let mut min_height: CpuScalar = 0.0;
        let mut max_height: CpuScalar = 0.0;

        let num_written = image
            .enumerate_pixels()
            .map(|(x, y, pixel)| {
                let value = pixel.data[0] as CpuScalar;
                min_height = min_height.min(value);
                max_height = max_height.max(value);
                height[(y * x_samples + x) as usize] = value;
            })
            .count();

        assert!(num_samples == num_written && num_samples == height.len());
        info!(
            "Heightmap len: {} [{}, {}]",
            height.len(),
            min_height,
            max_height
        );

        Ok(Heightmap {
            height: height,
            radius: radius,
            x_max: (x_samples - 1) as usize,
            y_max: (y_samples - 1) as usize,
        })
    }

    #[inline]
    fn discrete_height_at(&self, x: usize, y: usize) -> CpuScalar {
        self.height[y * (self.x_max + 1) + x]
    }
}

impl ScalarField2 for Heightmap {
    #[inline]
    fn value_at(&self, position: &Point2<CpuScalar>) -> CpuScalar {
        let (long, lat) = (position[0], position[1]);
        assert!(
            0.0 <= long && long <= 1.0 && 0.0 <= lat && lat <= 1.0,
            format!("{} {}", long, lat)
        );
        let x = self.x_max as CpuScalar * long.min(0.999).max(0.001);
        let y = self.y_max as CpuScalar * lat.min(0.999).max(0.001);

        // Integer grid coordinates as floats
        let x0 = (x - 0.5).floor().max(0.0);
        let x1 = (x + 0.5).floor().min(self.x_max as CpuScalar);
        let y0 = (y - 0.5).floor().max(0.0);
        let y1 = (y + 0.5).floor().min(self.y_max as CpuScalar);

        // Heights on the grid
        let h00 = self.discrete_height_at(x0 as usize, y0 as usize);
        let h01 = self.discrete_height_at(x0 as usize, y1 as usize);
        let h10 = self.discrete_height_at(x1 as usize, y0 as usize);
        let h11 = self.discrete_height_at(x1 as usize, y1 as usize);

        let hx0 = ((x1 - x) * h00 + (x - x0) * h10) / (x1 - x0);
        let hx1 = ((x1 - x) * h01 + (x - x0) * h11) / (x1 - x0);
        let hxy = ((y1 - y) * hx0 + (y - y0) * hx1) / (y1 - y0);

        // if hxy != 0.0 {
        //     println!("long: {} lat: {} -> xy: {} {} {} {} | h: {} {} {} {} | hxy: {} {} {}",
        //              long,
        //              lat,
        //              x0,
        //              x1,
        //              y0,
        //              y1,
        //              h00,
        //              h01,
        //              h10,
        //              h11,
        //              hx0,
        //              hx1,
        //              hxy);
        // }

        assert!(
            hxy.is_finite(),
            format!(
                "long: {} lat: {} -> xy: {} {} {} {} | h: {} {} {} {} | \
                         hxy: {} {} {}",
                long,
                lat,
                x0,
                x1,
                y0,
                y1,
                h00,
                h01,
                h10,
                h11,
                hx0,
                hx1,
                hxy
            )
        );
        hxy
    }
}

impl ScalarField3 for Heightmap {
    #[inline]
    fn value_at(&self, position: &Point3<CpuScalar>) -> CpuScalar {
        let r = position.distance(&Point3::origin()) + 1e-4;
        let long = (position[2].atan2(position[0]) + PI) * FRAC_1_PI * 0.5;
        let lat = (position[1] / r).acos() * FRAC_1_PI;

        let field_radius = self.radius +
            <Self as ScalarField2>::value_at(self, &(Point2::new(long, lat))) / 1000.0;

        r - field_radius
    }
}

// pub trait MapProjection {
//     fn project(&self, position: &Point3<CpuScalar>) -> Point2<CpuScalar>;
// }

// impl<Proj> ScalarField3 for Proj
//     where Proj: MapProjection + ScalarField2
// {
//     #[inline]
//     fn value_at(&self, position: &Point3<CpuScalar>) -> CpuScalar {
//         let projection = <Self as MapProjection>::project(self, position);
//         <Self as ScalarField2>::value_at(self, &projection)
//     }
// }

// pub struct CylindricalProjection {
//     radius: CpuScalar,
// }

// impl CylindricalProjection {
//     pub fn new(radius: CpuScalar) -> Self {
//         CylindricalProjection { radius: radius }
//     }
// }

// impl MapProjection for CylindricalProjection {
//     fn project(&self, position: &Point3<CpuScalar>) -> Point2<CpuScalar> {
//         let r = position.distance(&Point3::origin()) + 1e-4;
//         let long = (position[2].atan2(position[0]) + PI) * FRAC_1_PI * 0.5;
//         let lat = (position[1] / r).acos() * FRAC_1_PI;
//         Point2::new(long, lat)
//     }
// }
