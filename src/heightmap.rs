use std::cmp::min;
use std::f32::consts::{FRAC_1_PI, FRAC_2_PI, PI};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use nalgebra::Vector2;

use errors::{ChainErr, ErrorKind, Result};
use math::{CpuScalar, ScalarField, ScalarField2};

pub struct Heightmap {
    radius: CpuScalar,
    height: Vec<CpuScalar>,
    x_max: usize,
    y_max: usize,
}

impl Heightmap {
    pub fn from_pds<P>(radius: CpuScalar,
                       x_samples: usize,
                       y_samples: usize,
                       path: P)
                       -> Result<Self>
        where P: AsRef<Path> + Debug
    {
        let file = try!(File::open(path).chain_err(|| "Falied opening heightmap file."));
        let mut reader = BufReader::new(file);
        let num_samples = x_samples * y_samples;
        let mut height = Vec::with_capacity(num_samples);

        let mut min_height: CpuScalar = 0.0;
        let mut max_height: CpuScalar = 0.0;

        while height.len() < num_samples {
            let value = try!(reader.read_i16::<BigEndian>()
                .chain_err(|| {
                    "Heightmap creation failed! Could not read value from file."
                })) as CpuScalar;
            min_height = min_height.min(value);
            max_height = max_height.max(value);
            height.push(value);
        }
        let remaining = try!(reader.read_to_end(&mut Vec::new())
            .chain_err(|| "Heightmap creation failed! Could not read value from file."));
        if remaining > 0 {
            error!("Found unexpected data in heightmap file; expected {} ({} x {}) values)",
                   num_samples,
                   x_samples,
                   y_samples);
            Err(ErrorKind::UnexhaustedHeightmapFile.into())
        } else {
            info!("Heightmap len: {} [{}, {}]",
                  height.len(),
                  min_height,
                  max_height);

            Ok(Heightmap {
                height: height,
                radius: radius,
                x_max: x_samples - 1,
                y_max: y_samples - 1,
            })
        }
    }

    #[inline]
    pub fn height_at(&self, long: CpuScalar, lat: CpuScalar) -> CpuScalar {
        assert!(0.0 <= long && long <= 1.0 && 0.0 <= lat && lat <= 1.0,
                format!("{} {}", long, lat));
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

        // assert!(hxy.is_finite(),
        //         format!("long: {} lat: {} -> xy: {} {} {} {} | h: {} {} {} {} | \
        //                  hxy: {} {} {}",
        //                 long,
        //                 lat,
        //                 x0,
        //                 x1,
        //                 y0,
        //                 y1,
        //                 h00,
        //                 h01,
        //                 h10,
        //                 h11,
        //                 hx0,
        //                 hx1,
        //                 hxy));
        hxy
    }

    #[inline]
    fn discrete_height_at(&self, x: usize, y: usize) -> CpuScalar {
        self.height[y * self.x_max + x]
    }
}

impl ScalarField2 for Heightmap {
    #[inline]
    fn value_at(&self, position: &Vector2<CpuScalar>) -> CpuScalar {
        0.0
    }
}

impl ScalarField for Heightmap {
    #[inline]
    fn value_at(&self, x: CpuScalar, y: CpuScalar, z: CpuScalar) -> CpuScalar {
        let r = (x * x + y * y + z * z + 1e-4).sqrt();
        let long = (z.atan2(x) + PI) * FRAC_1_PI * 0.5;
        let lat = (y / r).acos() * FRAC_1_PI;

        r - (self.radius + self.height_at(long, lat) / 200.0)
    }
}
