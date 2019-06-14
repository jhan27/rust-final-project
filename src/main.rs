use image_processor::{
    utils,
    traits::*,
    OwnedImage::OwnedImage,
    ImageSlice
};

fn main() {

    /// example use of our image processor library
    let image = match utils::import("test.png") {
        Ok(image) => image,
        Err(err) => return utils::dispatch_error(err)
    };

    /****** Immutable Operations ******/

    /// Operation 1: Immutable crop, returns new cropped image
    /// If input dimensions out of bound, will use the image's own width/height to perform cropping
    let slice = image.crop(100, 100, utils::Dimensions{width: 500, height: 400});
    utils::save_image(&slice, "cropped.png");

    /// Operation 2: Immutable flip, horizontally, vertically, and both
    let horiz = image.flipped(true, false);
    let verti = image.flipped(true, false);
    let both = image.flipped(true, true);
    utils::save_image(&horiz, "horiz_flipped.png");
    utils::save_image(&verti, "verti_flipped.png");
    utils::save_image(&both, "horiz_verti_flipped.png");

    /// Operation 3: Immutable greyscale
    let grey = image.greyscale();
    utils::save_image(&grey, "greyscale.png");

    /// Operation 4: Immutable blur
    /// amount parameter should be between 0-10
    let blurred = image.blurred(3);
    utils::save_image(&blurred, "blurred.png");

    /****** Mutable Operations ******/

    /// Can make an empty image
    let mut image_copy = OwnedImage::new(
        utils::Dimensions{width: 0, height: 0}, Vec::new().into_boxed_slice());

    /// Copy image into image_copy
    image_copy.copy_from(&image);

    /// Operation 1: Mutable flip
    image_copy.flip(true, false);
    utils::save_image(&image_copy, "horiz_flipped_mut.png");

    /// Operation 2: Mutable greyscale
    image_copy.greyscale_mut();
    utils::save_image(&image_copy, "greyscale_mut.png");

    /// Operation 3: Mutable blur
    image_copy.blur(2);
    utils::save_image(&image_copy, "blurred_mut.png");


    /****** Slice Operations ******/

    /// Perform crop and then greyscale
    let crop_grey = image.crop(100, 100, utils::Dimensions{width: 500, height: 400})
        .greyscale();
    utils::save_image(&crop_grey, "cropped_greyscale.png");

    /// Perform crop and then blur
    let crop_blur = image.crop(100, 100, utils::Dimensions{width: 500, height: 400})
        .blurred(3);
    utils::save_image(&crop_blur, "cropped_blurred.png");

    /// Perform crop and then flip
    let crop_flip = image.crop(100, 100, utils::Dimensions{width: 500, height: 400})
        .flipped(true,false);
    utils::save_image(&crop_flip, "cropped_flipped.png");

}

