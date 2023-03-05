use std::{fs::File, io::Write};

use serde::Serialize;

// 处理评论
pub mod comments;
// 处理视频信息
pub mod video_info;

const OID: &str = "211900891";
const BVID: &str = "BV19a41187Ww";

fn main() {
    let source_comments = comments::get_comments(OID);
    let extract_comments = comments::take_comments(source_comments);

    let sort_group_comments = comments::sort_group_comments(extract_comments);
    write_json(&sort_group_comments, "assets/sort_group_comments.json");

    let comments = comments::max_like_comments(sort_group_comments);
    write_json(&comments, "assets/comments.json");

    let video_info = video_info::extract_video_info(video_info::get_video_info(BVID));
    write_json(&video_info, "assets/video_info.json");
}

// 向文件中写入 json 数据
fn write_json<T: ?Sized + Serialize>(value: &T, file_name: &str) {
    let json = serde_json::to_string(value).unwrap();
    let mut json_file = File::create(file_name).unwrap();
    json_file.write_all(json.as_bytes()).unwrap();
}
