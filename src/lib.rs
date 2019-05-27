const BYTES_RANGE_TYPE: &'static str = "bytes";
const BINARY_CONTENT_TYPE: &'static str = "bytes";
const SMALL_FILE_URL: &'static str = "https://recurse-uploads-production.s3.amazonaws.com/b9349b0c-359a-473a-9441-c1bc54a96ca6/austin_guest_resume.pdf";

#[derive(Debug, PartialEq)]
pub struct Info {
    accept_ranges: &'static str,
    content_type: &'static str,
    content_length: i64,
    etag: Option<String>,
}

pub fn get_info(url: &'static str) -> Info {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getting_file_info() {
        let expected_info = super::Info {
            accept_ranges: BYTES_RANGE_TYPE,
            content_type: BINARY_CONTENT_TYPE,
            content_length: 53143,
            etag: Some("ac89ac31a669c13ec4ce037f1203022c".to_string()),
        };
        let actual_info = get_info(SMALL_FILE_URL);

        assert_eq!(expected_info, actual_info);
    }
}
