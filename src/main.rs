mod color;
mod decoration;
mod pair;

use clap::{arg_enum, value_t};
use failure::Fail;
use std::path::Path;
use tokio::prelude::*;

#[derive(Debug, Clone, Fail)]
enum ApplicationError {
    #[fail(display = "specified file does not exists: {:?}", _0)]
    InputFileDoesNotExists(Option<String>),
}

async fn check_file_exists<P>(file_path: P) -> Result<(), ApplicationError>
where
    P: AsRef<Path>,
{
    if file_path.as_ref().exists() {
        Ok(())
    } else {
        Err(ApplicationError::InputFileDoesNotExists(
            file_path.as_ref().to_str().map(Into::into),
        ))
    }
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    enum Gravity {
        UpperCentered,
        LeftCentered,
        LowerCentered,
        RightCentered,
        Centered,
    }
}

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    enum Format {
        Jpeg,
        Png,
    }
}

impl Format {
    fn to_image_output_format(self) -> image::ImageOutputFormat {
        match self {
            Format::Jpeg => image::ImageOutputFormat::JPEG(100),
            Format::Png => image::ImageOutputFormat::PNG,
        }
    }
}

type TextBox = (String, rusttype::Scale, (i32, i32, i32, i32));

fn textboxes<'a, T, U>(
    input_position: Option<T>,
    gravity: Option<Gravity>,
    texts: U,
    canvas_size: (u32, u32),
    font: &rusttype::Font<'_>,
    height: u32,
) -> Result<Vec<TextBox>, failure::Error>
where
    T: Into<(u32, u32)> + Copy,
    U: IntoIterator<Item = &'a str>,
{
    if let Some(position) = input_position {
        let position = position.into();
        let initial_rect = (position.0 as i32, position.1 as i32, 0, 0);

        let (textboxes, _) = texts
            .into_iter()
            .map(decoration::DecoratedString::from)
            .fold((Vec::new(), initial_rect), |(mut vec, previous_rect), l| {
                let scale_factor = l.decoration.scale_factor();
                let scale = rusttype::Scale {
                    x: height as f32 * scale_factor,
                    y: height as f32 * scale_factor,
                };

                let v_metrics = font.v_metrics(scale);
                let textbox_height = v_metrics.ascent.abs() + v_metrics.descent.abs();
                let textbox_width = font
                    .layout(&l.body, scale, rusttype::Point { x: 0.0, y: 0.0 })
                    .map(rusttype::PositionedGlyph::into_unpositioned)
                    .fold(0.0, |accm, g| accm + g.h_metrics().advance_width)
                    + 0.5;
                let difference = if previous_rect.2 != 0 {
                    (((previous_rect.2 as f32 - textbox_width) / 2.0) + 0.5) as i32
                } else {
                    0
                };
                let x = previous_rect.0 + difference;
                let y = previous_rect.1 + previous_rect.3;

                let textbox = (x, y, textbox_width as i32, textbox_height as i32);
                vec.push((l.body, scale, textbox));
                (vec, textbox)
            });
        Ok(textboxes)
    } else {
        let provisional_textboxes =
            textboxes(Some((0, 0)), None, texts, canvas_size, font, height)?;
        let max_width = provisional_textboxes
            .iter()
            .map(|(_, _, r)| r.2)
            .max()
            .unwrap_or(0);
        let sum_of_heights: i32 = provisional_textboxes.iter().map(|(_, _, r)| r.3).sum();
        let gravity = gravity.unwrap_or(Gravity::Centered);

        log::info!("canvas size: {:?}", canvas_size);
        log::info!("max_width: {}", max_width);

        match gravity {
            Gravity::UpperCentered => {
                let x = ((canvas_size.0 as f32 - max_width as f32) / 2.0) as i32;
                let y = ((sum_of_heights as f32 / 16.0) + 0.5) as i32;
                let textboxes = provisional_textboxes
                    .into_iter()
                    .map(|(l, s, r)| (l, s, (r.0 + x, r.1 + y, r.2, r.3)))
                    .collect::<Vec<_>>();
                Ok(textboxes)
            }
            Gravity::LeftCentered => {
                let x = ((sum_of_heights as f32 / 16.0) + 0.5) as i32;
                let y = (((canvas_size.1 as f32 - sum_of_heights as f32) / 2.0) + 0.5) as i32;
                let textboxes = provisional_textboxes
                    .into_iter()
                    .map(|(l, s, r)| (l, s, (r.0 + x, r.1 + y, r.2, r.3)))
                    .collect::<Vec<_>>();
                Ok(textboxes)
            }
            Gravity::LowerCentered => {
                let x = (((canvas_size.0 as f32 - max_width as f32) / 2.0) + 0.5) as i32;
                let y = (canvas_size.1 as i32 - sum_of_heights)
                    - ((sum_of_heights as f32 / 16.0) + 0.5) as i32;
                let textboxes = provisional_textboxes
                    .into_iter()
                    .map(|(l, s, r)| (l, s, (r.0 + x, r.1 + y, r.2, r.3)))
                    .collect::<Vec<_>>();
                Ok(textboxes)
            }
            Gravity::RightCentered => {
                let x = canvas_size.0 as i32 - max_width;
                let y = (((canvas_size.1 as f32 - sum_of_heights as f32) / 2.0) + 0.5) as i32;
                let textboxes = provisional_textboxes
                    .into_iter()
                    .map(|(l, s, r)| (l, s, (r.0 + x, r.1 + y, r.2, r.3)))
                    .collect::<Vec<_>>();
                Ok(textboxes)
            }
            Gravity::Centered => {
                let x = ((canvas_size.0 as f32 - max_width as f32) / 2.0) as i32;
                let y = ((canvas_size.1 as f32 - sum_of_heights as f32) / 2.0) as i32;
                let textboxes = provisional_textboxes
                    .into_iter()
                    .map(|(l, s, r)| (l, s, (r.0 + x, r.1 + y, r.2, r.3)))
                    .collect::<Vec<_>>();
                Ok(textboxes)
            }
        }
    }
}

#[derive(Debug)]
struct DrawingOptions<P, Q>
where
    P: AsRef<Path>,
    Q: Into<(u32, u32)> + Copy,
{
    in_path: P,
    out_path: P,
    text: String,
    color: color::Color,
    shadow_color: Option<color::Color>,
    font_path: P,
    height: u32,
    position: Option<Q>,
    gravity: Option<Gravity>,
    format: Format,
}

async fn draw_text_rgba<P, Q>(options: DrawingOptions<P, Q>) -> Result<(), failure::Error>
where
    P: AsRef<Path>,
    Q: Into<(u32, u32)> + Copy,
{
    let mut font_file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(&options.font_path)
        .await?;
    let mut font = Vec::new();
    font_file.read_to_end(&mut font).await?;
    drop(font_file);
    let font = rusttype::FontCollection::from_bytes(font)?.font_at(0)?;
    let image = image::open(&options.in_path)?.to_rgba();

    let texts = options.text.lines();
    let textboxes = textboxes(
        options.position,
        options.gravity,
        texts,
        (image.width(), image.height()),
        &font,
        options.height,
    )?;

    let draw_layer = textboxes
        .into_iter()
        .fold(image, |mut accm, (l, scale, textbox)| {
            if let Some(shadow_color) = options.shadow_color {
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    shadow_color.into(),
                    textbox.0.max(0) as u32 + 2,
                    textbox.1.max(0) as u32 + 2,
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    shadow_color.into(),
                    (textbox.0.max(0) as u32).saturating_sub(2),
                    textbox.1.max(0) as u32 + 2,
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    shadow_color.into(),
                    (textbox.0.max(0) as u32).saturating_sub(2),
                    (textbox.1.max(0) as u32).saturating_sub(2),
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    shadow_color.into(),
                    textbox.0.max(0) as u32 + 2,
                    (textbox.1.max(0) as u32).saturating_sub(2),
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    options.color.into(),
                    textbox.0.max(0) as u32,
                    textbox.1.max(0) as u32,
                    scale,
                    &font,
                    &l,
                );
                accm
            } else {
                imageproc::drawing::draw_text(
                    &mut accm,
                    options.color.into(),
                    textbox.0.max(0) as u32,
                    textbox.1.max(0) as u32,
                    scale,
                    &font,
                    &l,
                )
            }
        });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(options.out_path)?;
    image::DynamicImage::ImageRgba8(draw_layer)
        .write_to(&mut file, options.format.to_image_output_format())?;

    Ok(())
}

async fn draw_text_luma_alpha<P, Q>(options: DrawingOptions<P, Q>) -> Result<(), failure::Error>
where
    P: AsRef<Path>,
    Q: Into<(u32, u32)> + Copy,
{
    let mut font_file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(&options.font_path)
        .await?;
    let mut font = Vec::new();
    font_file.read_to_end(&mut font).await?;
    drop(font_file);
    let font = rusttype::FontCollection::from_bytes(font)?.font_at(0)?;
    let image = image::open(&options.in_path)?.to_luma_alpha();

    let texts = options.text.lines();
    let textboxes = textboxes(
        options.position,
        options.gravity,
        texts,
        (image.width(), image.height()),
        &font,
        options.height,
    )?;

    let draw_layer = textboxes
        .into_iter()
        .fold(image, |mut accm, (l, scale, textbox)| {
            if options.shadow_color.is_some() {
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    image::LumaA([255, 255]),
                    textbox.0.max(0) as u32 + 2,
                    textbox.1.max(0) as u32 + 2,
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    image::LumaA([255, 255]),
                    (textbox.0.max(0) as u32).saturating_sub(2),
                    textbox.1.max(0) as u32 + 2,
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    image::LumaA([255, 255]),
                    (textbox.0.max(0) as u32).saturating_sub(2),
                    (textbox.1.max(0) as u32).saturating_sub(2),
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    image::LumaA([255, 255]),
                    textbox.0.max(0) as u32 + 2,
                    (textbox.1.max(0) as u32).saturating_sub(2),
                    scale,
                    &font,
                    &l,
                );
                imageproc::drawing::draw_text_mut(
                    &mut accm,
                    image::LumaA([0, 255]),
                    textbox.0.max(0) as u32,
                    textbox.1.max(0) as u32,
                    scale,
                    &font,
                    &l,
                );
                accm
            } else {
                imageproc::drawing::draw_text(
                    &mut accm,
                    image::LumaA([0, 255]),
                    textbox.0.max(0) as u32,
                    textbox.1.max(0) as u32,
                    scale,
                    &font,
                    &l,
                )
            }
        });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(options.out_path)?;
    image::DynamicImage::ImageLumaA8(draw_layer)
        .write_to(&mut file, options.format.to_image_output_format())?;

    Ok(())
}

async fn resize_image_keep_aspect_ratio<P>(
    in_path: P,
    out_path: P,
    width: u32,
    format: Format,
) -> Result<(), failure::Error>
where
    P: AsRef<Path>,
{
    let image = image::open(in_path)?;

    let new_image = image.resize(
        width,
        std::u32::MAX,
        image::imageops::FilterType::CatmullRom,
    );
    drop(image);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(out_path)?;
    new_image.write_to(&mut file, format.to_image_output_format())?;

    Ok(())
}

async fn resize_image<P, Q>(
    in_path: P,
    out_path: P,
    dimensions: Q,
    format: Format,
) -> Result<(), failure::Error>
where
    P: AsRef<Path>,
    Q: Into<(u32, u32)>,
{
    let image = image::open(in_path)?;
    let target_dimensions = dimensions.into();

    let new_image = image.resize_exact(
        target_dimensions.0,
        target_dimensions.1,
        image::imageops::FilterType::CatmullRom,
    );
    drop(image);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(out_path)?;
    new_image.write_to(&mut file, format.to_image_output_format())?;

    Ok(())
}

fn add_options_to_app<'a, 'b>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.version(clap::crate_version!())
        .author(clap::crate_authors!())
        .name(clap::crate_name!())
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(
            clap::Arg::with_name("text")
                .short("t")
                .long("text")
                .takes_value(true)
                .value_name("STRING")
                .help("Sets the text to draw.")
                .requires_all(&["font", "font_height"]),
        )
        .arg(
            clap::Arg::with_name("gravity")
                .short("a")
                .long("gravity")
                .takes_value(true)
                .value_name("GRAVITY")
                .help(
                    "Sets the text position by default values. Conflicts with `--position` option.",
                )
                .conflicts_with("position")
                .possible_values(&[
                    "UpperCentered",
                    "LeftCentered",
                    "LowerCentered",
                    "RightCentered",
                    "Centered",
                ]),
        )
        .arg(
            clap::Arg::with_name("format")
                .short("m")
                .long("file-format")
                .takes_value(true)
                .value_name("Jpeg | Png")
                .help("Sets the output file format. `Png` is default.")
                .possible_values(&["Png", "Jpeg"]),
        )
}

async fn dispatch(
    input: &str,
    output: &str,
    output_format: Format,
    matches: &clap::ArgMatches<'_>,
) -> Result<(), failure::Error> {
    log::info!("input: {}", input);
    log::info!("output: {}", output);

    check_file_exists(input).await?;

    if let Some(pair) = matches
        .value_of("resize")
        .map(pair::Pair::from)
        .or_else(|| matches.value_of("resize_keep").map(pair::Pair::from))
    {
        log::info!("size: {:?}", pair);

        if matches.is_present("resize") {
            resize_image(input, output, pair, output_format).await?;
        } else {
            resize_image_keep_aspect_ratio(input, output, pair.x, output_format).await?;
        }
    } else if let (Some(text), Some(color), Some(font_path), Some(height)) = (
        matches.value_of("text"),
        matches
            .value_of("color")
            .map(color::Color::from)
            .or_else(|| Some(color::Color::black())),
        matches.value_of("font"),
        matches
            .value_of("font_height")
            .map(|s| s.parse::<u32>().unwrap_or(12)),
    ) {
        let shadow_color = matches.value_of("shadow_color").map(color::Color::from);
        let position = matches.value_of("position").map(pair::Pair::from);
        let gravity = value_t!(matches, "gravity", Gravity).ok();

        log::info!("text: {}", text);
        log::info!("color: {:?}", color);
        log::info!("shadow color: {:?}", shadow_color);
        log::info!("font path: {}", font_path);
        log::info!("font height: {}", height);
        log::info!("position: {:?}", position);
        log::info!("gravity: {:?}", gravity);

        let options = DrawingOptions {
            in_path: input.to_owned(),
            out_path: output.to_owned(),
            text: text.to_owned(),
            color,
            shadow_color,
            font_path: font_path.to_owned(),
            height,
            position,
            gravity,
            format: output_format,
        };

        if matches.is_present("grayscale") {
            draw_text_luma_alpha(options).await?;
        } else {
            draw_text_rgba(options).await?;
        }
    }

    Ok(())
}

async fn run() -> Result<(), failure::Error> {
    env_logger::try_init()?;

    let yaml = clap::load_yaml!("../cli.yaml");
    let app = clap::App::from_yaml(&yaml);
    let app = add_options_to_app(app);
    let matches = app.get_matches();

    let output_format = value_t!(matches, "format", Format).unwrap_or(Format::Png);
    log::info!("output format: {:?}", output_format);

    if let (Some(input), Some(output)) = (matches.value_of("input"), matches.value_of("output")) {
        dispatch(input, output, output_format, &matches).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => {}
        Err(error) => log::error!("{:?}", error),
    }
}
