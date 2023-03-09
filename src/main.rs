use clap::{error::ErrorKind, CommandFactory, Parser};
use image::{io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};

#[derive(Parser)]
struct Cli {
    /// The path of the image that you want to process
    path: std::path::PathBuf,

    /// The scale factor by which the image will be scaled down (must be a power of two)
    scale_factor: u8,

    /// Keep the dimensions of the output image the same as the input
    #[clap(long, short, action)]
    keep_dimensions: bool,

    /// Force crop the image in order for it to be divisible by the scale factor
    #[clap(long, short, action)]
    force_crop: bool,

    /// Centre the image if cropping is required
    #[clap(long, short, action)]
    centre: bool,

    /// Use all optional flags
    #[clap(long, short, action)]
    all: bool,
}

fn main() {
    let arguments = Cli::parse();
    let mut command = Cli::command();

    let (path, scale_factor, keep_dimensions, force_crop, centre) = (
        arguments.path,
        arguments.scale_factor,
        arguments.keep_dimensions || arguments.all,
        arguments.force_crop || arguments.all,
        arguments.centre || arguments.all,
    );

    if scale_factor < 2 || scale_factor > 8 {
        command
            .error(
                ErrorKind::InvalidValue,
                "scale factor must be between 2 and 8",
            )
            .exit();
    }

    let mut image = match ImageReader::open(&path) {
        Ok(file) => match file.decode() {
            Ok(image) => image,
            Err(_) => {
                command
                    .error(ErrorKind::Io, "could not decode image")
                    .exit();
            }
        },
        Err(_) => {
            command.error(ErrorKind::Io, "could not open file").exit();
        }
    };

    let (mut width, mut height) = (image.width(), image.height());

    if width % (scale_factor as u32) != 0 || height % (scale_factor as u32) != 0 {
        if !force_crop {
            command
            .error(
                ErrorKind::InvalidValue,
                "image dimensions must be divisible by scale factor. You can force crop the image using the -f flag",
            )
            .exit();
        } else {
            image = crop_image(&mut image, scale_factor, centre);
            (width, height) = (image.width(), image.height());
        }
    }

    let (new_width, new_height) = (
        width / (scale_factor as u32),
        height / (scale_factor as u32),
    );

    let mut new_image = if keep_dimensions {
        image::DynamicImage::new_rgb8(width, height)
    } else {
        image::DynamicImage::new_rgb8(new_width, new_height)
    };

    for x in 0..new_width {
        for y in 0..new_height {
            if keep_dimensions {
                let mut pixels: Vec<Rgba<u8>> =
                    vec![Rgba([0, 0, 0, 0]); (scale_factor * scale_factor) as usize];

                for i in 0..scale_factor {
                    for j in 0..scale_factor {
                        pixels[(i * scale_factor + j) as usize] = image.get_pixel(
                            x * (scale_factor as u32) + i as u32,
                            y * (scale_factor as u32) + j as u32,
                        );
                    }
                }

                let pixel = average_pixels(&pixels);

                for i in 0..scale_factor {
                    for j in 0..scale_factor {
                        new_image.put_pixel(
                            x * (scale_factor as u32) + i as u32,
                            y * (scale_factor as u32) + j as u32,
                            pixel,
                        );
                    }
                }
            } else {
                let pixel = image.get_pixel(x * (scale_factor as u32), y * (scale_factor as u32));

                new_image.put_pixel(x, y, pixel);
            }
        }
    }

    let original_file_name = &path.file_name().unwrap().to_str().unwrap();

    match new_image.save(format!("pixelated_{}", original_file_name)) {
        Ok(_) => return,
        Err(_) => command.error(ErrorKind::Io, "could not save image").exit(),
    };
}

fn crop_image(image: &mut DynamicImage, scale_factor: u8, centre: bool) -> DynamicImage {
    let (width, height) = (image.width(), image.height());

    let (new_width, new_height) = (
        width - (width % scale_factor as u32),
        height - (height % scale_factor as u32),
    );

    let (x_offset, y_offset) = if centre {
        ((width - new_width) / 2, (height - new_height) / 2)
    } else {
        (0, 0)
    };

    image.crop(x_offset, y_offset, new_width, new_height)
}

fn average_pixels(pixels: &Vec<Rgba<u8>>) -> Rgba<u8> {
    let mut red = 0;
    let mut green = 0;
    let mut blue = 0;
    let mut alpha = 0;

    for pixel in pixels {
        red += pixel[0] as u32;
        green += pixel[1] as u32;
        blue += pixel[2] as u32;
        alpha += pixel[3] as u32;
    }

    let pixel_count = pixels.len() as u32;

    Rgba([
        (red / pixel_count) as u8,
        (green / pixel_count) as u8,
        (blue / pixel_count) as u8,
        (alpha / pixel_count) as u8,
    ])
}
