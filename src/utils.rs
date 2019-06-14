use super::OwnedImage::*;
use super::traits::*;
use std::{cmp};
use snafu::{Snafu};
use image as rust_image;

/************************************ Shared Struct and Functions *************************************/
/******************************************************************************************************/

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: u8
}

pub struct SliceSpec {
    pub offset: (u32, u32),     // x and y offsets
    pub size: (u32, u32),       // this is the dimension of original owned image
    pub dims: Dimensions,       // this is the dimension of the slice
}

pub fn box_blur(image: &mut OwnedImage, w: u32, h:u32, r:i32){
    for j in 0..h as i32{
        for i in 0..w as i32{
            let initial_pixel = image.get_pixel(i as u32, j as u32);
            let total_ct= ((r+r+1)*(r+r+1)) as u32;
            let mut r_total = 0;
            let mut g_total = 0;
            let mut b_total = 0;
            for iy in (j-r)..(j+r+1){
                for ix in (i-r)..(i+r+1){
                    let x = cmp::min(w-1, cmp::max(0, ix as u32));
                    let y = cmp::min(h-1, cmp::max(0, iy as u32));
                    let this_pixel = image.get_pixel(x as u32, y as u32);

                    r_total += this_pixel.r as u32;
                    b_total += this_pixel.b as u32;
                    g_total += this_pixel.g as u32;
                }
            }
            image.set_pixel(i as u32, j as u32, Pixel{
                r: (r_total/total_ct) as u8,
                g: (g_total/total_ct) as u8,
                b:(b_total/total_ct) as u8,
                alpha: initial_pixel.alpha,
            });

        }
    }

}


pub fn import() {
    let img = match rust_image::open("test.png") {
        Ok(img) => img,
        Err(err) => return dispatch_error(ImageError::InvalidFormat)
    };

    let image = OwnedImage::import(img);

    let slice = image.crop(100, 100, Dimensions{width: 100, height: 100});

    save_image(slice, "processed.png");

}

pub fn save_image(source: impl Image, name: &str) {
    let pixels = source.pixels();
    match image::save_buffer(name, &*pixels, source.dimensions().width, source.dimensions().height,image::RGBA(8)) {
        Ok(img) => {img}
        Err(err) => return dispatch_error(ImageError::ImageOperationFailed)
    };
}


pub fn dispatch_error(err: ImageError) {
    println!("{}", err);
}

#[derive(Debug, Snafu)]
pub enum ImageError {
    #[snafu(display("Can't open image due to error"))]
    InvalidFormat,

    #[snafu(display("Can't parse the raw pixel vector into a matrix"))]
    ParseError,

    #[snafu(display("Index out of bound"))]
    IndexOutOfBound,

    #[snafu(display("Can not perform image operation"))]
    ImageOperationFailed,
}