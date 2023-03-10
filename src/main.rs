use clap::{error::ErrorKind, Command, CommandFactory, Parser};
use image::{io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

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

    /// Overwrite the input image
    #[clap(long, short, action)]
    overwrite: bool,

    /// Use all optional flags
    #[clap(long, short, action)]
    all: bool,
}

#[derive(PartialEq)]
enum ErrorResponse {
    Ignore,
    Exit,
}

fn main() {
    let arguments = Cli::parse();
    let mut command = Cli::command();

    let (path, scale_factor, keep_dimensions, force_crop, centre, overwrite) = (
        arguments.path,
        arguments.scale_factor,
        arguments.keep_dimensions || arguments.all,
        arguments.force_crop || arguments.all,
        arguments.centre || arguments.all,
        arguments.overwrite || arguments.all,
    );

    if scale_factor < 2 || scale_factor > 8 {
        command
            .error(
                ErrorKind::InvalidValue,
                "scale factor must be between 2 and 8",
            )
            .exit();
    }

    let path_metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => {
            command.error(ErrorKind::Io, "could not open file").exit();
        }
    };

    if path_metadata.is_file() {
        process_image(
            &mut command,
            scale_factor,
            &path,
            keep_dimensions,
            force_crop,
            centre,
            overwrite,
            ErrorResponse::Exit,
        );

        return;
    } else if path_metadata.is_dir() {
        let paths = match fs::read_dir(&path) {
            Ok(paths) => paths,
            Err(_) => {
                command
                    .error(ErrorKind::Io, "could not read directory")
                    .exit();
            }
        };

        for path in paths {
            let path = match path {
                Ok(path) => path,
                Err(_) => {
                    command
                        .error(ErrorKind::Io, "could not read directory")
                        .exit();
                }
            };

            let path = path.path();

            if path.is_file() {
                process_image(
                    &mut command,
                    scale_factor,
                    &path,
                    keep_dimensions,
                    force_crop,
                    centre,
                    overwrite,
                    ErrorResponse::Ignore,
                );
            }
        }
    }
}

fn process_image(
    command: &mut Command,
    scale_factor: u8,
    path: &PathBuf,
    keep_dimensions: bool,
    force_crop: bool,
    centre: bool,
    overwrite: bool,
    error_response: ErrorResponse,
) {
    let mut image = match ImageReader::open(&path) {
        Ok(file) => match file.decode() {
            Ok(image) => image,
            Err(_) => match error_response {
                ErrorResponse::Exit => {
                    command
                        .error(ErrorKind::Io, "could not decode image")
                        .exit();
                }
                ErrorResponse::Ignore => {
                    log(
                        &*format!("could not decode image at '{}'; skipping", path.display()),
                        LogType::Error,
                    );

                    return;
                }
            },
        },
        Err(_) => match error_response {
            ErrorResponse::Exit => {
                command.error(ErrorKind::Io, "could not open file").exit();
            }
            ErrorResponse::Ignore => {
                log(
                    &*format!("could not open file at '{}'; skipping", path.display()),
                    LogType::Error,
                );

                return;
            }
        },
    };

    let (mut width, mut height) = (image.width(), image.height());

    if width % (scale_factor as u32) != 0 || height % (scale_factor as u32) != 0 {
        if !force_crop {
            match error_response {
                ErrorResponse::Exit => {
                    command
                        .error(ErrorKind::Io, "image dimensions must be divisible by scale factor. you can force crop the image using the -f flag")
                        .exit();
                }
                ErrorResponse::Ignore => {
                    log(
                        &*format!("image dimensions at '{}' were not divisible by the scale factor. you can force crop the image using the -f flag", path.display()),
                        LogType::Error,
                    );

                    return;
                }
            }
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

    let directory = &path
        .ancestors()
        .nth(1)
        .unwrap_or_else(|| Path::new("."))
        .display();

    let original_file_name = &path.file_name().unwrap().to_str().unwrap();

    let file_name = if overwrite {
        format!("{}", original_file_name)
    } else {
        format!("pixelated_{}", original_file_name)
    };

    match new_image.save(format!("{}/{}", directory, file_name)) {
        Ok(_) => return,
        Err(_) => match error_response {
            ErrorResponse::Exit => {
                command.error(ErrorKind::Io, "could not save image").exit();
            }
            ErrorResponse::Ignore => {
                log(
                    &*format!("could not save image at '{}'", path.display()),
                    LogType::Error,
                );

                return;
            }
        },
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

enum LogType {
    // Info,
    Error,
}

fn log(message: &str, log_type: LogType) {
    let prefix = match log_type {
        // LogType::Info => "INFO",
        LogType::Error => "ERROR",
    };

    println!("{}: {}", prefix, message);
}
