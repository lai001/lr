use rs_core_minimal::misc::is_valid_name;

pub trait UrlExtension {
    fn get_name_in_editor(&self) -> String;
    fn set_name_in_editor(&mut self, new_name: String);
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

    fn set_name_in_editor(&mut self, new_name: String) {
        if !is_valid_name(&new_name) {
            return;
        }
        let path = self.path();
        if path.is_empty() {
            unimplemented!()
        } else {
            let mut path = percent_encoding::percent_decode_str(path)
                .decode_utf8_lossy()
                .to_string();
            if path.starts_with("/") {
                path = path.replacen("/", "", 1);
            }
            let mut split = path
                .split("/")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            if let Some(old_name) = split.last_mut() {
                *old_name = new_name;
            }
            let new_path = split.join("/");
            if let Ok(new_url) = url::Url::parse(&format!(
                "{}://{}/{}",
                self.scheme(),
                self.host_str().unwrap(),
                new_path
            )) {
                *self = new_url;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::UrlExtension;
    #[test]
    fn test_case() {
        let mut url = url::Url::parse("content://Content/Empty").unwrap();
        url.set_name_in_editor("Empty1".to_string());
        assert_eq!(url.to_string(), "content://Content/Empty1");
    }
}
