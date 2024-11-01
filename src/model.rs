#[derive(Debug, Clone)]
pub enum DicomValue {
    String(String),
    U16Pair((String, String)),
    Float(Vec<f32>),
    Double(Vec<f64>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    I16(Vec<i16>),
    U32(Vec<u32>),
    U16(Vec<u16>),
    Bytes(Vec<u8>),
    Sequence(Vec<DataElement>),
}

#[derive(Debug, Clone)]
pub struct DataElement {
    pub tag_group: String,
    pub tag_element: String,
    pub tag: String,
    pub tag_for_human: String,
    pub vr: String,
    pub data: DicomValue,
}
