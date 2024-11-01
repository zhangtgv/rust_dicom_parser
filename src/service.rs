use image::{ImageBuffer, Luma};

use crate::CommonResult;

pub fn get_preamble(buffer: &Vec<u8>) -> CommonResult<(String, usize)> {
    let length = 128;

    let preamble = String::from_utf8(buffer[..length].try_into()?)?;

    Ok((preamble, length))
}

pub fn get_prefix(buffer: &Vec<u8>) -> CommonResult<(String, usize)> {
    let length = 4;

    let prefix = String::from_utf8(buffer[..length].try_into()?)?;

    Ok((prefix, length))
}

// 这里默认都用小端存储
pub fn get_data_element(buffer: &Vec<u8>) -> CommonResult<(crate::model::DataElement, usize)> {
    let mut length = 0;

    // 获取tag的group部分
    let mut group_tag_buffer = buffer[length..length + 2].to_vec();
    group_tag_buffer.reverse();

    length += 2;

    let tag_group = crate::util::process_vec_to_tag(&group_tag_buffer);

    // println!("tag_group: {}", tag_group);

    // 获取tag的element部分
    let mut element_tag_buffer = buffer[length..length + 2].to_vec();
    element_tag_buffer.reverse();

    length += 2;

    let tag_element = crate::util::process_vec_to_tag(&element_tag_buffer);

    // println!("tag_element: {}", tag_element);

    let tag = format!("{},{}", tag_group, tag_element);

    // println!("tag: {}", tag);

    let tag_for_human = crate::util::get_tag_human_name(&tag)?;

    // 获取vr部分
    let vr_buffer = buffer[length..length + 2].to_vec();

    length += 2;

    let vr = crate::util::process_vec_to_vr(&vr_buffer);

    // println!("vr: {}", vr);

    // 获取所有vr的可能值
    let all_vr_values = crate::util::get_vr_values();

    let data_element = if all_vr_values.contains(&vr) {
        // 计算出data element的长度
        let data_element_length;
        if ["OB", "OW", "OF", "UT", "UN"].contains(&vr.as_str()) {
            // 显式vr特殊结构（带预留）
            // 跳过保留的字节
            length += 2;

            // 这几种类型的长度是4字节的，这4字节不包含保留的那2字节
            // https://zhuanlan.zhihu.com/p/671921616
            data_element_length =
                u32::from_le_bytes(buffer[length..length + 4].try_into()?) as usize;

            length += 4;
        } else if vr.as_str() == "SQ" {
            // 显式vr特殊结构（带预留）
            // 跳过保留的字节
            length += 2;

            // 这里SQ的data_element_length的赋值其实没啥用
            // 只是为了维持结构一致
            data_element_length = 0;

            // let mut buffer = Vec::new();
            // file.seek(std::io::SeekFrom::Start(offset + length))?;
            // file.read_to_end(&mut buffer)?;

            // parse_sq_data(&buffer);
            // let result = parse_sq_data(&buffer[length..].to_vec())?;
        } else {
            // 显式vr普通结构（无预留）
            // 显示vr普通结构的长度是2字节的
            data_element_length =
                u16::from_le_bytes(buffer[length..length + 2].try_into()?) as usize;

            length += 2;
        }

        // println!("data element length is: {}", data_element_length);

        // 解析实际数据
        let data_value;

        // 如果是SQ则使用特殊的方式进行解析
        if vr.as_str() == "SQ" {
            let result = parse_sq_data(&buffer[length..].to_vec())?;

            data_value = result.0;
            length += result.1;
        } else {
            data_value = parse_data(&buffer[length..].to_vec(), &vr, data_element_length)?;

            length += data_element_length;
        }

        // println!("data value is: {:#?}", data_value);

        crate::model::DataElement {
            tag_group: tag_group.clone(),
            tag_element: tag_element.clone(),
            tag: tag,
            tag_for_human: tag_for_human,
            vr: vr,
            data: data_value,
        }
    } else {
        println!("这段代码未经过测试，bravo");
        let data_element_length =
            u32::from_le_bytes(buffer[length..length + 4].try_into()?) as usize;

        let data_value = buffer[length..length + data_element_length].to_vec();

        crate::model::DataElement {
            tag_group: tag_group.clone(),
            tag_element: tag_element.clone(),
            tag: tag,
            tag_for_human: tag_for_human,
            vr: "implicit".to_string(),
            data: crate::model::DicomValue::Bytes(data_value),
        }
    };

    Ok((data_element, length))
}

fn parse_data(
    buffer: &Vec<u8>,
    vr: &String,
    data_length: usize,
) -> CommonResult<crate::model::DicomValue> {
    let vr_match = vr.as_str();

    // 这下面的处理逻辑中不会包含SQ
    // 因为SQ的处理方式比较特殊，所以使用专门的parse_sq_data进行处理
    let result = match vr_match {
        "UL" => {
            let mut offset = 0;
            let mut datas = Vec::new();

            loop {
                let data = u32::from_le_bytes(buffer[offset..offset + 4].try_into()?);

                datas.push(data);

                offset += 4;

                if offset >= data_length {
                    break;
                }
            }

            crate::model::DicomValue::U32(datas)
        }
        "US" => {
            let mut offset = 0;
            let mut datas = Vec::new();

            loop {
                let data = u16::from_le_bytes(buffer[offset..offset + 2].try_into()?);

                datas.push(data);

                offset += 2;

                if offset >= data_length {
                    break;
                }
            }

            crate::model::DicomValue::U16(datas)
        }
        "FD" => {
            let mut offset = 0;
            let mut datas = Vec::new();

            loop {
                let data = f64::from_le_bytes(buffer[offset..offset + 8].try_into()?);

                datas.push(data);

                offset += 8;

                if offset >= data_length {
                    break;
                }
            }

            crate::model::DicomValue::Double(datas)
        }
        "FL" => {
            let mut offset = 0;
            let mut datas = Vec::new();

            loop {
                let data = f32::from_le_bytes(buffer[offset..offset + 4].try_into()?);

                datas.push(data);

                offset += 4;

                if offset >= data_length {
                    break;
                }
            }

            crate::model::DicomValue::Float(datas)
        }
        "DS" => {
            let (result, _, _) = encoding_rs::GBK.decode(&buffer[..data_length]);

            let string = result.trim();

            let string_vec = string.split("\\").collect::<Vec<_>>();

            let datas = string_vec
                .iter()
                .map(|v| v.parse::<f64>().unwrap())
                .collect();

            crate::model::DicomValue::Double(datas)
        }
        "OW" => {
            // 对于ow的数据处理，尤其是像素的数据处理比较复杂
            // 这个交给处理图像的部分进行处理
            // 这里只是把数据拿出来
            let pixels = buffer[..data_length].to_vec();
            crate::model::DicomValue::Bytes(pixels)
        }
        "OB" => {
            let (data, _, _) = encoding_rs::GBK.decode(&buffer[..data_length]);

            crate::model::DicomValue::String(data.trim().trim_end_matches("\0").to_string())
        }
        "SL" => {
            let mut offset = 0;
            let mut datas = Vec::new();

            loop {
                let data = i32::from_le_bytes(buffer[offset..offset + 4].try_into()?);

                datas.push(data);

                offset += 4;

                if offset >= data_length {
                    break;
                }
            }

            crate::model::DicomValue::I32(datas)
        }
        "SS" => {
            let mut offset = 0;
            let mut datas = Vec::new();

            loop {
                let data = i16::from_le_bytes(buffer[offset..offset + 2].try_into()?);

                datas.push(data);

                offset += 2;

                if offset >= data_length {
                    break;
                }
            }

            crate::model::DicomValue::I16(datas)
        }
        "UI" | "SH" | "CS" | "DA" | "TM" | "LO" | "PN" | "UN" | "IS" | "DT" | "ST" | "AS"
        | "AE" | "LT" => {
            // todo 解析字符集
            // specific character set
            // 0008,0005
            // 这里因为默认的字符集是ISO_IR 100
            // 他不是utf8，需要使用gbk进行解码
            let (result, _, _) = encoding_rs::GBK.decode(&buffer[..data_length]);

            crate::model::DicomValue::String(result.trim().trim_end_matches("\0").to_string())
        }
        "OD" | "OF" | "UT" => {
            panic!("not support");
        }
        _ => {
            panic!("not support");
        }
    };

    Ok(result)
}

pub fn generate_image(data_elements: &Vec<crate::model::DataElement>) -> CommonResult<()> {
    // 获取rows数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,0010".to_string());

    if result.is_none() {
        panic!("no rows data");
    }

    let dicom_value = result.unwrap().data;
    let rows;

    if let Some(crate::model::DicomValue::U16(v)) = Some(dicom_value) {
        rows = v[0];
    } else {
        rows = 0;
    }

    // 获取columns数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,0011".to_string());

    if result.is_none() {
        panic!("no columns data");
    }

    let dicom_value = result.unwrap().data;
    let columns;

    if let Some(crate::model::DicomValue::U16(v)) = Some(dicom_value) {
        columns = v[0];
    } else {
        columns = 0;
    }

    // 获取photometric interpretation数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,0004".to_string());

    if result.is_none() {
        panic!("no photometric interpretation data");
    }

    let dicom_value = result.unwrap().data;
    let photometric_interpretation;

    if let Some(crate::model::DicomValue::String(v)) = Some(dicom_value) {
        photometric_interpretation = v;
    } else {
        photometric_interpretation = "".to_string();
    }

    // 获取bit allocated数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,0100".to_string());

    if result.is_none() {
        panic!("no bit allocated data");
    }

    let dicom_value = result.unwrap().data;
    let bit_allocated;

    if let Some(crate::model::DicomValue::U16(v)) = Some(dicom_value) {
        bit_allocated = v[0];
    } else {
        bit_allocated = 0;
    }

    // 获取bit stored数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,0101".to_string());

    if result.is_none() {
        panic!("no bit stored data");
    }

    let dicom_value = result.unwrap().data;
    let bit_stored;

    if let Some(crate::model::DicomValue::U16(v)) = Some(dicom_value) {
        bit_stored = v[0];
    } else {
        bit_stored = 0;
    }

    // 获取window center数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,1050".to_string());

    if result.is_none() {
        panic!("no window center data");
    }

    let dicom_value = result.unwrap().data;
    let window_center;

    if let Some(crate::model::DicomValue::Double(v)) = Some(dicom_value) {
        window_center = v[0];
    } else {
        window_center = 0.0;
    }

    // 获取window width数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,1051".to_string());

    if result.is_none() {
        panic!("no window width data");
    }

    let dicom_value = result.unwrap().data;
    let window_width;

    if let Some(crate::model::DicomValue::Double(v)) = Some(dicom_value) {
        window_width = v[0];
    } else {
        window_width = 0.0;
    }

    // 获取rescale intercept数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,1052".to_string());

    if result.is_none() {
        panic!("no rescale intercept data");
    }

    let dicom_value = result.unwrap().data;
    let rescale_intercept;

    if let Some(crate::model::DicomValue::Double(v)) = Some(dicom_value) {
        rescale_intercept = v[0];
    } else {
        rescale_intercept = 0.0;
    }

    // 获取rescale slope数据
    let result = crate::util::get_data_element_via_tag(data_elements, "0028,1053".to_string());

    if result.is_none() {
        panic!("no rescale slope data");
    }

    let dicom_value = result.unwrap().data;
    let rescale_slope;

    if let Some(crate::model::DicomValue::Double(v)) = Some(dicom_value) {
        rescale_slope = v[0];
    } else {
        rescale_slope = 0.0;
    }

    // 获取图像数据
    let result = crate::util::get_data_element_via_tag(data_elements, "7FE0,0010".to_string());

    if result.is_none() {
        panic!("no pixel data");
    }

    let dicom_value = result.unwrap().data;
    let pixel_datas;

    if let Some(crate::model::DicomValue::Bytes(v)) = Some(dicom_value) {
        pixel_datas = v;
    } else {
        pixel_datas = Vec::new();
    }

    // 处理像素数据
    let mut pixels = Vec::new();
    let mut offset = 0;
    let mask = 0xff_u8 << (8 - (bit_allocated - bit_stored));
    let bit_allocated_by_bytes = (bit_allocated / 8) as usize;

    loop {
        if offset >= pixel_datas.len() {
            break;
        }

        let mut pixel_data_buffer = pixel_datas[offset..offset + bit_allocated_by_bytes].to_vec();

        pixel_data_buffer[0] = pixel_data_buffer[0] & mask;

        let pixel = u16::from_le_bytes(pixel_data_buffer[..].try_into()?);

        pixels.push(pixel);

        offset += bit_allocated_by_bytes;
    }

    let processed_pixels = process_image_pixels(
        &pixels,
        &photometric_interpretation,
        rescale_intercept,
        rescale_slope,
        window_width,
        window_center,
    )?;

    write_image_pixels_to_file(columns as u32, rows as u32, &processed_pixels)?;

    Ok(())
}

// 参考了https://github.com/ykuo2/dicom2jpg/blob/main/dicom2jpg/utils.py#L116
fn process_image_pixels(
    pixels: &Vec<u16>,
    photometric_interpretation: &String,
    rescale_intercept: f64,
    rescale_slope: f64,
    window_width: f64,
    window_center: f64,
) -> CommonResult<Vec<u8>> {
    if photometric_interpretation != "MONOCHROME2" && photometric_interpretation != "MONOCHROME1" {
        panic!("photometric interpretation is not supported");
    }

    let data_min = pixels.iter().min().unwrap().to_owned() as f64;
    let data_max = pixels.iter().max().unwrap().to_owned() as f64;
    let data_range = data_max - data_min;

    let pixels = pixels
        .iter()
        .map(|v| v.to_owned() as f64)
        .map(|v| v * rescale_slope + rescale_intercept)
        .map(|v| {
            if v <= (window_center - window_width / 2.0) {
                data_min
            } else if v > (window_center + window_width / 2.0) {
                data_max
            } else {
                (v - window_center + window_width / 2.0) / window_width * data_range + data_min
            }
        })
        .collect::<Vec<f64>>();

    let piexel_min = pixels
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let piexel_max = pixels
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let mut pixels = pixels
        .iter()
        .map(|v| ((v - piexel_min) / (piexel_max - piexel_min)) * 255.0)
        .map(|v| v.round() as u8)
        .collect::<Vec<u8>>();

    // 对MONOCHROME1的显示进行额外处理
    if photometric_interpretation == "MONOCHROME1" {
        let pixels_max_value = pixels.iter().max().unwrap();

        pixels = pixels
            .iter()
            .map(|v| (pixels_max_value - v).to_owned())
            .collect::<Vec<u8>>();
    }

    Ok(pixels)
}

fn write_image_pixels_to_file(width: u32, height: u32, datas: &Vec<u8>) -> CommonResult<()> {
    // 创建一个256x256的RGB图像
    let mut img = ImageBuffer::<Luma<u8>, _>::new(width, height);

    // 将数据映射到图像的像素值
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let index = (y * width + x) as usize;
        let value = datas[index];
        *pixel = Luma([value]); // 将数据作为RGB值
    }

    // 保存图像到文件
    img.save("./images/output.png")?;

    Ok(())
}

// 具体的实现一句参考下方链接里的三个表格
// https://dicom.nema.org/dicom/2013/output/chtml/part05/sect_7.5.html
fn parse_sq_data(buffer: &Vec<u8>) -> CommonResult<(crate::model::DicomValue, usize)> {
    let mut offset = 0;

    let data_element_length = u32::from_le_bytes(buffer[offset..offset + 4].try_into()?) as usize;

    // 这里是跳过4字节的data element length
    offset += 4;

    // 用于记录SQ实际存储内容的buffer
    // 已经去除前后的表示帧界的特定序列
    // 只包含多个item的数据
    let data_value_buffer;

    if data_element_length == 0xffffffff {
        let sequence = crate::util::swap_every_two_bytes(&vec![
            0xFF, 0xFE, 0xE0, 0xDD, 0x00, 0x00, 0x00, 0x00,
        ]);

        if let Some(index) = buffer[offset..]
            .windows(sequence.len())
            .position(|window| window == sequence)
        {
            // 这里计算index因为是基于offset做过偏移了
            // 所以在获取数据的时候是需要在终止点上加上offset的
            data_value_buffer = buffer[offset..offset + index].to_vec();

            // 因为windows这个函数的匹配不会把sequence本身的长度算进去
            // 所以这里SQ真正的中止点需要加上sequence的长度
            offset += index + sequence.len();
        } else {
            panic!("Seq. Delim. Tag not found");
        }
    } else {
        // 如果不是8f，那么data_element_length就是实际的长度
        // 这里是长度的4字节
        data_value_buffer = buffer[offset..offset + data_element_length as usize].to_vec();

        offset += data_element_length;
    }

    // println!("data_value_buffer {:?}", &data_value_buffer);
    let sub_elements = crate::model::DicomValue::Sequence(parse_sq_items(&data_value_buffer)?);

    Ok((sub_elements, offset))
}

fn parse_sq_items(buffer: &Vec<u8>) -> CommonResult<Vec<crate::model::DataElement>> {
    let mut offset = 0;

    // 因为sq中存储类似element的数组
    let mut sub_elements = Vec::new();

    loop {
        if offset >= buffer.len() {
            break;
        }

        let item_tag_string =
            crate::util::swap_every_two_bytes_and_echo_string(&buffer[offset..offset + 4].to_vec());

        offset += 4;

        if item_tag_string != "FFFEE000".to_string() {
            panic!("item tag is invalid");
        }

        let mut item_length = u32::from_le_bytes(buffer[offset..offset + 4].try_into()?) as usize;
        let mut item_content_buffer = Vec::new();

        offset += 4;

        if item_length == 0xffffffff {
            let sequence = crate::util::swap_every_two_bytes(&vec![
                0xFF, 0xFE, 0xE0, 0x0D, 0x00, 0x00, 0x00, 0x00,
            ]);

            if let Some(index) = buffer[offset..]
                .windows(sequence.len())
                .position(|window| window == sequence)
            {
                item_length = index as usize;

                // 这里计算index因为是基于offset做过偏移了
                // 所以在获取数据的时候是需要在终止点上加上offset的
                item_content_buffer = buffer[offset..offset + item_length].to_vec();

                // 因为windows这个函数的匹配不会把sequence本身的长度算进去
                // 所以这里SQ真正的中止点需要加上sequence的长度
                offset += index + sequence.len();
            } else {
                panic!("Seq. Delim. Tag not found");
            }
        } else {
            println!("这段代码未经过测试，alpha");
            item_content_buffer = buffer[offset..offset + item_length].to_vec();

            offset += item_length as usize;
        }

        // 好像一个item value data set中是可以包含多个element的
        let mut item_offset = 0;

        loop {
            let result = get_data_element(&item_content_buffer[item_offset..].to_vec())?;

            let data_element = result.0;
            let consumed_bytes = result.1;

            sub_elements.push(data_element);
            item_offset += consumed_bytes;

            if item_offset >= item_content_buffer.len() {
                break;
            }
        }
    }

    Ok(sub_elements)
}
