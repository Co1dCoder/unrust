use nom::types::CompleteStr;
use token::*;
use operator::Operator;
use expression::{array_expression_specifier, assignment_expression, Expression};
use nom::IResult;

type CS<'a> = CompleteStr<'a>;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionPrototype {
    ret_type: FullyTypeSpecifier,
    name: Identifier,
    params: Vec<ParamDeclaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
    Attribute,
    Varying,
    InvariantVarying,
    Uniform,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrecisionQualifier {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSpecifier {
    pub precision: Option<PrecisionQualifier>,
    pub actual_type: BasicType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FullyTypeSpecifier {
    pub qualifer: Option<TypeQualifier>,
    pub type_spec: TypeSpecifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamQualifier {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDeclaration {
    pub type_qualifer: Option<TypeQualifier>,
    pub param_qualifier: Option<ParamQualifier>,
    pub type_spec: TypeSpecifier,
    pub name: Option<Identifier>,
    pub array_spec: Option<Expression>,
}

named!(
    #[allow(unused_imports)], // fix value! bug
    param_qualifier<CS, ParamQualifier>,
    alt!(
        value!(ParamQualifier::InOut, tag!("inout")) |
        value!(ParamQualifier::In, tag!("in")) |
        value!(ParamQualifier::Out, tag!("out"))
    )
);

named!(param_declaration<CS, ParamDeclaration>,
    ows!(do_parse!(
        tq: opt!(type_qualifier) >>
        pq: opt!(param_qualifier) >>
        ts: type_specifier >>
        n:  opt!(identifier) >>
        a:  opt!(array_expression_specifier) >>
        (ParamDeclaration{
            type_qualifer : tq,
            param_qualifier: pq,
            type_spec: ts,
            name: n,
            array_spec: a
        })
    ))
);

named!(
    #[allow(unused_imports)], // fix value! bug
    precision_qualifier<CS, PrecisionQualifier>,
    alt!(
        value!(PrecisionQualifier::High, tag!("highp")) |
        value!(PrecisionQualifier::Medium, tag!("mediump")) |
        value!(PrecisionQualifier::Low, tag!("lowp"))
    )
);

named!(  
    type_specifier<CS, TypeSpecifier>,     
    ows!(do_parse!(
        p: opt!(precision_qualifier) >>
        t: alt!(basic_type | map!(identifier, BasicType::TypeName)) >>
        (TypeSpecifier {
            precision : p,
            actual_type : t
        })
    ))
);

named!(
    #[allow(unused_imports)], // fix value! bug
    type_qualifier<CS, TypeQualifier>,
    alt!(
        value!(TypeQualifier::Const, tag!("const")) |
        value!(TypeQualifier::Attribute, tag!("attribute")) |
        value!(TypeQualifier::InvariantVarying, pair!( tag!("invariant"),tag!("varying"))) |
        value!(TypeQualifier::Varying, tag!("varying")) |        
        value!(TypeQualifier::Uniform, tag!("uniform"))
    )
);

named!(
    full_type_specifier<CS, FullyTypeSpecifier>,     
    ows!(do_parse!(
        q: opt!(type_qualifier) >>
        ts: type_specifier >>
        (FullyTypeSpecifier {
            qualifer : q,
            type_spec : ts
        })
    ))
);

named!(function_prototype<CS, FunctionPrototype>, 
    ows!(do_parse!(
        ts : full_type_specifier >>
        ident: identifier >>
        op!(Operator::LeftParen) >>
        params: ows!(separated_list!(op!(Operator::Comma), param_declaration)) >>
        op!(Operator::RightParen) >>        

        (FunctionPrototype {
            ret_type: ts,
            name: ident,
            params: params
        })
    ))
);

#[derive(Debug, Clone, PartialEq)]
pub enum VariantTypeSpecifier {
    Normal(FullyTypeSpecifier),
    Invariant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SingleDeclaration {
    type_spec: VariantTypeSpecifier,
    name: Option<Identifier>,
    array_spec: Option<Expression>,
    equal_to: Option<Expression>,
}

named!(initializer<CS, Expression>,
    call!(assignment_expression)
);

named!(
    #[allow(unused_imports)], // fix value! bug
    single_declaration<CS,SingleDeclaration>,
    ows!(alt!(
        do_parse!(
            ts: value!(VariantTypeSpecifier::Invariant, tag!("invariant")) >>
            n: identifier >> 
            (SingleDeclaration{
                type_spec: ts,
                name: Some(n),
                array_spec: None,
                equal_to: None,
            })
        ) | 
        do_parse!(
            ts : map!(full_type_specifier, VariantTypeSpecifier::Normal) >> 
            n : identifier >>
            eq : preceded!(op!(Operator::Equal), initializer) >>
            (SingleDeclaration{
                type_spec: ts,
                name: Some(n),
                array_spec: None,
                equal_to: Some(eq),
            })            
        ) |
        do_parse!(
            ts : map!(full_type_specifier, VariantTypeSpecifier::Normal) >> 
            n : opt!(identifier) >>
            a : opt!(array_expression_specifier) >> 
            (SingleDeclaration{
                type_spec: ts,
                name: n,
                array_spec: a,
                equal_to: None,
            })            
        )
    ))    
);

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    FunctionPrototype(FunctionPrototype),
    DeclarationList(Vec<SingleDeclaration>),
    Precision(PrecisionQualifier, BasicType),
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
fn declaration_list_part<'a>(input: CompleteStr<'a>, sd: &SingleDeclaration) -> IResult<CompleteStr<'a>, SingleDeclaration> {
    ows!(input, preceded!(        
        op!(Operator::Comma),
        alt!(
            do_parse!(
                n: identifier >> 
                a: array_expression_specifier >> 
                (SingleDeclaration {
                    type_spec: sd.type_spec.clone(),
                    name: Some(n),
                    array_spec: Some(a),
                    equal_to: None,
                })
            ) 
            | do_parse!(
                n: identifier >> 
                eq: preceded!(op!(Operator::Equal), initializer) >> 
                (SingleDeclaration {
                    type_spec: sd.type_spec.clone(),
                    name: Some(n),
                    array_spec: None,
                    equal_to: Some(eq),
                })
            )
            | do_parse!(
                n: identifier >>                
                (SingleDeclaration {
                    type_spec: sd.type_spec.clone(),
                    name: Some(n),
                    array_spec: None,
                    equal_to: None,
                })
            )
        )
    ))
}

#[cfg_attr(rustfmt, rustfmt_skip)] 
named!(declaration_list<CS,Vec<SingleDeclaration>>,
    do_parse!(
        sd: single_declaration >>
        ls: many0!(call!(declaration_list_part, &sd))  >>         
        ({
            let mut r = ls.clone();
            r.insert(0, sd);
            r
        })
    )
);

named!(pub declaration<CS, Declaration>,
    ows!( terminated!(alt!(
        map!(function_prototype, Declaration::FunctionPrototype) |
        map!(declaration_list, Declaration::DeclarationList) |
        do_parse!(
            tag!("precision") >>
            pq: precision_qualifier >>
            ts: basic_type >>
            (Declaration::Precision(pq, ts))
        )        
    ), op!(Operator::SemiColon) ))
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_function_prototype_no_params() {
        let i = function_prototype(CompleteStr("const highp vec3 f()"));

        assert_eq!(format!("{:?}", 
            i.unwrap().1), 
            "FunctionPrototype { ret_type: FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: TypeName(\"vec3\") } }, name: \"f\", params: [] }"
            );
    }

    #[test]
    fn parse_param_decl() {
        let i = param_declaration(CompleteStr("vec3 a)"));

        assert_eq!(format!("{:?}", 
            i.unwrap().1), 
            "ParamDeclaration { type_qualifer: None, param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: TypeName(\"vec3\") }, name: Some(\"a\"), array_spec: None }"
            );
    }

    #[test]
    fn parse_function_prototype_with_params() {
        let i = function_prototype(CompleteStr(
            "const highp vec3 f(const vec3 a, in Obj b , float a[2] )",
        ));

        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "FunctionPrototype { ret_type: FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: TypeName(\"vec3\") } }, name: \"f\", params: [ParamDeclaration { type_qualifer: Some(Const), param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: TypeName(\"vec3\") }, name: Some(\"a\"), array_spec: None }, ParamDeclaration { type_qualifer: None, param_qualifier: Some(In), type_spec: TypeSpecifier { precision: None, actual_type: TypeName(\"Obj\") }, name: Some(\"b\"), array_spec: None }, ParamDeclaration { type_qualifer: None, param_qualifier: None, type_spec: TypeSpecifier { precision: None, actual_type: Float }, name: Some(\"a\"), array_spec: Some(Constant(Integer(2))) }] }"
            );
    }

    #[test]
    fn parse_single_declaration() {
        let i = single_declaration(CompleteStr("const highp vec3 name"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: TypeName(\"vec3\") } }), name: Some(\"name\"), array_spec: None, equal_to: None }"
            );

        let i = single_declaration(CompleteStr("vec3 name[12]"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: TypeName(\"vec3\") } }), name: Some(\"name\"), array_spec: Some(Constant(Integer(12))), equal_to: None }"
            );

        let i = single_declaration(CompleteStr("float name = 10"));
        assert_eq!(format!("{:?}", 
            i.unwrap().1), 
            "SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: None, type_spec: TypeSpecifier { precision: None, actual_type: Float } }), name: Some(\"name\"), array_spec: None, equal_to: Some(Constant(Integer(10))) }"
            );
    }

    #[test]
    fn parse_declaration() {
        let i = declaration(CompleteStr("const highp vec3 name;"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: TypeName(\"vec3\") } }), name: Some(\"name\"), array_spec: None, equal_to: None }])"
            );

        let i = declaration(CompleteStr("const highp vec3 a, b;"));
        assert_eq!(format!("{:?}",
            i.unwrap().1),
            "DeclarationList([SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: TypeName(\"vec3\") } }), name: Some(\"a\"), array_spec: None, equal_to: None }, SingleDeclaration { type_spec: Normal(FullyTypeSpecifier { qualifer: Some(Const), type_spec: TypeSpecifier { precision: Some(High), actual_type: TypeName(\"vec3\") } }), name: Some(\"b\"), array_spec: None, equal_to: None }])"
            );
    }

}