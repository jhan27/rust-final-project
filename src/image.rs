use image as rust_image;
use rayon::prelude::*;
use image::{DynamicImage, ImageBuffer, GenericImageView};
use core::borrow::Borrow;
use std::{borrow::Cow::Owned,
          cmp};
use snafu::{Snafu, ResultExt};
use std::sync::mpsc::TryRecvError::Disconnected;

type Matrix<T> = Box<[T]>;

/// An owned image.
pub struct OwnedImage {
    dims: Dimensions,
    pixels: Matrix<u8>,
}

pub struct Dimensions {
    width: u32,
    height: u32,
}

pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    alpha: u8
}

/// A borrowed slice of an image.
pub struct ImageSlice<'a> {
    pixels: &'a [u8],
    spec: SliceSpec,
}

/// A mutable, borrowed slice of an image.
pub struct ImageSliceMut<'a> {
    pixels: &'a mut [u8],
    spec: SliceSpec,
}

pub struct SliceSpec {
    offset: (u32, u32),
    stride: (u32, u32),
    dims: Dimensions,
}

/// Immutable image operations.
pub trait Image {
    /// The type of new images generated from this image.
    type Owned: ImageMut;

    /// This image's width and height.
    fn dimensions(&self) -> &Dimensions;

    /// Borrows a sub-image.
    fn crop(&self, x: u32, y: u32, dims: Dimensions) -> ImageSlice;

    /// Returns a new image that is this one blurred.
    fn blurred(&self, amount: u32) -> Self::Owned;

    /// Returns a new image that is this one flipped.
    fn flipped(&self, horiz: bool, vert: bool) -> Self::Owned;
}

impl OwnedImage {
    pub fn new(image:DynamicImage) -> OwnedImage {
        let (w, h) = image.dimensions();
        OwnedImage {
            dims: Dimensions { width: w, height: h},
            pixels: image.raw_pixels().into_boxed_slice(),
        }
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel{
        let w = self.dims.width;
        let h = self.dims.height;

        if x < w && y < h{
            dispatch_error(ImageError::IndexOutOfBound)
        }
        let index = ((x*w*4) + 4 * y) as usize;
        Pixel {
            r: self.pixels[index],
            g: self.pixels[index+1],
            b: self.pixels[index+2],
            alpha: self.pixels[index+3],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, pix: Pixel) {
        let w = self.dims.width;
        let h = self.dims.height;
        if x < w && y < h{
            dispatch_error(ImageError::IndexOutOfBound)
        }
        let index = ((x*w*4) + 4 * y) as usize;
        self.pixels[index] = pix.r;
        self.pixels[index+1] = pix.g;
        self.pixels[index+2] = pix.b;
        self.pixels[index+3] = pix.alpha;
    }
}

impl<'a> ImageSlice<'a> {

    pub fn generate_image(&self) {
        let (off_x, off_y)= self.spec.offset;
        let (w, h) = self.spec.stride;
        let dims = &self.spec.dims;

        let mut buffer = Vec::new();

        for j in 0..dims.height {
            let offset = w * j * 4 + off_x + off_y*w*4;
            for i in 0..dims.width * 4 {
                buffer.push(self.pixels[(offset+i) as usize]);
            }
        }

        let image = match image::save_buffer("image.png", &buffer.into_boxed_slice(),
                                             dims.width, dims.height, image::RGBA(8)) {
            Ok(img) => {img}
            Err(err) => return dispatch_error(ImageError::CropFailed)
        };

        let img = match rust_image::open("image.png") {
            Ok(img) => img,
            Err(err) => return dispatch_error(ImageError::InvalidFormat)
        };

//        println!("after crop image is: {:?}", img.raw_pixels());
    }
}

impl Image for OwnedImage{

    type Owned = OwnedImage;

    fn dimensions(&self) -> &Dimensions {
        &self.dims
    }

    fn crop(&self, x: u32, y: u32, dims: Dimensions) -> ImageSlice {
        let img_dims = self.dimensions();
        let x = cmp::min(img_dims.width, x);
        let y = cmp::min(img_dims.height, y);

        let w = cmp::min(dims.width, img_dims.width - x);
        let h = cmp::min(dims.height, img_dims.height - y);

        let slice_spec = SliceSpec {offset: (x*4, y*4),
            dims: dims,
            stride: (img_dims.width, img_dims.height)};

        ImageSlice {pixels: self.get_pixels(), spec: slice_spec}
    }

    fn blurred(&self, amount: u32) -> Self::Owned {
        unimplemented!()
    }

    fn flipped(&self, horiz: bool, vert: bool) -> Self::Owned {
        unimplemented!()
    }
}


/// Four quadrants of an image.
pub struct SplitMut<'a> {
    pub top_left: ImageSliceMut<'a>,
    pub bottom_left: ImageSliceMut<'a>,
    pub top_right: ImageSliceMut<'a>,
    pub bottom_right: ImageSliceMut<'a>,

}

pub trait ImageMut: Image {
    /// Mutable borrows a sub-image.
    fn crop_mut(&mut self, x: u32, y: u32, dims: Dimensions) -> ImageSliceMut;

    /// Splits an image four ways, to work on each quadrant in parallel.
    /// The pixel at (`x`, `y`) becomes the top-left pixel of the bottom-right
    /// quadrant.
    fn split_mut(&mut self, x: u32, y: u32) -> SplitMut;

    /// Blurs this image in place.
    fn blur(&mut self, amount: u32);

    /// Flips this image in place.
    fn flip(&mut self, horiz: bool, vert: bool);

    /// Blurs this image into another, existing image.
    /// (You can probably override this with something faster.)
    fn blur_from(&mut self, amount: u32, source: impl Image) {
        self.blit_from(source);
        self.blur(amount);
    }

    /// Flips this image into another, existing image.
    /// (You can probably override this with something faster.)
    fn flip_from(&mut self, horiz: bool, vert: bool, source: impl Image) {

        self.blit_from(source);
        self.flip(horiz, vert);
    }

    /// Copies into this image from another, existing image.
    /// (Do dimensions need to match?)
    fn blit_from(&mut self, source: impl Image);

}

impl ImageMut for OwnedImage {
    fn crop_mut(&mut self, x: u32, y: u32, dims: Dimensions) -> ImageSliceMut {unimplemented!()}

    fn split_mut(&mut self, x: u32, y: u32) -> SplitMut {unimplemented!()}

    fn blur(&mut self, amount: u32) {unimplemented!()}

    fn flip(&mut self, horiz: bool, vert: bool) {unimplemented!()}

    fn blur_from(&mut self, amount: u32, source: impl Image) {
        self.blit_from(source);
        self.blur(amount);
    }

    fn flip_from(&mut self, horiz: bool, vert: bool, source: impl Image) {

        self.blit_from(source);
        self.flip(horiz, vert);
    }

    fn blit_from(&mut self, source: impl Image) {unimplemented!()}
}

enum Orientation {
    I, H, V, HV, R, RH, RV, RHV
}

pub fn import() {
    let img = match rust_image::open("test.png") {
        Ok(img) => img,
        Err(err) => return dispatch_error(ImageError::InvalidFormat)
    };

//    println!("original image is: {:?}", img.raw_pixels());
    let image = OwnedImage::new(img);
    println!("width is: {}", image.dims.width);
    println!("height is: {}", image.dims.height);

    let slice = image.crop(50, 100, Dimensions {width: 300, height: 500 });
    slice.generate_image();
}


fn dispatch_error(err: ImageError) {
    println!("{}", err);
}

#[derive(Debug, Snafu)]
pub enum ImageError {
    #[snafu(display("Can't open image because of invalid format"))]
    InvalidFormat,

    #[snafu(display("Can't parse the raw pixel vector into a matrix"))]
    ParseError,

    #[snafu(display("Can't parse the raw pixel vector into a matrix"))]
    IndexOutOfBound,

    #[snafu(display("Can not perform cropping"))]
    CropFailed,
}