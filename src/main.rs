use std::{collections::HashMap, io::Read};

pub type CommonError = Box<dyn std::error::Error>;
pub type CommonResult<T> = std::result::Result<T, CommonError>;
use lazy_static::lazy_static;

mod model;
mod service;
mod util;

lazy_static! {
    static ref FULL_MATCH_MAPPING: HashMap<String, String> =
        util::load_and_convert_tag_mapping().unwrap().0;
    static ref PARTIAL_MATCH_MAPPING: HashMap<String, String> =
        util::load_and_convert_tag_mapping().unwrap().1;
}

fn main() -> CommonResult<()> {
    let file_path = "./datas/1-003.dcm";
    // let file_path = "./datas/93117444";

    let mut offset = 0;

    // 获取文件句柄
    let mut file = crate::util::get_file(&file_path)?;

    let file_length = (&file).metadata()?.len();

    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    // 读取导言
    let result = crate::service::get_preamble(&file_content[offset..].to_vec())?;

    let preamble = result.0;
    let consumed_bytes = result.1;
    offset += consumed_bytes;

    println!("preamble: {}", preamble);

    // 读取前缀
    let result = crate::service::get_prefix(&file_content[offset..].to_vec())?;

    let prefix = result.0;
    let consumed_bytes = result.1;
    offset += consumed_bytes;

    println!("prefix: {}", prefix);

    // 将所有的数据按照element全部都分割好了
    let mut data_elements = Vec::new();

    loop {
        let result = crate::service::get_data_element(&file_content[offset..].to_vec())?;
        let consumed_bytes = result.1;
        offset += consumed_bytes;

        data_elements.push(result.0);

        // println!("offset: {}", offset);

        if offset as u64 >= file_length - 1 {
            break;
        }
    }

    // 检查是否为小端方式进行存储的
    let transfer_syntax_result = util::get_data_element_via_tag(&data_elements, "0002,0010".to_string()).unwrap().data;

    if let Some(model::DicomValue::String(v)) = Some(transfer_syntax_result) {
        if &v != "1.2.840.10008.1.2.1" {
            panic!("目前只支持显式小端的格式");
        }
    }

    println!("{:#?}", data_elements[..4].to_vec());

    // 生成图像数据
    let _ = service::generate_image(&data_elements);

    // 尝试读取一个数据元素
    // let result = crate::service::get_data_element(&file_content[offset..].to_vec())?;
    // let consumed_bytes = result.1;
    // offset += consumed_bytes;

    Ok(())
}
