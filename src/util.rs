use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, BufRead},
};

use regex::Regex;

use crate::CommonResult;

pub fn get_file(file_path: &str) -> CommonResult<File> {
    let f = OpenOptions::new().read(true).open(file_path)?;

    Ok(f)
}

// 因为从buffer中读取到的就是一个u8
// 需要把u8补上前导0，然后拼接在一起
pub fn process_vec_to_tag(buffer: &Vec<u8>) -> String {
    let result = buffer
        .iter()
        .map(|ele| format!("{:02X}", ele))
        .collect::<Vec<String>>();

    result.join("")
}

pub fn process_vec_to_vr(buffer: &Vec<u8>) -> String {
    let result = buffer
        .iter()
        .map(|ele| (ele.to_owned() as char).to_string())
        .collect::<Vec<String>>();

    result.join("")
}

fn vr_mapping() -> Vec<(String, String)> {
    let mapping = vec![
        ("AE", "Application Entity"),
        ("AS", "Age String"),
        ("AT", "Attribute Tag"),
        ("CS", "Code String"),
        ("DA", "Date"),
        ("DS", "Decimal String"),
        ("DT", "Date Time"),
        ("FL", "Floating Point Single"),
        ("FD", "Floating Point Double"),
        ("IS", "Integer String"),
        ("LO", "Long String"),
        ("LT", "Long Text"),
        ("OB", "Other Byte String"),
        ("OD", "Other Double String"),
        ("OF", "Other Float String"),
        ("OW", "Other Word String"),
        ("PN", "Person Name"),
        ("SH", "Short String"),
        ("SL", "Signed Long"),
        ("SQ", "Sequence of Items"),
        ("SS", "Signed Short"),
        ("ST", "Short Text"),
        ("TM", "Time"),
        ("UI", "Unique Identifier (UID)"),
        ("UL", "Unsigned Long"),
        ("UN", "Unknown"),
        ("US", "Unsigned Short"),
        ("UT", "Unlimited Text"),
    ];

    let result = mapping
        .iter()
        .map(|v| (v.0.to_string(), v.1.to_string()))
        .collect::<Vec<(String, String)>>();

    result
}

pub fn get_vr_values() -> Vec<String> {
    let vr_mapping = vr_mapping();

    let result = vr_mapping
        .iter()
        .map(|v| v.0.to_owned())
        .collect::<Vec<String>>();

    result
}

pub fn swap_every_two_bytes_and_echo_string(bytes: &Vec<u8>) -> String {
    let mut result = Vec::new();

    let mut counter = 0;

    loop {
        let start = counter;
        let mut end = counter + 1;

        if start >= bytes.len() {
            break;
        }

        if end >= bytes.len() {
            end = bytes.len() - 1;
        }

        let mut data_segment = bytes[start..end + 1].to_vec();

        data_segment.reverse();

        for ele in &data_segment {
            result.push(format!("{:02X}", ele));
        }

        counter += 2;
    }

    result.join("")
}

pub fn swap_every_two_bytes(bytes: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();

    let mut counter = 0;

    loop {
        let start = counter;
        let mut end = counter + 1;

        if start >= bytes.len() {
            break;
        }

        if end >= bytes.len() {
            end = bytes.len() - 1;
        }

        let mut data_segment = bytes[start..end + 1].to_vec();

        data_segment.reverse();

        for ele in &data_segment {
            result.push(ele.to_owned());
        }

        counter += 2;
    }

    result
}

pub fn get_data_element_via_tag(
    data_elements: &Vec<crate::model::DataElement>,
    tag: String,
) -> Option<crate::model::DataElement> {
    for data_element in data_elements {
        if data_element.tag == tag {
            return Some(data_element.to_owned());
        }
    }

    None
}

pub fn get_tag_human_name(tag: &String) -> CommonResult<String> {
    let mut result = "unknown".to_string();

    let _result = crate::FULL_MATCH_MAPPING.get(tag);

    if _result.is_some() {
        if let Some(v) = _result {
            result = v.to_string();
        }
    } else {
        for (standard_tag, standard_explanation) in &*crate::PARTIAL_MATCH_MAPPING {
            let regex = Regex::new(standard_tag)?;

            if regex.is_match(tag) {
                result = standard_explanation.to_string();
                break;
            }
        }
    }

    Ok(result)
}

pub fn load_and_convert_tag_mapping(
) -> CommonResult<(HashMap<String, String>, HashMap<String, String>)> {
    let file = get_file("./tag_mapping.txt")?;

    let mut full_match_mapping = HashMap::new();
    let mut partial_match_mapping = HashMap::new();

    // 创建一个缓冲读取器
    let reader = io::BufReader::new(file);

    // 按行读取文件
    for line in reader.lines() {
        match line {
            Ok(text) => {
                let content_vec = text
                    .split("\t")
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>();

                let standard_tag = content_vec[0].clone();
                let standard_explanation = content_vec[1].clone();

                if standard_tag.contains("x") {
                    let standard_tag = standard_tag.replace("x", "\\w");
                    partial_match_mapping.insert(standard_tag, standard_explanation);
                } else {
                    full_match_mapping.insert(standard_tag, standard_explanation);
                }
            }
            Err(error) => eprintln!("读取行失败: {}", error),
        }
    }

    Ok((full_match_mapping, partial_match_mapping))
}

pub fn show_buffer_by_hex(buffer: &Vec<u8>) {
    let result = buffer
        .iter()
        .map(|v| format!("{:02X}", v))
        .collect::<Vec<String>>();

    println!("{:?}", result);
}
