extern crate image as rust_image;

use self::rust_image::{
    DynamicImage,
    GenericImageView
};
use snafu::{Snafu, ResultExt};


pub struct Image {
    width: u32,
    height: u32,
    matrix: RGB,
    raw_pixel: Vec<u8>,
}

// Our tentative way of storing a 3-dimensional matrix
pub struct RGB {
    r_channel: Vec<Vec<u8>>,
    g_channel: Vec<Vec<u8>>,
    b_channel: Vec<Vec<u8>>
}

pub fn import() {
    let img = match rust_image::open("test.png") {
        Ok(img) => img,
        Err(err) => return dispatch_error(ImageError::InvalidFormat)
    };

//    println!("{:?}", img.dimensions());
//    println!("{:?}", img.raw_pixels());
    let image = Image::new(img);
}

impl RGB {
    fn new(raw_pixel: &Vec<u8>, w:u32, h:u32) -> Result<Self, ImageError> {
        let (r, g, b) = match extract_channels(raw_pixel, w, h) {
            Ok((v1, v2, v3)) => (v1, v2, v3),
            Err(err) => return Err(ImageError::ParseError)
        };
        Ok(RGB {
            r_channel: r,
            g_channel: g,
            b_channel: b
        })
    }
}

fn extract_channels(pixel_vec: &Vec<u8>, w: u32, h: u32) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Vec<u8>>), ImageError> {
    let mut r = Vec::new();
    let mut g = Vec::new();
    let mut b = Vec::new();

    let mut counter = 0;

    for i in 0..h {
        let mut row_r = Vec::new();
        let mut row_g = Vec::new();
        let mut row_b = Vec::new();

        for j in 0..w {
            row_r.push(match pixel_vec.get(counter) {
                Some(val) => *val,
                None => return Err(ImageError::ParseError)
            });
            counter = counter + 1;

            row_g.push(match pixel_vec.get(counter) {
                Some(val) => *val,
                None => return Err(ImageError::ParseError)
            });
            counter = counter + 1;

            row_b.push(match pixel_vec.get(counter) {
                Some(val) => *val,
                None => return Err(ImageError::ParseError)
            });
            counter = counter + 1;

            counter = counter + 1;

        }
        r.push(row_r);
        g.push(row_g);
        b.push(row_b);
    }

    println!("matrix is {:?}", r);
    println!("rows are {}", r.len());
    Ok((r, g, b))
}


impl Image {
    fn new(image: DynamicImage) -> Self {
        let (w, h) = image.dimensions();
        Image {
            width: w,
            height: h,
            // do unwrap for now
            matrix: RGB:: new (&image.raw_pixels(), w, h).unwrap(),
            raw_pixel: image.raw_pixels()
        }
    }

    pub fn crop(&self, x:usize, y:usize, w:usize, h:usize) {
        unimplemented!()
    }

    pub fn flip_horizontal(&self) {
        unimplemented!()
    }

    pub fn flip_vertical(&self) {
        unimplemented!()
    }

    pub fn rotate(&self, angle:usize) {
        unimplemented!()
    }

    pub fn greyscale(&self) {
        unimplemented!()
    }

    pub fn blur(&self) {
        unimplemented!()
    }

    pub fn lighting_correction(&self) {
        unimplemented!()
    }

}


fn dispatch_error(err: ImageError) {
    println!("{}", err);
}

#[derive(Debug, Snafu)]
pub enum ImageError {
    #[snafu(display("Can't open image because of invalid format"))]
    InvalidFormat,

    #[snafu(display("Can't parse the raw pixel vector into a matrix"))]
    ParseError
}