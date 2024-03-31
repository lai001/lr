pub trait UrlExtension {
    fn get_name_in_editor(&self) -> String;
}

impl UrlExtension for url::Url {
    fn get_name_in_editor(&self) -> String {
        let path = self.path();
        if path.is_empty() {
            let host_str = self.host_str().unwrap();
            let host_str = percent_encoding::percent_decode_str(host_str)
                .decode_utf8_lossy()
                .to_string();
            host_str
        } else {
            let path = percent_encoding::percent_decode_str(path)
                .decode_utf8_lossy()
                .to_string();
            path.split("/").last().unwrap().to_string()
        }
    }
}
