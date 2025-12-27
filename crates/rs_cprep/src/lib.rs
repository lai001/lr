mod c_lexer;
pub mod error;
pub mod processor;

lalrpop_util::lalrpop_mod!(pub(crate) pp_expr);

#[cfg(test)]
mod test {
    use crate::pp_expr;

    #[test]
    fn pp_test() {
        let parser = pp_expr::PPParser::new();
        assert_eq!(parser.parse("22"), Ok(true));
        assert_eq!(parser.parse("(22)"), Ok(true));
        assert_eq!(parser.parse("((((22))))"), Ok(true));
        assert_eq!(parser.parse("11+22"), Ok(true));
        assert_eq!(parser.parse("true"), Ok(true));
        assert_eq!(parser.parse("false"), Ok(false));
        assert_eq!(parser.parse("1 || false"), Ok(true));
        assert_eq!(parser.parse("false || false"), Ok(false));
        assert_eq!(parser.parse("(0 || 0)"), Ok(false));
        assert_eq!(parser.parse("((false) || false)"), Ok(false));
        assert!(parser.parse("((22)").is_err());
    }
}
