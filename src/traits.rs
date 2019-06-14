use super::OwnedImage::*;
use super::ImageSlice::*;
use super::utils::*;

type Matrix<T> = Box<[T]>;

/// Immutable image operations.
pub trait Image {
    /// This image's width and height.
    fn dimensions(&self) -> &Dimensions;

    /// This image's pixels.
    fn pixels(&self) -> Matrix<u8>;

    /// Borrows a sub-image.
    fn crop(&self, x: u32, y: u32, dims: Dimensions) -> ImageSlice;

    /// Returns a new image that is this one blurred.
    fn blurred(&self, amount: u32) -> OwnedImage;

    /// Returns a new image that is this one flipped.
    fn flipped(&self, horiz: bool, vert: bool) -> OwnedImage;

    /// Return s new image that is this one in grayscale
    fn greyscale(&self) -> OwnedImage;
}

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