use super::OwnedImage::*;
use super::traits::*;
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


pub fn import(name: &str) -> Result<OwnedImage, ImageError> {
    match rust_image::open("test.png") {
        Ok(img) => return Ok(OwnedImage::import(img)),
        Err(err) => return Err(ImageError::InvalidFormat)
    };
}

pub fn save_image(source: &impl Image, name: &str) {
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