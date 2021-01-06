use std::error::Error;

use ffmpeg::frame::Video;
use opencv::{
    core::CV_8UC3,
    imgproc::{
        self, InterpolationFlags, MorphShapes, COLOR_BGR2GRAY, COLOR_GRAY2BGR, THRESH_BINARY,
    },
    prelude::*,
};
use opencv::{
    core::{BorderTypes, Mat, Point, Size, CV_8UC1},
    photo,
};

mod utils;

#[derive(Debug)]
pub struct ConnectedComponent {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub area: i32,
}

#[allow(dead_code)]
pub fn blur(src_frame: &Video, k: i32) -> Result<Video, Box<dyn Error>> {
    let src = utils::frame_to_mat(&src_frame);
    let mut out = Video::new(src_frame.format(), src_frame.width(), src_frame.height());
    let mut dst = utils::frame_to_mat(&mut out);

    imgproc::blur(
        &src,
        &mut dst,
        Size::new(k, k),
        Point::new(-1, -1),
        BorderTypes::BORDER_CONSTANT as i32,
    )?;

    Ok(out)
}

#[allow(dead_code)]
pub fn edges(src_frame: &Video, t1: f64, t2: f64) -> Result<Video, Box<dyn Error>> {
    let src = utils::frame_to_mat(src_frame);
    let mut out = Video::new(src_frame.format(), src_frame.width(), src_frame.height());
    let mut dst = utils::frame_to_mat(&mut out);
    let out_size = Size {
        width: src_frame.width() as i32,
        height: src_frame.height() as i32,
    };
    let mut edges = unsafe { Mat::new_size(out_size, CV_8UC1)? };

    imgproc::canny(&src, &mut edges, t1, t2, 3, false)?;
    imgproc::cvt_color(&edges, &mut dst, imgproc::COLOR_GRAY2BGR, 3)?;

    Ok(out)
}

#[allow(dead_code)]
pub fn denoise(src_frame: &Video) -> Result<Video, Box<dyn Error>> {
    let src = utils::frame_to_mat(src_frame);
    let mut out = Video::new(src_frame.format(), src_frame.width(), src_frame.height());
    let mut dst = utils::frame_to_mat(&mut out);

    photo::fast_nl_means_denoising_colored(&src, &mut dst, 3.0, 3.0, 7, 3)?;

    Ok(out)
}

#[allow(dead_code)]
pub fn pixelate(src_frame: &Video, k: i32) -> Result<Video, Box<dyn Error>> {
    let src = utils::frame_to_mat(src_frame);
    let mut out = Video::new(src_frame.format(), src_frame.width(), src_frame.height());
    let mut dst = utils::frame_to_mat(&mut out);
    let mut tmp = unsafe { Mat::new_size(Size::new(k, k), CV_8UC3)? };
    let tmp_size = Size {
        width: k,
        height: k,
    };

    imgproc::resize(
        &src,
        &mut tmp,
        tmp_size,
        0.0,
        0.0,
        InterpolationFlags::INTER_LINEAR as i32,
    )?;

    let out_size = Size {
        width: src_frame.width() as i32,
        height: src_frame.height() as i32,
    };
    imgproc::resize(
        &tmp,
        &mut dst,
        out_size,
        0.0,
        0.0,
        InterpolationFlags::INTER_NEAREST as i32,
    )?;

    Ok(out)
}

#[allow(dead_code)]
pub fn bgsub(src_frame: &Video) -> Result<Video, Box<dyn Error>> {
    let src = utils::frame_to_mat(src_frame);
    let mut out = Video::new(src_frame.format(), src_frame.width(), src_frame.height());
    let mut dst = utils::frame_to_mat(&mut out);
    dst.set_to(
        &opencv::core::Scalar::all(0.0),
        &opencv::core::no_array().unwrap(),
    )?;

    let out_size = Size {
        width: src_frame.width() as i32,
        height: src_frame.height() as i32,
    };
    let mut subtractor = opencv::video::create_background_subtractor_mog2(500, 16.0, false)?;
    let mut fg_mask = unsafe { Mat::new_size(out_size, CV_8UC3)? };

    BackgroundSubtractorMOG2::apply(&mut subtractor, &src, &mut fg_mask, -1.0)?;

    opencv::core::copy_to(&src, &mut dst, &fg_mask)?;

    Ok(out)
}

#[allow(dead_code)]
pub fn find_objects(
    src_frame: &Video,
    intermediate_frame: Option<&mut Video>,
) -> Result<Vec<ConnectedComponent>, Box<dyn Error>> {
    let src = utils::frame_to_mat(src_frame);
    let mut gray_mat = Mat::default()?;
    imgproc::cvt_color(&src, &mut gray_mat, COLOR_BGR2GRAY, 0)?;

    let mut gray2_mat = Mat::default()?;
    imgproc::blur(
        &gray_mat,
        &mut gray2_mat,
        Size::new(11, 11),
        Point::new(-1, -1),
        BorderTypes::BORDER_CONSTANT as i32,
    )?;

    imgproc::threshold(&gray2_mat, &mut gray_mat, 230.0, 255.0, THRESH_BINARY)?;

    imgproc::erode(
        &gray_mat,
        &mut gray2_mat,
        &imgproc::get_structuring_element(
            MorphShapes::MORPH_RECT as i32,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?,
        Point::new(-1, -1),
        2,
        BorderTypes::BORDER_CONSTANT as i32,
        imgproc::morphology_default_border_value()?,
    )?;

    imgproc::dilate(
        &gray2_mat,
        &mut gray_mat,
        &imgproc::get_structuring_element(
            MorphShapes::MORPH_RECT as i32,
            Size::new(3, 3),
            Point::new(-1, -1),
        )?,
        Point::new(-1, -1),
        4,
        BorderTypes::BORDER_CONSTANT as i32,
        imgproc::morphology_default_border_value()?,
    )?;

    if let Some(mut output_frame) = intermediate_frame {
        if src_frame.format() != output_frame.format()
            || src_frame.width() != output_frame.width()
            || src_frame.height() != output_frame.height()
        {
            return Err(
                "Cannot output intermediate frame. Format or size does not match input".into(),
            );
        }
        let mut intermediate_mat = utils::frame_to_mat(&mut output_frame);
        imgproc::cvt_color(&gray_mat, &mut intermediate_mat, COLOR_GRAY2BGR, 0)?;
    }

    let mut labels = Mat::default()?;
    let mut stats = Mat::default()?;
    let mut centroids = Mat::default()?;
    // TODO: how do we actually use the components?
    let num_labels = imgproc::connected_components_with_stats(
        &gray_mat,
        &mut labels,
        &mut stats,
        &mut centroids,
        4,
        opencv::core::CV_16U,
    )?;

    let mut components = vec![];
    for label in 1..num_labels {
        components.push(ConnectedComponent {
            left: *stats.at_2d::<i32>(label, 0)?,
            top: *stats.at_2d::<i32>(label, 1)?,
            width: *stats.at_2d::<i32>(label, 2)?,
            height: *stats.at_2d::<i32>(label, 3)?,
            area: *stats.at_2d::<i32>(label, 4)?,
        })
    }

    Ok(components)
}
