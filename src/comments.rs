use std::{thread, time::Duration};

use chrono::{Datelike, FixedOffset, Local, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtractComment {
    pub root: u64,
    pub ctime: i64,
    pub htime: String,
    pub like: u64,
    pub uname: String,
    pub avatar: String,
    pub level: u64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupComment {
    pub year: i32,
    pub month: u32,
    pub comments: Vec<ExtractComment>,
}

// 获取所有评论数据
pub fn get_comments(oid: &str) -> Vec<Value> {
    let mut comments = Vec::new();
    let mut count = 0;
    let mut page = 1;
    loop {
        let comment: Value = ureq::get("https://api.bilibili.com/x/v2/reply")
            .query("type", "1")
            .query("oid", oid)
            .query("sort", "0")
            .query("nohot", "0")
            .query("ps", "20")
            .query("pn", page.to_string().as_str())
            .call()
            .unwrap()
            .into_json()
            .unwrap();

        // 增加获取的评论数，和页数
        count += comment
            .pointer("/data/page/size")
            .unwrap()
            .as_u64()
            .unwrap();
        page += 1;

        // 提取评论总数
        let all_count = comment
            .pointer("/data/page/count")
            .unwrap()
            .as_u64()
            .unwrap();

        comments.push(comment); // 将取到的评论存到 comments 中

        // 取到所有评论，跳出循环
        if count >= all_count {
            break;
        }

        // 降低访问频率，防止触发反爬虫机制
        thread::sleep(Duration::from_millis(250));
    }

    comments // 返回获取的所有评论
}

// 提取评论数据中所需部分
pub fn take_comments(comments: Vec<Value>) -> Vec<ExtractComment> {
    // 存放提取出来的评论数据
    let mut extract_comments = Vec::new();

    // 遍历提取数据
    for comment in comments.into_iter() {
        let replies = comment.pointer("/data/replies").unwrap().as_array();

        // replies 在一些情况下可能为 None
        let replies = if let Some(replies) = replies {
            replies
        } else {
            break;
        };

        for reply in replies.iter() {
            let root = reply.pointer("/root").unwrap().as_u64().unwrap();
            let ctime = reply.pointer("/ctime").unwrap().as_i64().unwrap();
            let htime = FixedOffset::east_opt(8 * 60 * 60)
                .unwrap()
                .from_local_datetime(&Local.timestamp_opt(ctime, 0).unwrap().naive_local())
                .unwrap()
                .to_rfc3339();
            let like = reply.pointer("/like").unwrap().as_u64().unwrap();
            let uname = reply
                .pointer("/member/uname")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let avatar = reply
                .pointer("/member/avatar")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let level = reply
                .pointer("/member/level_info/current_level")
                .unwrap()
                .as_u64()
                .unwrap();
            let message = reply
                .pointer("/content/message")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();

            // 将提取出来的数据组成结构体
            let extract_comment = ExtractComment {
                root,
                ctime,
                htime,
                like,
                uname,
                avatar,
                level,
                message,
            };

            // 将数据存到 extract_comments 中
            extract_comments.push(extract_comment);
        }
    }

    extract_comments // 返回提取出来的数据
}

// 对提取出来的评论数据按时间升序排序，并按年月分组
pub fn sort_group_comments(mut extract_comments: Vec<ExtractComment>) -> Vec<GroupComment> {
    // 排序
    extract_comments.sort_by(|a, b| a.ctime.cmp(&b.ctime));

    // 分组
    let mut group_index = 0;
    let current_date = Local.timestamp_opt(extract_comments[0].ctime, 0).unwrap();
    let current_year = current_date.year();
    let mut current_month = current_date.month();
    // 存放分组数据
    let mut groups = vec![GroupComment {
        year: current_year,
        month: current_month,
        comments: Vec::new(),
    }];
    // 进行分组
    for comment in extract_comments.into_iter() {
        let date = Local.timestamp_opt(comment.ctime, 0).unwrap();
        let month = date.month();
        if current_month == month {
            groups[group_index].comments.push(comment);
        } else {
            group_index += 1;
            current_month = month;
            let year = date.year();
            groups.push(GroupComment {
                year,
                month,
                comments: vec![comment],
            });
        }
    }

    groups // 返回分组数据
}

// 取出每个分组中点赞最多的评论
pub fn max_like_comments(groups: Vec<GroupComment>) -> Vec<ExtractComment> {
    groups
        .into_iter()
        .map(|x| {
            x.comments
                .into_iter()
                .max_by(|x, y| x.like.cmp(&y.like))
                .unwrap()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};

    #[test]
    fn test_get_comments() {
        let comments = get_comments("243922477");
        let comments = serde_json::to_string(&comments).unwrap();
        let mut source_comments = File::create("assets/source_comments.json").unwrap();
        source_comments.write_all(comments.as_bytes()).unwrap();
    }

    #[test]
    fn test_take_comments() {
        let comments = get_comments("243922477");
        let extract_comments = take_comments(comments);
        let extract_comments = serde_json::to_string(&extract_comments).unwrap();
        let mut extract_comments_file = File::create("assets/extract_comments.json").unwrap();
        extract_comments_file
            .write_all(extract_comments.as_bytes())
            .unwrap();
    }

    #[test]
    fn test_sort_group_comments() {
        let comments = get_comments("243922477");
        let extract_comments = take_comments(comments);
        let sort_group_comments = sort_group_comments(extract_comments);
        let sort_group_comments = serde_json::to_string(&sort_group_comments).unwrap();
        let mut sort_group_comments_file = File::create("assets/sort_group_comments.json").unwrap();
        sort_group_comments_file
            .write_all(sort_group_comments.as_bytes())
            .unwrap();
    }

    #[test]
    fn test_max_like_comments() {
        let comments = get_comments("243922477");
        let extract_comments = take_comments(comments);
        let sort_group_comments = sort_group_comments(extract_comments);
        let max_like_comments = max_like_comments(sort_group_comments);
        let max_like_comments = serde_json::to_string(&max_like_comments).unwrap();
        let mut max_like_comments_file = File::create("assets/max_like_comments.json").unwrap();
        max_like_comments_file
            .write_all(max_like_comments.as_bytes())
            .unwrap();
    }
}
