use image as rust_image;
use rayon::prelude::*;
use image::{DynamicImage, GenericImageView};
use std::{cmp};
use snafu::{Snafu, ResultExt};

type Matrix<T> = Box<[T]>;


/******************************* OwnedImage Struct and Member Functions *******************************/
/******************************************************************************************************/

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

impl OwnedImage {

    pub fn new (dims: Dimensions, pixels: Matrix<u8>) -> OwnedImage {
        OwnedImage {
            dims,
            pixels,
        }
    }

    pub fn import(image:DynamicImage) -> OwnedImage {
        let (w, h) = image.dimensions();
        OwnedImage {
            dims: Dimensions { width: w, height: h},
            pixels: image.raw_pixels().into_boxed_slice(),
        }
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        let w = self.dims.width;
        let h = self.dims.height;

        if x >= w && y >= h{
            dispatch_error(ImageError::IndexOutOfBound)
        }
        let index = ((y*w*4) + 4 * x) as usize;
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
        if x >= w || y >= h{
            dispatch_error(ImageError::IndexOutOfBound)
        }
        let index = ((y*w*4) + 4 * x) as usize;
        self.pixels[index] = pix.r;
        self.pixels[index+1] = pix.g;
        self.pixels[index+2] = pix.b;
        self.pixels[index+3] = pix.alpha;
    }

    pub fn save_image(&self){
        let image = match image::save_buffer("processed.png", &self.pixels,
                                             self.dims.width, self.dims.height, image::RGBA(8)) {
            Ok(img) => {img}
            Err(err) => return dispatch_error(ImageError::ImageOperationFailed)
        };
    }
}

/******************************************* Image Trait **********************************************/
/******************************************************************************************************/

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

        let spec = SliceSpec {offset: (x*4, y*4),
            dims: Dimensions{ width: w, height: h},
            stride: (img_dims.width, img_dims.height),
            orientation: Orientation:: I};

        ImageSlice {pixels: &self.get_pixels(), spec}
    }

    fn blurred(&self, amount: u32) -> Self::Owned {
        unimplemented!()
    }

    fn flipped(&self, horiz: bool, vert: bool) -> Self::Owned {
        let width = self.dims.width;
        let height = self.dims.height;

        let mut pixels = Vec::new();

        for j in 0..height {
            for i in 0..width {
                let p;
                if horiz && vert {
                    p = self.get_pixel(width-i-1, height-j-1);
                } else if horiz {
                    p = self.get_pixel(width-i-1, j);
                } else if vert {
                    p = self.get_pixel(i, height-j-1);
                } else {
                    p = self.get_pixel(i, j);
                }
                pixels.push(p.r);
                pixels.push(p.g);
                pixels.push(p.b);
                pixels.push(p.alpha);
            }
        }

        OwnedImage:: new(Dimensions{ width, height}, pixels.into_boxed_slice())
    }
}


/***************************************** ImageMut Trait *********************************************/
/******************************************************************************************************/

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
    fn crop_mut(&mut self, x: u32, y: u32, dims: Dimensions) -> ImageSliceMut {
        unimplemented!()
    }

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

/***************************************** Other Structs **********************************************/
/******************************************************************************************************/

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
    orientation: Orientation,
}

/// Four quadrants of an image.
pub struct SplitMut<'a> {
    pub top_left: ImageSliceMut<'a>,
    pub bottom_left: ImageSliceMut<'a>,
    pub top_right: ImageSliceMut<'a>,
    pub bottom_right: ImageSliceMut<'a>,

}

enum Orientation {
    I, H, V, HV, R, RH, RV, RHV
}

impl<'a> ImageSlice<'a> {

    pub fn transform_pixel_index(&self) -> Matrix<u8>{
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

        buffer.into_boxed_slice()
    }
}

/****************************************** Functions *************************************************/
/******************************************************************************************************/

pub fn import() {
    let img = match rust_image::open("test.png") {
        Ok(img) => img,
        Err(err) => return dispatch_error(ImageError::InvalidFormat)
    };

    let image = OwnedImage::import(img);

    let flipped = image.flipped(false, true);
    flipped.save_image();
}


/**************************************** Error Handling **********************************************/
/******************************************************************************************************/

fn dispatch_error(err: ImageError) {
    println!("{}", err);
}

#[derive(Debug, Snafu)]
pub enum ImageError {
    #[snafu(display("Can't open image because of invalid format"))]
    InvalidFormat,

    #[snafu(display("Can't parse the raw pixel vector into a matrix"))]
    ParseError,

    #[snafu(display("Index out of bound"))]
    IndexOutOfBound,

    #[snafu(display("Can not perform image operation"))]
    ImageOperationFailed,
}