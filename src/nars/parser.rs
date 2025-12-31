use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{map, map_res, opt, recognize, value, all_consuming},
    multi::separated_list0,
    sequence::{delimited, pair, tuple, preceded},
    IResult,
    Parser,
};
use super::term::{Term, Operator, VarType};
use super::sentence::{Sentence, Punctuation, Stamp};
use super::truth::TruthValue;

// --- Helpers ---

fn is_alphanumeric_or_underscore(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '+'
}

fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

// --- Truth Value ---

fn parse_float(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize(pair(
            opt(char('-')),
            pair(digit1, opt(pair(char('.'), digit1)))
        )),
        |s: &str| s.parse::<f32>()
    ).parse(input)
}

fn parse_truth_value(input: &str) -> IResult<&str, TruthValue> {
    let (input, _) = char('%')(input)?;
    let (input, frequency) = parse_float(input)?;
    let (input, confidence) = opt(preceded(char(';'), parse_float)).parse(input)?;
    let (input, _) = opt(char('%')).parse(input)?;
    Ok((input, TruthValue::new(frequency, confidence.unwrap_or(0.9))))
}

// --- Terms ---

fn parse_atom(input: &str) -> IResult<&str, Term> {
    map(take_while1(is_alphanumeric_or_underscore), |s: &str| {
        Term::atom_from_str(s)
    }).parse(input)
}

fn parse_variable(input: &str) -> IResult<&str, Term> {
    let (input, prefix) = one_of("$#?")(input)?;
    let (input, name) = take_while1(is_alphanumeric_or_underscore)(input)?;
    
    let var_type = match prefix {
        '$' => VarType::Independent,
        '#' => VarType::Dependent,
        '?' => VarType::Query,
        _ => unreachable!(),
    };
    
    Ok((input, Term::var_from_str(var_type, name)))
}

fn parse_set_ext(input: &str) -> IResult<&str, Term> {
    let (input, args) = delimited(
        char('{'),
        separated_list0(ws(char(',')), parse_term),
        char('}')
    ).parse(input)?;
    Ok((input, Term::Compound(Operator::ExtSet, args)))
}

fn parse_set_int(input: &str) -> IResult<&str, Term> {
    let (input, args) = delimited(
        char('['),
        separated_list0(ws(char(',')), parse_term),
        char(']')
    ).parse(input)?;
    Ok((input, Term::Compound(Operator::IntSet, args)))
}

fn parse_copula(input: &str) -> IResult<&str, Operator> {
    alt((
        value(Operator::Inheritance, tag("-->")),
        value(Operator::Similarity, tag("<->")),
        value(Operator::Implication, tag("==>")),
        value(Operator::Equivalence, tag("<=>")),
        value(Operator::Instance, tag("{--")),
        value(Operator::Property, tag("--]")),
        value(Operator::InstanceProperty, tag("{-]")),
        value(Operator::ConcurrentImplication, tag("=|>")),
        value(Operator::PredictiveImplication, tag("=/>")),
        value(Operator::RetrospectiveImplication, tag("=\\>")),
        value(Operator::ConcurrentEquivalence, tag("<|>")),
        value(Operator::PredictiveEquivalence, tag("</>")),
        value(Operator::RetrospectiveEquivalence, tag("<\\>")),
    )).parse(input)
}

fn parse_term_operator(input: &str) -> IResult<&str, Operator> {
    alt((
        value(Operator::Product, tag("*")),
        value(Operator::Conjunction, tag("&&")), // Longer tags first
        value(Operator::Disjunction, tag("||")),
        value(Operator::ParallelEvents, tag("&|")),
        value(Operator::SequentialEvents, tag("&/")),
        value(Operator::Negation, tag("--")),
        value(Operator::ExtIntersection, tag("|")),
        value(Operator::IntIntersection, tag("&")),
        value(Operator::ExtImage, tag("/")),
        value(Operator::IntImage, tag("\\")),
        value(Operator::Difference, tag("-")),
        value(Operator::Difference, tag("~")),
        value(Operator::List, tag("#")),
    )).parse(input)
}

fn parse_operation(input: &str) -> IResult<&str, Operator> {
    let (input, _) = char('^')(input)?;
    let (input, name) = take_while1(is_alphanumeric_or_underscore)(input)?;
    Ok((input, Operator::Other(format!("^{}", name))))
}

fn parse_prefix_compound(input: &str) -> IResult<&str, Term> {
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = alt((parse_copula, parse_term_operator, parse_operation)).parse(input)?;
    let (input, _) = multispace0(input)?;
    // Optional comma after operator
    let (input, _) = opt(char(',')).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, args) = separated_list0(ws(char(',')), parse_term).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, Term::Compound(op, args)))
}

fn parse_infix_compound(input: &str) -> IResult<&str, Term> {
    let (input, _) = char('<')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, left) = parse_term(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = alt((parse_copula, parse_term_operator)).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, right) = parse_term(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('>')(input)?;
    Ok((input, Term::Compound(op, vec![left, right])))
}

fn parse_term_recursive(input: &str) -> IResult<&str, Term> {
    alt((
        parse_set_ext,
        parse_set_int,
        parse_prefix_compound,
        parse_infix_compound,
        parse_variable,
        parse_atom,
    )).parse(input)
}

pub fn parse_term(input: &str) -> IResult<&str, Term> {
    ws(parse_term_recursive).parse(input)
}

// --- Sentence ---

fn parse_punctuation(input: &str) -> IResult<&str, Punctuation> {
    alt((
        value(Punctuation::Judgement, char('.')),
        value(Punctuation::Question, char('?')),
        value(Punctuation::Goal, char('!')),
        value(Punctuation::Quest, char('@')),
    )).parse(input)
}

fn parse_tense(input: &str) -> IResult<&str, &str> {
    alt((
        tag(":|:"),
        tag(":/:"),
        tag(":\\:"),
        recognize(delimited(char(':'), take_while1(|c| c != ':'), char(':'))),
    )).parse(input)
}

pub fn parse_narsese(input: &str) -> Result<Sentence, String> {
    let parser = tuple((
        opt(ws(parse_tense)),
        parse_term,
        ws(parse_punctuation),
        opt(ws(parse_tense)), // Tense can be after punctuation too
        opt(ws(parse_truth_value)),
    ));

    let (_, (tense1, term, punctuation, tense2, truth_opt)) = all_consuming(ws(parser)).parse(input)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Default truth value if not present
    let truth = truth_opt.unwrap_or_else(|| {
        match punctuation {
            Punctuation::Judgement => TruthValue::new(1.0, 0.9),
            Punctuation::Goal => TruthValue::new(1.0, 0.9),
            Punctuation::Question => TruthValue::new(0.0, 0.0),
            Punctuation::Quest => TruthValue::new(0.0, 0.0),
        }
    });

    let stamp = Stamp {
        creation_time: 0,
        evidence: vec![],
    };

    Ok(Sentence::new(term, punctuation, truth, stamp))
}
