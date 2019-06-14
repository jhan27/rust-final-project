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

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
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

    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
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
            let offset = w * j * 4 + off_x + off_y*w*4;
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

        let w  = self.spec.dims.width;
        let h = self.spec.dims.height;

        let mut copy = OwnedImage:: new(Dimensions{ width: w, height: h}, self.pixels().clone());

        box_blur(&mut copy, w, h, (sizes[0]-1)/2);
        box_blur(&mut copy, w, h, (sizes[1]-1)/2 );
        box_blur(&mut copy, w, h, (sizes[2]-1)/2 );

        copy
    }

    fn flipped(&self, horiz: bool, vert: bool) -> OwnedImage {
        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

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
        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

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
            let offset = w * j * 4 + off_x + off_y*w*4;
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

        let w  = self.spec.dims.width;
        let h = self.spec.dims.height;

        let mut copy = OwnedImage:: new(Dimensions{ width: w, height: h}, self.pixels().clone());

        box_blur(&mut copy, w, h, (sizes[0]-1)/2);
        box_blur(&mut copy, w, h, (sizes[1]-1)/2 );
        box_blur(&mut copy, w, h, (sizes[2]-1)/2 );

        copy
    }

    fn flipped(&self, horiz: bool, vert: bool) -> OwnedImage {
        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

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
        let width = self.spec.dims.width;
        let height = self.spec.dims.height;

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