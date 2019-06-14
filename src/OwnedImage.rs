use super::traits::*;
use super::ImageSlice::*;
use super::utils::*;
use std::{cmp};
pub type Matrix<T> = Box<[T]>;

use image::{DynamicImage, GenericImageView};


/******************************* OwnedImage Struct and Member Functions *******************************/
/******************************************************************************************************/

/// An owned image.
pub struct OwnedImage {
    dims: Dimensions,
    pixels: Matrix<u8>,
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

    pub fn get_pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        let w = self.dims.width;
        let h = self.dims.height;

        if x >= w || y >= h{
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
}

/************************************* Image Trait for OwnedImage *************************************/
/******************************************************************************************************/

impl Image for OwnedImage{

    fn dimensions(&self) -> &Dimensions {
        &self.dims
    }

    fn pixels(&self) -> Matrix<u8> {
        self.pixels.clone()
    }

    fn crop(&self, x: u32, y: u32, dims: Dimensions) -> ImageSlice {
        let img_dims = self.dimensions();
        let x = cmp::min(img_dims.width, x);
        let y = cmp::min(img_dims.height, y);

        let w = cmp::min(dims.width, img_dims.width - x);
        let h = cmp::min(dims.height, img_dims.height - y);

        let spec = SliceSpec {offset: (x*4, y*4),
            dims: Dimensions{ width: w, height: h},
            size: (img_dims.width, img_dims.height)};

        ImageSlice::new(&self.get_pixels(), spec)
    }

    fn blurred(&self, amount: u32) -> OwnedImage {
        let sigma = amount as f32;
        let n = 3.0;
        let wideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
        let wl = wideal.floor();
        let wu = wl + 2.0;

        let mideal = (12.0 * sigma * sigma - n * wl * wl - 4.0*n*wl-3.0*n) / (-4.0*wl - 4.0);
        let m = mideal.ceil();

        let mut sizes: Vec<i32> = Vec::new();
        for i in 0..n as i32{
            if i < m as i32 {
                sizes.push(wl as i32);
            } else {
                sizes.push(wu as i32);
            }
        }

        let w  = self.dims.width;
        let h = self.dims.height;

        let mut copy = OwnedImage:: new(Dimensions{ width: w, height: h}, self.pixels.clone());

        box_blur(&mut copy, w, h, (sizes[0]-1)/2);
        box_blur(&mut copy, w, h, (sizes[1]-1)/2 );
        box_blur(&mut copy, w, h, (sizes[2]-1)/2 );

        copy
    }

    fn flipped(&self, horiz: bool, vert: bool) -> OwnedImage {
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

    fn greyscale(&self) -> OwnedImage {
        let width = self.dims.width;
        let height = self.dims.height;

        let mut pixels = Vec::new();

        for j in 0..height {
            for i in 0..width {
                let p = self.get_pixel(i,j);
                let avg = 0.299 * (p.r as f32) + 0.587 * (p.g as f32) + 0.114 * (p.b as f32);
                let avg = avg as u8;
                pixels.push(avg);
                pixels.push(avg);
                pixels.push(avg);
                pixels.push(p.alpha);
            }
        }

        OwnedImage:: new(Dimensions{ width, height}, pixels.into_boxed_slice())
    }
}

/*********************************** ImageMut Trait for OwnedImage ************************************/
/******************************************************************************************************/

pub trait ImageMut: Image {
    /// Mutable borrows a sub-image.
    fn crop_mut(&mut self, x: u32, y: u32, dims: Dimensions) -> ImageSliceMut;

    /// Blurs this image in place.
    fn blur(&mut self, amount: u32);

    /// Flips this image in place.
    fn flip(&mut self, horiz: bool, vert: bool);

    /// Turn this image into greyscale in place.
    fn greyscale_mut(&mut self);

    /// Blurs this image into another, existing image.
    /// (You can probably override this with something faster.)
    fn blur_from(&mut self, amount: u32, source: impl Image) {
        self.copy_from(source);
        self.blur(amount);
    }

    /// Flips this image into another, existing image.
    /// (You can probably override this with something faster.)
    fn flip_from(&mut self, horiz: bool, vert: bool, source: impl Image) {

        self.copy_from(source);
        self.flip(horiz, vert);
    }

    /// Copies into this image from another, existing image.
    /// (Do dimensions need to match?)
    fn copy_from(&mut self, source: impl Image);
}

impl ImageMut for OwnedImage {
    fn crop_mut(&mut self, x: u32, y: u32, dims: Dimensions) -> ImageSliceMut {
        let img_dims = self.dimensions();
        let x = cmp::min(img_dims.width, x);
        let y = cmp::min(img_dims.height, y);

        let w = cmp::min(dims.width, img_dims.width - x);
        let h = cmp::min(dims.height, img_dims.height - y);

        let spec = SliceSpec {offset: (x*4, y*4),
            dims: Dimensions{ width: w, height: h},
            size: (img_dims.width, img_dims.height)};

        ImageSliceMut::new(self.get_pixels_mut(), spec)
    }

    fn blur(&mut self, amount: u32) {
        let sigma = amount as f32;
        let n = 3.0;
        let wideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
        let wl = wideal.floor();
        let wu = wl + 2.0;

        let mideal = (12.0 * sigma * sigma - n * wl * wl - 4.0*n*wl-3.0*n) / (-4.0*wl - 4.0);
        let m = mideal.ceil();

        let mut sizes: Vec<i32> = Vec::new();
        for i in 0..3{
            if i < m as i32 {
                sizes.push(wl as i32);
            } else {
                sizes.push(wu as i32);
            }
        }

        let w  = self.dims.width;
        let h = self.dims.height;

        box_blur(self, w, h, (sizes[0]-1)/2);
        box_blur(self, w, h, (sizes[1]-1)/2 );
        box_blur(self, w, h, (sizes[2]-1)/2 );
    }

    fn flip(&mut self, horiz: bool, vert: bool) {
        let width = self.dims.width;
        let height = self.dims.height;

        for j in 0..height {
            for i in 0..(width+1)/2 {
                let saved = self.get_pixel(i, j);

                let p;
                if horiz && vert {
                    p = self.get_pixel(width-i-1, height-j-1);
                    self.set_pixel(width-i-1, height-j-1, saved);
                } else if horiz {
                    p = self.get_pixel(width-i-1, j);
                    self.set_pixel(width-i-1, j, saved);
                } else if vert {
                    p = self.get_pixel(i, height-j-1);
                    self.set_pixel(i, height-j-1, saved);
                } else {
                    break;
                }
                self.set_pixel(i, j, p);
            }
        }
    }

    fn greyscale_mut(&mut self) {
        let width = self.dims.width;
        let height = self.dims.height;

        for j in 0..height {
            for i in 0..width {
                let p = self.get_pixel(i,j);
                let avg = 0.299 * (p.r as f32) + 0.587 * (p.g as f32) + 0.114 * (p.b as f32);
                let avg = avg as u8;
                self.set_pixel(i, j, Pixel {
                    r:avg,
                    g:avg,
                    b:avg,
                    alpha:p.alpha
                });
            }
        }
    }

    fn blur_from(&mut self, amount: u32, source: impl Image) {
        self.copy_from(source);
        self.blur(amount);
    }

    fn flip_from(&mut self, horiz: bool, vert: bool, source: impl Image) {
        self.copy_from(source);
        self.flip(horiz, vert);
    }

    fn copy_from(&mut self, source: impl Image) {
        self.dims = Dimensions {
            width: source.dimensions().width,
            height: source.dimensions().height
        };
        self.pixels = source.pixels().clone()
    }
}
