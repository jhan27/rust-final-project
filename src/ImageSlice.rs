use super::traits::*;
use super::OwnedImage::*;
use super::utils::*;
use std::{cmp};

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

impl<'a> ImageSlice<'a> {

    pub fn new (pixels: &'a [u8], spec: SliceSpec) -> Self {
        ImageSlice {
            pixels,
            spec,
        }
    }

    fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        let w = self.spec.dims.width;
        let h = self.spec.dims.width;
        let (off_x, off_y) = self.spec.offset;

        // if x, y not contained in the current slice
        if  x < off_x  || y < off_y || (x >= w+off_x || y >= h + off_y) {
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
}

impl<'a> ImageSliceMut<'a> {

    pub fn new (pixels: &'a mut [u8], spec: SliceSpec) -> Self {
        ImageSliceMut {
            pixels,
            spec,
        }
    }

    fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        let w = self.spec.dims.width;
        let h = self.spec.dims.width;
        let (off_x, off_y) = self.spec.offset;

        // if x, y not contained in the current slice
        if  x < off_x  || y < off_y || (x >= w+off_x || y >= h + off_y) {
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
}

impl<'a> Image for ImageSlice<'a> {

    fn dimensions(&self) -> &Dimensions {
        &self.spec.dims
    }

    fn pixels(&self) -> Matrix<u8> {
        let (off_x, off_y)= self.spec.offset;
        let (w, _) = self.spec.size;
        let dims = &self.spec.dims;

        let mut buffer = Vec::new();

        for j in 0..dims.height {
            let offset = w * j * 4 + off_x*4 + off_y*w*4;
            for i in 0..dims.width * 4 {
                buffer.push(self.pixels[(offset+i) as usize]);
            }
        }

        buffer.into_boxed_slice()
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

        ImageSlice {pixels: &self.pixels, spec}
    }

    fn blurred(&self, amount: u32) -> OwnedImage {

        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

        let owned = OwnedImage::new(Dimensions{ width, height}, self.pixels());
        owned.blurred(amount)
    }

    fn flipped(&self, horiz: bool, vert: bool) -> OwnedImage {

        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

        let owned = OwnedImage::new(Dimensions{ width, height}, self.pixels());
        owned.flipped(horiz, vert)
    }

    fn greyscale(&self) -> OwnedImage {

        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

        let owned = OwnedImage::new(Dimensions{ width, height}, self.pixels());
        owned.greyscale()
    }
}

impl<'a> Image for ImageSliceMut<'a> {
    fn dimensions(&self) -> &Dimensions {
        &self.spec.dims
    }

    fn pixels(&self) -> Matrix<u8> {
        let (off_x, off_y)= self.spec.offset;
        let (w, _) = self.spec.size;
        let dims = &self.spec.dims;

        let mut buffer = Vec::new();

        for j in 0..dims.height {
            let offset = w * j * 4 + off_x*4+ off_y*w*4;
            for i in 0..dims.width * 4 {
                buffer.push(self.pixels[(offset+i) as usize]);
            }
        }

        buffer.into_boxed_slice()
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

        ImageSlice {pixels: &self.pixels, spec}
    }

    fn blurred(&self, amount: u32) -> OwnedImage {

        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

        let owned = OwnedImage::new(Dimensions{ width, height}, self.pixels());
        owned.blurred(amount)
    }

    fn flipped(&self, horiz: bool, vert: bool) -> OwnedImage {

        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

        let owned = OwnedImage::new(Dimensions{ width, height}, self.pixels());
        owned.flipped(horiz, vert)
    }

    fn greyscale(&self) -> OwnedImage {

        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

        let owned = OwnedImage::new(Dimensions{ width, height}, self.pixels());
        owned.greyscale()
    }
}