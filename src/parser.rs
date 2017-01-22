use nom::{multispace, oct_digit, digit, hex_digit};
use std::str;

#[derive(Debug, Clone)]
pub enum Type {
    Bool,
    Int,
    Uint,
    Hyper,
    Uhyper,
    Float,
    Double,
    Quadruple,
}

#[derive(Debug, Clone)]
pub enum Token {
    Constant(i64),
    Type(Type),
    Enum(Vec<(Token, Token)>),
    Struct(Vec<Token>),
    Ident(String),
    Blank,
    VoidDecl,
    Decl{ty: Box<Token>, id: Box<Token>},
    PointerDecl{ty: Box<Token>, id: Box<Token>},
    OpaqueDecl{id: Box<Token>, size: Box<Token>},
    StringDecl{id: Box<Token>, size: Box<Option<Token>>},
    VarOpaqueDecl{id: Box<Token>, size: Box<Option<Token>>},
    ArrayDecl{ty: Box<Token>, id: Box<Token>, size: Box<Token>},
    VarArrayDecl{ty: Box<Token>, id: Box<Token>, size: Box<Option<Token>>},
    ConstantDef(Box<Token>),
    TypeDef(Box<Token>),
    EnumDef{id: Box<Token>, decl: Box<Token>},
    StructDef{id: Box<Token>, decl: Box<Token>},
    UnionDef{id: Box<Token>, decl: Box<Token>},
    UnionCase {
        vals: Vec<Token>,
        decl: Box<Token>
    },
    Union {
        decl: Box<Token>,
        cases: Vec<Token>,
        default: Box<Option<Token>>
    },
    Comment(String),
    CodeSnippet(String),
}

named!(eol, tag!("\n"));

named!(inline_comment, do_parse!(
        tag!("//") >>
        comment: take_until!("\n") >>
        (comment)
    )
);


named!(blank<Token>,
    do_parse!(
        alt!(multispace | eol) >>
        (Token::Blank)
    )
);

named!(code_snippet<Token>,
    do_parse!(
        tag!("%") >>
        comment: take_until_and_consume!("\n") >>
        (Token::CodeSnippet(String::from_utf8(comment.to_vec()).unwrap()))
    )
);

named!(comment<Token>,
    do_parse!(
        tag!("//") >>
        comment: take_until_and_consume!("\n") >>
        (Token::Comment(String::from_utf8(comment.to_vec()).unwrap()))
    )
);

// Based on XDR spec RFC4506

named!(definition<Token>, alt!(type_def | constant_def));
named!(value<Token>, alt!(constant | identifier));

named!(type_def<Token>,
    alt!(
        do_parse!(
            tag!("typedef")     >>
            multispace          >>
      decl: declaration         >>
            opt!(multispace)    >>
            tag!(";")           >>
            (Token::TypeDef(Box::new(decl)))
        ) |
        do_parse!(
            tag!("enum")        >>
            multispace          >>
     ident: identifier          >>
            multispace          >>
      decl: enum_body           >>
            opt!(multispace)    >>
            tag!(";")           >>
            (Token::EnumDef{
                id: Box::new(ident),
                decl: Box::new(Token::Enum(decl))
            })
        ) |
        do_parse!(
            tag!("union")       >>
            multispace          >>
     ident: identifier          >>
            multispace          >>
      decl: union_body          >>
            opt!(multispace)    >>
            tag!(";")           >>
            (Token::UnionDef{
                id: Box::new(ident),
                decl: Box::new(decl)
            })
        ) |
        do_parse!(
            tag!("struct")      >>
            multispace          >>
     ident: identifier          >>
            multispace          >>
      decl: struct_body         >>
            opt!(multispace)    >>
            tag!(";")           >>
            opt!(multispace)    >>
            (Token::StructDef{
                id: Box::new(ident),
                decl: Box::new(Token::Struct(decl))
            })
        )
    )
);

fn parse_num(u8arr: &[u8], base: u32) -> i64 {
    let s = str::from_utf8(u8arr).unwrap();
    i64::from_str_radix(s, base).unwrap()
}

named!(constant<Token>,
    alt!(
        do_parse!(
            tag!("0x") >>
            digit: hex_digit >>
            (Token::Constant(parse_num(digit, 16)))
        ) |
        do_parse!(
            tag!("0") >>
            digit: oct_digit >>
            (Token::Constant(parse_num(digit, 8)))
        ) |
        do_parse!(
            digit: digit >>
            (Token::Constant(parse_num(digit, 10)))
        )
    )
);

named!(constant_def<Token>,
    do_parse!(
        tag!("const")       >>
        multispace          >>
        identifier          >>
        opt!(multispace)    >>
        tag!("=")           >>
        opt!(multispace)    >>
     c: constant            >>
        opt!(multispace)    >>
        tag!(";")           >>
        opt!(inline_comment)>>
        (Token::ConstantDef(Box::new(c)))
    )
);

named!(type_specifier<Token>,
    alt!(
        do_parse!(
            tag!("unsigned") >>
            multispace >>
            tag!("int") >>
            (Token::Type(Type::Uint))
        ) |
        do_parse!(
            tag!("int") >>
            (Token::Type(Type::Int))
        ) |
        do_parse!(
            tag!("unsigned") >>
            multispace >>
            tag!("hyper") >>
            (Token::Type(Type::Uhyper))
        ) |
        do_parse!(
            tag!("hyper") >>
            (Token::Type(Type::Hyper))
        ) |
        do_parse!(
            tag!("float") >>
            (Token::Type(Type::Float))
        ) |
        do_parse!(
            tag!("double") >>
            (Token::Type(Type::Double))
        ) |
        do_parse!(
            tag!("quadruple") >>
            (Token::Type(Type::Quadruple))
        ) |
        do_parse!(
            tag!("bool") >>
            (Token::Type(Type::Bool))
        ) |
        // These aren't standard XDR, but what I'm attempting to interop
        // with uses them like they are... so oh well
        do_parse!(
            tag!("u_int32_t") >>
            (Token::Type(Type::Uint))
        ) |
        do_parse!(
            tag!("int32_t") >>
            (Token::Type(Type::Int))
        ) |
        do_parse!(
            tag!("u_int64_t") >>
            (Token::Type(Type::Uhyper))
            ) |
        do_parse!(
            tag!("u_int64_t") >>
            (Token::Type(Type::Hyper))
            ) |
        // There are standard XDR again!
        enum_type_specifier |
        struct_type_specifier |
        union_type_specifier |
        identifier
    )
);

named!(union_type_specifier<Token>,
    do_parse!(
        tag!("union")        >>
        multispace           >>
 union: union_body           >>
        (union)
    )
);

named!(union_case<&[u8], Token>,
    do_parse!(
        opt!(multispace)        >>
 cases: many1!(case_def)        >>
        opt!(multispace)        >>
  decl: declaration             >>
        opt!(multispace)        >>
        tag!(";")               >>
        opt!(multispace)        >>
        opt!(inline_comment)    >>
        opt!(multispace)        >>
        (Token::UnionCase {
               vals: cases,
               decl: Box::new(decl)
        })
    )
);

named!(default_case<&[u8], Token>,
    do_parse!(
        opt!(multispace)        >>
        tag!("default:")        >>
        opt!(multispace)        >>
  decl: declaration             >>
        opt!(multispace)        >>
        tag!(";")               >>
        opt!(multispace)        >>
        opt!(inline_comment)    >>
        opt!(multispace)        >>
        (decl)
    )
);

named!(case_def<&[u8], Token>,
       do_parse!(
           tag!("case")         >>
           opt!(multispace)     >>
    value: value                >>
           opt!(multispace)     >>
           tag!(":")            >>
           opt!(multispace)     >>
           opt!(inline_comment) >>
           opt!(multispace)     >>
           (value)
       )
);

named!(union_body<&[u8], Token>,
       do_parse!(
           tag!("switch")       >>
           opt!(multispace)     >>
           tag!("(")            >>
           opt!(multispace)     >>
     decl: declaration          >>
           opt!(multispace)     >>
           tag!(")")            >>
           opt!(multispace)     >>
           tag!("{")            >>
    cases: many1!(union_case)   >>
           opt!(multispace)     >>
  default: opt!(default_case)   >>
           opt!(multispace)     >>
           tag!("}")            >>
           (Token::Union {
               decl: Box::new(decl),
               cases: cases,
               default: Box::new(default),
           })
       )
);

named!(enum_type_specifier<Token>,
       do_parse!(
           tag!("enum")         >>
           multispace           >>
     args: enum_body            >>
           (Token::Enum(args))
        )
);

named!(enum_body<&[u8], Vec<(Token, Token)> >,
       do_parse!(
           opt!(multispace)     >>
           tag!("{")            >>
           opt!(multispace)     >>
       kv: many0!(enum_kv)      >>
           opt!(multispace)     >>
           tag!("}")            >>
           (kv)
        )
);

named!(enum_kv<&[u8], (Token, Token)>,
    do_parse!(
        opt!(multispace)        >>
   key: identifier              >>
        opt!(multispace)        >>
        tag!("=")               >>
        opt!(multispace)        >>
   val: value                   >>
        opt!(multispace)        >>
        opt!(tag!(","))         >>
        opt!(multispace)        >>
        opt!(inline_comment)    >>
        opt!(multispace)        >>
        (key, val)
    )
);

named!(struct_type_specifier<Token>,
       do_parse!(
           tag!("struct")       >>
           multispace           >>
    decls: struct_body          >>
           (Token::Struct(decls))
        )
);

named!(struct_body<&[u8], Vec<(Token)> >,
       do_parse!(
           tag!("{")            >>
           opt!(multispace)     >>
    decls: many0!(struct_decls) >>
           opt!(multispace)     >>
           tag!("}")            >>
           (decls)
        )
);

named!(struct_decls<&[u8], Token>,
    do_parse!(
        opt!(multispace)        >>
  decl: declaration             >>
        opt!(multispace)        >>
        tag!(";")               >>
        opt!(multispace)        >>
        opt!(inline_comment)    >>
        opt!(multispace)        >>
        (decl)
    )
);

named!(declaration<Token>,
    alt!(
        do_parse!(
            tag!("opaque")      >>
            multispace          >>
        id: identifier          >>
            tag!("[")           >>
        sz: value               >>
            tag!("]")           >>
            (Token::OpaqueDecl {
                id: Box::new(id),
                size: Box::new(sz)
            })
        ) |
        do_parse!(
            tag!("opaque")      >>
            multispace          >>
        id: identifier          >>
            tag!("<")           >>
        sz: opt!(value)         >>
            tag!(">")           >>
            (Token::VarOpaqueDecl {
                id: Box::new(id),
                size: Box::new(sz)
            })
        ) |
        do_parse!(
            tag!("string")      >>
            multispace          >>
        id: identifier          >>
            tag!("<")           >>
        sz: opt!(value)         >>
            tag!(">")           >>
            (Token::StringDecl {
                id: Box::new(id),
                size: Box::new(sz)
            })
        ) |
        do_parse!(
        ty: type_specifier      >>
            multispace          >>
        id: identifier          >>
            tag!("[")           >>
        sz: value               >>
            tag!("]")           >>
            (Token::ArrayDecl {
                ty: Box::new(ty),
                id: Box::new(id),
                size: Box::new(sz)
            })
        ) |
        do_parse!(
        ty: type_specifier      >>
            multispace          >>
        id: identifier          >>
            tag!("<")           >>
        sz: opt!(value)         >>
            tag!(">")           >>
            (Token::VarArrayDecl {
                ty: Box::new(ty),
                id: Box::new(id),
                size: Box::new(sz)
            })
        ) |
        do_parse!(
        ty: type_specifier      >>
            opt!(multispace)    >>
            tag!("*")           >>
            multispace          >>
        id: identifier          >>
            (Token::PointerDecl {
                ty: Box::new(ty),
                id: Box::new(id)
            })
        ) |
        do_parse!(
        ty: type_specifier      >>
            multispace          >>
        id: identifier          >>
            (Token::Decl {
                ty: Box::new(ty),
                id: Box::new(id)
            })
        ) |
        do_parse!(
            tag!("void")        >>
            opt!(multispace)    >>
            (Token::VoidDecl)
        )
    )
);

named!(identifier<Token>,
    do_parse!(
        id: take_while1!(is_ident) >>
        (Token::Ident(String::from_utf8(id.to_vec()).unwrap()))
    )
);

#[inline]
pub fn is_ident(chr:u8) -> bool {
    (chr >= 0x41 && chr <= 0x5A) ||
    (chr >= 0x61 && chr <= 0x7A) ||
    (chr >= 0x30 && chr <= 0x39) ||
    chr == 0x5f
}

named!(expression<Token>,
    alt_complete!(definition | comment | code_snippet | blank)
);

named!(tokenize<Vec<Token> >, many0!(expression));

pub fn parse(i: &[u8], debug: bool) -> Option<Vec<Token>> {
    let parsed = tokenize(i);

    let p = if parsed.is_done(){
        Some(parsed.unwrap().1)
    } else {
        None
    };

    if debug {
        println!("{:?}", p);
    }
    p
}

