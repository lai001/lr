use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[\r\n]")]
#[rustfmt::skip]
pub enum TokType {

    #[token("#")] Hash,
    #[token("define")] KwDefine,
    #[token("undef")]  KwUndef,
    #[token("include")] KwInclude,
    #[token("if")]     KwIf,
    #[token("ifdef")]  KwIfdef,
    #[token("ifndef")] KwIfndef,
    #[token("elif")]   KwElif,
    #[token("else")]   KwElse,
    #[token("endif")]  KwEndif,

    #[token("defined")]  KwDefined,

    #[token("\n",  priority = 3)] Newline,
    #[token("\\")] Backslash,
    #[regex(r" |\t")] Space,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")] Ident,
    #[regex(r"0[xX][0-9A-Fa-f]+|[0-9]+")] Integer,
    #[regex(r#""([^"\\]|\\.)*""#)] String,

    // #[token("||")] OrOr,
    // #[token("&&")] AndAnd,
    // #[token("==")] EqEq,
    // #[token("!=")] Ne,
    // #[token("<=")] Le,
    // #[token(">=")] Ge,
    // #[token("<<")] Shl,
    // #[token(">>")] Shr,
    // #[token("#")]  HashOp,
    // #[token("##")] Concat,
    // #[token("+")]  Plus,
    // #[token("-")]  Minus,
    // #[token("*")]  Star,
    // #[token("/")]  Slash,
    // #[token("%")]  Percent,
    // #[token("<")]  Lt,
    // #[token(">")]  Gt,
    // #[token("&")]  And,
    // #[token("|")]  Or,
    // #[token("^")]  Xor,
    // #[token("~")]  Tilde,
    // #[token("!")]  Bang,

    #[token("(")] LParen,
    #[token(")")] RParen,
    // #[token(",")] Comma,

    #[regex(r"[^\s]", priority = 0)] Other,
}

pub type Span = std::ops::Range<usize>;
// pub type Item<'a> = (TokType, Span, &'a str);

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub ty: TokType,
    pub span: Span,
    pub str: &'a str,
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokType>,
    // previous: Vec<(TokType, Span, &'a str)>,
    previous: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: TokType::lexer(input),
            previous: vec![],
        }
    }
    pub fn next(&mut self) -> Option<Token<'a>> {
        let tok = self.inner.next()?;
        let span = self.inner.span();
        let slice = &self.inner.source()[span.clone()];
        let t = Token {
            ty: tok.ok()?,
            span,
            str: slice,
        };
        Some(t)
        // Some((tok.ok()?, span, slice))
    }

    pub fn enqueue_last(&mut self, last: Token<'a>) {
        self.previous.push(last);
    }

    pub fn last(&self) -> Option<&Token<'a>> {
        self.previous.last()
    }

    pub fn previous(&self) -> &[Token<'a>] {
        &self.previous
    }
}
