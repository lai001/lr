use crate::c_lexer::TokType;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    iter::zip,
};

#[derive(Clone, Debug)]
pub struct Definition {
    name: Option<String>,
    args: Vec<String>,
    content: String,
    stack: Vec<TokType>,
}

impl Definition {
    fn new() -> Self {
        const CAPACITY: usize = 1 << 5;
        Self {
            name: None,
            args: vec![],
            content: String::with_capacity(CAPACITY),
            stack: vec![],
        }
    }

    pub fn input(name: String, string: &str) -> Self {
        let (args, content) = Self::args(string);
        Self {
            name: Some(name),
            args,
            content,
            stack: vec![],
        }
    }

    fn push(&mut self, previous: &[crate::c_lexer::Token], token: &crate::c_lexer::Token) -> bool {
        let _ = previous;
        let mut is_stop = false;
        if self.name.is_none() {
            if token.str == " " {
            } else {
                assert_eq!(token.ty, TokType::Ident);
                self.name = Some(token.str.to_string());
            }
        } else {
            if token.ty == TokType::Newline {
                if let Some(last) = self.stack.last() {
                    if *last == TokType::Newline {
                        is_stop = true;
                    }
                } else {
                    is_stop = true;
                }
            } else if token.ty == TokType::Backslash {
                self.content.push_str("\n");
            } else {
                self.content.push_str(token.str);
            }
        }
        if token.ty == TokType::Newline || token.ty == TokType::Backslash {
            self.stack.push(token.ty.clone());
        }

        if is_stop {
            let (args, new_content) = Self::args(&self.content);
            self.args = args;
            self.content = new_content;
        }
        is_stop
    }

    fn args(content: &str) -> (Vec<String>, String) {
        let content_trim = content.trim();
        let new_content: String;
        let mut args: Vec<String> = vec![];
        let re = regex::Regex::new(r"^\(([^)]*)\)").unwrap();
        if let Some(caps) = re.captures(content_trim.trim()) {
            new_content = content_trim[caps.get_match().end()..content_trim.len()].to_string();
            let args_content = &caps[0][1..caps[0].len() - 1];
            for value in args_content.split(",") {
                args.push(value.trim().to_string());
            }
        } else {
            new_content = content.to_string();
        }
        (args, new_content)
    }
}

struct ResolveDefinition {
    definition: Definition,
    content: String,
    args_content: String,
    stack: Vec<()>,
}

impl ResolveDefinition {
    fn new(definition: Definition) -> Self {
        const CAPACITY: usize = 1 << 5;
        Self {
            definition,
            content: String::with_capacity(CAPACITY),
            args_content: String::with_capacity(CAPACITY),
            stack: vec![],
        }
    }

    fn push(&mut self, previous: &[crate::c_lexer::Token], token: &crate::c_lexer::Token) -> bool {
        let _ = previous;
        let mut is_stop = false;
        if self.definition.args.is_empty() {
            is_stop = true;
            self.content = self.definition.content.clone();
        } else {
            if self.stack.is_empty() {
                if token.ty == TokType::Space {
                } else if token.ty == TokType::LParen {
                    self.stack.push(());
                } else {
                    panic!();
                }
            } else if self.stack.len() == 1 {
                if token.ty == TokType::RParen {
                    is_stop = true;
                    self.args_content = self.args_content.trim().to_string();
                    let _ = validate(&self.args_content).unwrap();
                    let mut args: Vec<String> = vec![];
                    for value in self.args_content.split(",") {
                        args.push(value.trim().to_string());
                    }
                    assert_eq!(args.len(), self.definition.args.len());
                    let mut new_content = self.definition.content.clone();
                    for (to, from) in zip(args.iter(), self.definition.args.iter()) {
                        new_content = new_content.replace(from, to);
                    }
                    let re = regex::Regex::new(r"\s*##\s*").unwrap();
                    new_content = re.replace_all(&new_content, "").to_string();
                    self.content = new_content;
                } else {
                    self.args_content.push_str(token.str);
                }
            } else {
                panic!();
            }
        }
        is_stop
    }
}

#[derive(Clone, Debug)]
struct BranchResult {
    is_stop: bool,
    is_consume: bool,
}

impl BranchResult {
    fn new() -> Self {
        Self {
            is_stop: false,
            is_consume: false,
        }
    }
}

struct BranchExpression {
    ty: TokType,
    expression: String,
    result: Option<bool>,
}

impl BranchExpression {
    fn new(ty: TokType) -> Self {
        const CAPACITY: usize = 1 << 5;
        Self {
            ty,
            expression: String::with_capacity(CAPACITY),
            result: None,
        }
    }
}

struct MacroBranch {
    expression: BranchExpression,
    is_expression_consume: bool,
    branch_result: BranchResult,
    is_ignore: bool,
}

impl MacroBranch {
    fn new(mark: TokType, is_ignore: bool) -> Self {
        Self {
            expression: BranchExpression::new(mark),
            branch_result: BranchResult::new(),
            is_expression_consume: true,
            is_ignore,
        }
    }

    fn push(
        &mut self,
        macros: &HashMap<String, Definition>,
        previous: &[crate::c_lexer::Token],
        token: &crate::c_lexer::Token,
    ) -> &BranchResult {
        let _ = previous;
        let branch_result = &mut self.branch_result;
        branch_result.is_stop = false;

        if self.is_ignore {
            branch_result.is_consume = true;
            if token.ty == TokType::KwEndif {
                branch_result.is_stop = true;
            }
            return branch_result;
        }
        match &token.ty {
            TokType::KwIf => {
                unreachable!();
            }
            TokType::KwIfdef => {
                unreachable!();
            }
            TokType::KwIfndef => {
                unreachable!();
            }
            TokType::KwElif => {
                if let Some(last) = previous.last() {
                    if matches!(last.ty, crate::c_lexer::TokType::Hash) {
                        if let Some(result) = self.expression.result {
                            if result {
                                self.is_expression_consume = false;
                                self.expression.result = None;
                            } else {
                                self.is_expression_consume = true;
                                self.expression.ty = TokType::KwElif;
                                self.expression.expression.clear();
                            }
                        }

                        branch_result.is_consume = true;
                    }
                }
            }
            TokType::KwElse => {
                if let Some(last) = previous.last() {
                    if matches!(last.ty, crate::c_lexer::TokType::Hash) {
                        if let Some(result) = self.expression.result {
                            if result {
                                self.expression.result = None;
                            } else {
                                self.expression.result = Some(true);
                            }
                        }
                        self.is_expression_consume = false;
                        self.expression.ty = TokType::KwElse;
                        self.expression.expression.clear();
                        branch_result.is_consume = true;
                    }
                }
            }
            TokType::KwEndif => {
                if let Some(last) = previous.last() {
                    if matches!(last.ty, crate::c_lexer::TokType::Hash) {
                        branch_result.is_stop = true;
                        branch_result.is_consume = true;
                    }
                }
            }
            TokType::Newline => {
                let current_ty = &self.expression.ty;
                match current_ty {
                    TokType::KwIf => {
                        if self.is_expression_consume {
                            let is_true = Self::resolve_no_args_definition(
                                self.expression.expression.clone(),
                                macros,
                            );
                            self.expression.result = Some(is_true);
                            self.is_expression_consume = false;
                        }
                    }
                    TokType::KwElse => {}
                    TokType::KwElif => {
                        if self.is_expression_consume {
                            let is_true = Self::resolve_no_args_definition(
                                self.expression.expression.clone(),
                                macros,
                            );
                            self.expression.result = Some(is_true);
                            self.is_expression_consume = false;
                        }
                    }
                    TokType::KwIfdef => {
                        if self.is_expression_consume {
                            let is_true = macros.contains_key(self.expression.expression.trim());
                            self.expression.result = Some(is_true);
                            self.is_expression_consume = false;
                        }
                    }
                    TokType::KwIfndef => {
                        if self.is_expression_consume {
                            let is_true = !macros.contains_key(self.expression.expression.trim());
                            self.expression.result = Some(is_true);
                            self.is_expression_consume = false;
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }

                if self.is_expression_consume {
                    branch_result.is_consume = true;
                } else {
                    if let Some(result) = self.expression.result {
                        if result {
                            branch_result.is_consume = false;
                        } else {
                            branch_result.is_consume = true;
                        }
                    } else {
                        branch_result.is_consume = true;
                    }
                }
            }
            _ => {
                if self.is_expression_consume {
                    branch_result.is_consume = true;
                    self.expression.expression.push_str(token.str);
                } else {
                    if let Some(result) = self.expression.result {
                        if result {
                            branch_result.is_consume = false;
                        } else {
                            branch_result.is_consume = true;
                        }
                    } else {
                        branch_result.is_consume = true;
                    }
                }
            }
        }

        branch_result
    }

    fn resolve_no_args_definition(
        mut expression: String,
        macros: &HashMap<String, Definition>,
    ) -> bool {
        let parser = crate::pp_expr::PPParser::new();
        for (name, def) in macros {
            assert!(def.args.is_empty());
            let re = regex::Regex::new(&format!(r"\b{}\b", name)).unwrap();
            expression = re.replace_all(&expression, &def.content).to_string();
        }
        let result = parser.parse(&expression);
        if let Err(err) = result {
            panic!("Error: {err}, expression: \"{expression}\"");
        }
        result.unwrap()
    }
}

struct MacroBranchStack {
    stack: Vec<MacroBranch>,
}

impl MacroBranchStack {
    fn new() -> Self {
        const CAPACITY: usize = 1 << 2;
        Self {
            stack: Vec::with_capacity(CAPACITY),
        }
    }

    fn push(
        &mut self,
        macros: &HashMap<String, Definition>,
        previous: &[crate::c_lexer::Token],
        token: &crate::c_lexer::Token,
    ) -> BranchResult {
        let mut branch_result = BranchResult::new();
        branch_result.is_stop = false;
        match &token.ty {
            TokType::KwIf | TokType::KwIfdef | TokType::KwIfndef => {
                if let Some(last) = previous.last() {
                    if matches!(last.ty, crate::c_lexer::TokType::Hash) {
                        if let Some(last_mut) = self.stack.last_mut() {
                            if let Some(result) = last_mut.expression.result {
                                self.stack.push(MacroBranch::new(token.ty.clone(), !result));
                            } else {
                                self.stack.push(MacroBranch::new(token.ty.clone(), true));
                            }
                        } else {
                            self.stack.push(MacroBranch::new(token.ty.clone(), false));
                        }
                        branch_result.is_consume = true;
                    } else {
                        if let Some(last_mut) = self.stack.last_mut() {
                            branch_result = last_mut.branch_result.clone();
                        }
                    }
                }
            }

            _ => {
                if let Some(last_mut) = self.stack.last_mut() {
                    branch_result = last_mut.push(macros, previous, token).clone();
                    if branch_result.is_stop {
                        self.stack.remove(self.stack.len() - 1);
                    }
                }
            }
        }
        branch_result
    }
}

fn validate(input: &str) -> Result<(), String> {
    let allowed = regex::Regex::new(r"^[A-Za-z,_\s]+$").unwrap();
    if !allowed.is_match(input) {
        return Err(format!("\"{input}\" Contains illegal characters"));
    }
    let re = regex::Regex::new(r"^\s*[A-Za-z_]+(?:,\s*[A-Za-z_]+)*$").unwrap();
    if !re.is_match(input) {
        if input.contains(' ') && !input.contains(',') {
            return Err(format!("\"{input}\" Missing comma"));
        } else {
            return Err(format!("\"{input}\" Format error"));
        }
    }
    Ok(())
}

pub fn process_simple(
    code: &str,
    custom_include: &mut impl FnMut(&str) -> Box<dyn std::io::Read>,
) -> String {
    let mut macros: HashMap<String, Definition> = HashMap::new();
    let contents = process_internal(code, &mut macros, custom_include);
    contents
}

fn process_internal(
    code: &str,
    macros: &mut HashMap<String, Definition>,
    custom_include: &mut impl FnMut(&str) -> Box<dyn std::io::Read>,
) -> String {
    let mut lexer = crate::c_lexer::Lexer::new(code);
    let mut contents = String::with_capacity(code.len() * 2);
    let mut include_state: bool = false;
    let mut define_definition: Option<Definition> = None;
    let mut resolve_definition: Option<ResolveDefinition> = None;

    let mut macro_branch_stack: MacroBranchStack = MacroBranchStack::new();

    while let Some(item) = lexer.next() {
        let branch_result = macro_branch_stack.push(macros, lexer.previous(), &item);

        if branch_result.is_consume {
            lexer.enqueue_last(item);
            continue;
        }

        if define_definition.is_some() && resolve_definition.is_some() {
            panic!();
        }
        if let Some(define_definition_mut) = &mut define_definition {
            let is_stop = define_definition_mut.push(lexer.previous(), &item);
            if is_stop {
                let name = define_definition_mut.name.as_ref().expect("Valid");
                let _ = macros.insert(name.clone(), define_definition_mut.clone());
                define_definition = None;
            } else {
                lexer.enqueue_last(item);
                continue;
            }
        }
        if let Some(resolve_definition_mut) = &mut resolve_definition {
            let is_stop = resolve_definition_mut.push(lexer.previous(), &item);
            if is_stop {
                contents.push_str(&resolve_definition_mut.content);
                resolve_definition = None;
            } else {
                lexer.enqueue_last(item);
                continue;
            }
        }
        match item.ty {
            crate::c_lexer::TokType::KwDefine => {
                if let Some(last) = lexer.last() {
                    if matches!(last.ty, crate::c_lexer::TokType::Hash) {
                        define_definition = Some(Definition::new());
                    }
                }
            }
            crate::c_lexer::TokType::KwUndef => {
                unimplemented!();
            }
            crate::c_lexer::TokType::KwInclude => {
                if let Some(last) = lexer.last() {
                    if matches!(last.ty, crate::c_lexer::TokType::Hash) {
                        include_state = true;
                    }
                }
            }
            crate::c_lexer::TokType::Ident => {
                if let Some(def) = macros.get(item.str) {
                    resolve_definition = Some(ResolveDefinition::new(def.clone()));
                } else {
                    contents.push_str(item.str);
                }
            }
            crate::c_lexer::TokType::Integer => {
                contents.push_str(item.str);
            }
            crate::c_lexer::TokType::String => {
                if include_state {
                    if item.str.starts_with("\"") && item.str.ends_with("\"") {
                        let re = regex::Regex::new(r#""([^"]*)""#).unwrap();
                        if let Some(captures) = re.captures(item.str) {
                            if captures.len() == 2 {
                                let capture = &captures[1];
                                let mut reader = custom_include(capture);
                                let mut file_contents = String::new();
                                let _ = reader.read_to_string(&mut file_contents).expect("Valid");
                                file_contents =
                                    process_internal(&file_contents, macros, custom_include);
                                contents.push_str(&file_contents);
                            }
                        }
                    }
                    include_state = false;
                } else {
                    contents.push_str(item.str);
                }
            }
            crate::c_lexer::TokType::Hash
            | crate::c_lexer::TokType::KwIfdef
            | crate::c_lexer::TokType::KwIfndef
            | crate::c_lexer::TokType::KwElif
            | crate::c_lexer::TokType::KwEndif
            | crate::c_lexer::TokType::KwDefined
            | crate::c_lexer::TokType::Backslash => {}
            crate::c_lexer::TokType::LParen
            | crate::c_lexer::TokType::KwIf
            | crate::c_lexer::TokType::KwElse
            | crate::c_lexer::TokType::RParen
            | crate::c_lexer::TokType::Other
            | crate::c_lexer::TokType::Newline
            | crate::c_lexer::TokType::Space => {
                contents.push_str(item.str);
            }
        }
        lexer.enqueue_last(item);
    }

    contents
}

pub struct Preprocessor {
    include_dirs: HashSet<String>,
    defines: HashMap<String, Definition>,
}

impl Preprocessor {
    pub fn new(include_dirs: HashSet<String>, defines: HashMap<String, String>) -> Self {
        Self {
            include_dirs,
            defines: defines
                .iter()
                .map(|(x, x1)| (x.clone(), Definition::input(x.clone(), x1)))
                .collect(),
        }
    }

    pub fn empty() -> Self {
        Self {
            include_dirs: HashSet::new(),
            defines: HashMap::new(),
        }
    }

    pub fn add_include_dir(&mut self, dir: String) {
        self.include_dirs.insert(dir);
    }

    pub fn add_define(&mut self, name: String, value: String) {
        self.defines
            .insert(name.clone(), Definition::input(name, &value));
    }

    pub fn process(&mut self, contents: &str) -> crate::error::Result<String> {
        let contents = process_internal(contents, &mut self.defines, &mut |file| {
            let include_dirs = &self.include_dirs;
            for dir in include_dirs {
                let path = std::path::Path::new(dir).join(file);
                if path.exists() {
                    let file_contents =
                        std::fs::read_to_string(&path).expect(&format!("{:?}", &path));
                    let buffer = file_contents.as_bytes().to_vec();
                    let buff = std::io::Cursor::new(buffer);
                    return Box::new(buff);
                }
            }
            panic!("#include \"{}\"", file);
        });
        Ok(contents)
    }

    pub fn process_file<P>(&mut self, path: P) -> crate::error::Result<String>
    where
        P: AsRef<std::path::Path>,
    {
        if path.as_ref().exists() {
            let file_contents =
                std::fs::read_to_string(&path).expect(&format!("{:?}", path.as_ref()));
            return self.process(&file_contents);
        } else {
            panic!();
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::processor::{Definition, MacroBranch, process_simple};
    use std::collections::HashMap;

    fn process_include(file: &str) -> Box<dyn std::io::Read> {
        let _ = file;
        let contents = String::from(
            "#ifndef A
#define A
B
#endif",
        );
        let buffer = contents.as_bytes().to_vec();
        let buff = std::io::Cursor::new(buffer);
        Box::new(buff)
    }

    #[test]
    pub fn test0() {
        let code = "#ifdef MAX_BONES
#define MAX_BONES 1
MAX_BONES
qwe
#endif
\"abc\"
#define GROUP_BINDING(x) @group(x ## _GROUP) \\
@binding(x ## _BINDING)
GROUP_BINDING(    GLOBAL_CONSTANTS    )";

        let mut macros: HashMap<String, Definition> = HashMap::new();
        macros.insert(
            format!("MAX_BONES"),
            Definition::input(format!("MAX_BONES"), "1"),
        );
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!(
            contents,
            "\n\n 1\nqwe\n\n\"abc\"\n\n @group(GLOBAL_CONSTANTS_GROUP) \n@binding(GLOBAL_CONSTANTS_BINDING))"
        );
    }

    #[test]
    pub fn test13() {
        let code = "#include \"1\"
#include \"1\"
#include \"1\"
#include \"1\"
#include \"1\"
#include \"1\"";

        let mut macros: HashMap<String, Definition> = HashMap::new();
        let contents = crate::processor::process_internal(code, &mut macros, &mut process_include);
        assert_eq!(contents, " \n\nB\n\n \n \n \n \n ");
        assert_eq!(macros.get("A").is_some(), true);
        assert_eq!(macros.get("A").unwrap().content, "");
    }

    #[test]
    pub fn test12() {
        let code = "#ifdef MAX_BONES
#define MAX_BONES 1
MAX_BONES
qwe
#endif";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        macros.insert(
            format!("MAX_BONES"),
            Definition::input(format!("MAX_BONES"), "1"),
        );
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!(contents, "\n\n 1\nqwe\n");
    }

    #[test]
    fn test1() {
        assert!(crate::processor::validate("x y").is_err());
        assert!(crate::processor::validate("x, y").is_ok());
        assert!(crate::processor::validate(" x, y").is_ok());
        assert!(crate::processor::validate("xa io, pw").is_err());
        assert!(crate::processor::validate("xa, io, pw").is_ok());
        assert!(crate::processor::validate("pw1, io").is_err());
        assert!(crate::processor::validate("").is_err());
        assert!(crate::processor::validate("a").is_ok());
    }

    #[test]
    fn test3() {
        let content = "    (                              def, y )ghi(jkl)";
        let (args, new_content) = Definition::args(content);
        assert_eq!(args, vec!["def", "y"]);
        assert_eq!(new_content, "ghi(jkl)");
    }

    #[test]
    fn test4() {
        let content = "    1 ";
        let (args, new_content) = Definition::args(content);
        assert!(args.is_empty());
        assert_eq!(new_content, "    1 ");
    }

    #[test]
    fn test5() {
        let definition = Definition::input(String::from("A"), " ( x) x");
        assert_eq!(definition.args[0], "x");
        assert_eq!(definition.content, " x");
    }

    #[test]
    fn test6() {
        let definition = Definition::input(String::from("A"), "  x");
        assert!(definition.args.is_empty());
        assert_eq!(definition.content, "  x");
    }

    #[test]
    fn test7() {
        let code = "#if 2
    #if 0
        #if 0 + 3
            a
        #else
            b
        #endif
    #else
        c
    #endif
#else
    #if 0
        d
    #else
        e
    #endif
#endif";

        let contents = process_simple(code, &mut |_| {
            todo!();
        });
        assert_eq!(contents, "\n    \n        c\n    \n");
    }

    #[test]
    fn test9() {
        let code = "#if 0
    a
#elif 8
    b
#elif 1
    c           
#else
    d
#endif";
        let contents = process_simple(code, &mut |_| {
            todo!();
        });
        assert_eq!("\n    b\n", contents);
    }

    #[test]
    fn test10() {
        let code = "#ifdef MAX
a
#endif";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        macros.insert(format!("MAX"), Definition::input(format!("MAX"), "1"));
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!(contents, "\na\n");
    }

    #[test]
    fn test11() {
        let code = "#ifndef MAX
a
#else
b
#endif";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        macros.insert(format!("MAX"), Definition::input(format!("MAX"), "1"));
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!(contents, "\nb\n");
    }

    #[test]
    fn test8() {
        let mut macros: HashMap<String, Definition> = HashMap::new();
        macros.insert(
            "MAX".to_string(),
            Definition::input("MAX".to_string(), "true"),
        );
        assert_eq!(
            MacroBranch::resolve_no_args_definition("MAX".to_string(), &macros),
            true
        );
        assert_eq!(
            MacroBranch::resolve_no_args_definition(" MAX ".to_string(), &macros),
            true
        );
        assert_eq!(
            MacroBranch::resolve_no_args_definition(" MAX".to_string(), &macros),
            true
        );
        assert_eq!(
            MacroBranch::resolve_no_args_definition(" MAX && 0".to_string(), &macros),
            false
        );
    }

    #[test]
    fn test14() {
        let code = "if";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!(contents, "if");
    }

    #[test]
    fn test15() {
        let code = "if 1 {
    return 1.0;
} else {
    return 0.0;
}";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!(
            contents,
            "if 1 {\n    return 1.0;\n} else {\n    return 0.0;\n}"
        );
    }

    #[test]
    fn test16() {
        let code = "#ifndef A
#define A
if true {
} else {
}
#endif";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            todo!();
        });
        assert_eq!("\n\nif true {\n} else {\n}\n", contents);
    }

    #[test]
    fn test17() {
        let code = "#include \"A\"
#include \"A\"";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            let contents = "#ifndef B
#define B
if 1 {
} else {
}
#endif";
            let buffer = contents.as_bytes().to_vec();
            let buff = std::io::Cursor::new(buffer);
            Box::new(buff)
        });
        assert_eq!(" \n\nif 1 {\n} else {\n}\n\n ", contents);
    }

    #[test]
    fn test18() {
        let code = "#define B
#ifndef B
if 1 {
} else {
}
#endif";
        let mut macros: HashMap<String, Definition> = HashMap::new();
        let contents = crate::processor::process_internal(code, &mut macros, &mut |_| {
            panic!();
        });
        assert_eq!("\n", contents);
    }
}
