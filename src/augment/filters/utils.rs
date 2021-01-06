use ffmpeg::frame::Video;
use opencv::core::{Mat, CV_8UC3};

pub fn frame_to_mat(frame: &Video) -> Mat {
    unsafe {
        Mat::new_rows_cols_with_data(
            frame.height() as i32,
            frame.width() as i32,
            CV_8UC3,
            (*frame.as_ptr()).data[0] as *mut std::ffi::c_void,
            frame.stride(0),
        )
        .unwrap()
    }
}
