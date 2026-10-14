pub enum PasteboardItem {
    Files(Vec<Vec<u8>>),
    Image(Vec<u8>),
    Text(String),
}
