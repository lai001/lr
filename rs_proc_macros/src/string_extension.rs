pub(crate) trait StringExtension {
    fn trim_quote(&self) -> String;
}

impl StringExtension for String {
    fn trim_quote(&self) -> String {
        let file = self.replace("\"", "");
        file.trim().to_string()
    }
}

impl StringExtension for &str {
    fn trim_quote(&self) -> String {
        let file = self.replace("\"", "");
        file.trim().to_string()
    }
}
