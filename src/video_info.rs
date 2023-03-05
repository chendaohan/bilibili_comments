use chrono::{FixedOffset, Local, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtractOwner {
    pub face: String,
    pub mid: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtractVieoInfo {
    pub bvid: String,
    pub pic: String,
    pub title: String,
    pub ctime: i64,
    pub htime: String,
    pub owner: ExtractOwner,
    pub view: u64,
    pub coin: u64,
    pub like: u64,
    pub share: u64,
}

// 获取视频信息
pub fn get_video_info(bvid: &str) -> Value {
    ureq::get("https://api.bilibili.com/x/web-interface/view")
        .query("bvid", bvid)
        .call()
        .unwrap()
        .into_json()
        .unwrap()
}

// 提取视频信息
pub fn extract_video_info(video_info: Value) -> ExtractVieoInfo {
    let bvid = video_info
        .pointer("/data/bvid")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let pic = video_info
        .pointer("/data/pic")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let title = video_info
        .pointer("/data/title")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let ctime = video_info.pointer("/data/ctime").unwrap().as_i64().unwrap();
    let htime = FixedOffset::east_opt(8 * 60 * 60)
        .unwrap()
        .from_local_datetime(&Local.timestamp_opt(ctime, 0).unwrap().naive_local())
        .unwrap()
        .to_rfc3339();
    let face = video_info
        .pointer("/data/owner/face")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let mid = video_info
        .pointer("/data/owner/mid")
        .unwrap()
        .as_u64()
        .unwrap();
    let name = video_info
        .pointer("/data/owner/name")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let owner = ExtractOwner { face, mid, name };
    let view = video_info
        .pointer("/data/stat/view")
        .unwrap()
        .as_u64()
        .unwrap();
    let coin = video_info
        .pointer("/data/stat/coin")
        .unwrap()
        .as_u64()
        .unwrap();
    let like = video_info
        .pointer("/data/stat/like")
        .unwrap()
        .as_u64()
        .unwrap();
    let share = video_info
        .pointer("/data/stat/share")
        .unwrap()
        .as_u64()
        .unwrap();

    ExtractVieoInfo {
        bvid,
        pic,
        title,
        ctime,
        htime,
        owner,
        view,
        coin,
        like,
        share,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};

    #[test]
    fn test_get_video_info() {
        let video_info = get_video_info("BV1uv411q7Mv");
        let video_info = serde_json::to_string(&video_info).unwrap();
        let mut video_info_file = File::create("assets/video_info.json").unwrap();
        video_info_file.write_all(video_info.as_bytes()).unwrap();
    }

    #[test]
    fn test_extract_video_info() {
        let video_info = get_video_info("BV1uv411q7Mv");
        let extract_video_info = extract_video_info(video_info);
        let extract_video_info = serde_json::to_string(&extract_video_info).unwrap();
        let mut extract_video_info_file = File::create("assets/extract_video_info.json").unwrap();
        extract_video_info_file
            .write_all(extract_video_info.as_bytes())
            .unwrap();
    }
}
