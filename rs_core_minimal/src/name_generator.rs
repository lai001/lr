use std::collections::HashMap;

pub struct NameGenerator {
    current_number: Option<isize>,
    names: HashMap<String, i32>,
    re: regex::Regex,
}

impl NameGenerator {
    pub fn new(names: Vec<String>) -> NameGenerator {
        let mut name_map = HashMap::new();
        for name in names {
            name_map.insert(name, 0);
        }
        NameGenerator {
            current_number: None,
            names: name_map,
            re: regex::Regex::new(r"[0-9]\d*$").unwrap(),
        }
    }

    pub fn next(&mut self, name: &str) -> String {
        let mut new_name = name.to_string();
        loop {
            if !self.names.contains_key(&new_name) {
                self.names.insert(new_name.clone(), 0);
                break;
            }
            match self.re.find(&new_name) {
                Some(mt) => {
                    let s = mt.as_str();
                    let number = s.parse::<isize>().unwrap();
                    match &mut self.current_number {
                        Some(current_number) => {
                            *current_number = number + 1;
                        }
                        None => {
                            self.current_number = Some(number);
                        }
                    }
                    new_name = self
                        .re
                        .replace(&new_name, self.current_number.unwrap().to_string())
                        .to_string();
                }
                None => {
                    self.current_number = Some(0);
                    new_name = format!("{}_0", new_name);
                }
            }
        }
        new_name
    }
}

pub fn make_unique_name(names: Vec<String>, new_name: impl AsRef<str>) -> String {
    let mut generator = NameGenerator::new(names);
    generator.next(new_name.as_ref())
}

#[cfg(test)]
mod test {
    use crate::name_generator::make_unique_name;

    #[test]
    fn test_case1() {
        assert_eq!(
            make_unique_name(vec!["abc_1".to_string()], "abc_1"),
            "abc_2"
        );
        assert_eq!(make_unique_name(vec!["abc".to_string()], "abc"), "abc_0");
        assert_eq!(make_unique_name(vec!["abc0".to_string()], "abc0"), "abc1");
        assert_eq!(
            make_unique_name(vec!["abc".to_string(), "abc_0".to_string()], "abc"),
            "abc_1"
        );
    }
}
