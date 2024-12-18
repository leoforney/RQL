#![allow(unused)]
#![allow(unused)]
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "rqle.pest"]
pub struct UpdateParser;

#[derive(Debug)]
pub struct Assignment {
    pub variable: String,
    pub expression: String,
}

#[derive(Debug)]
pub struct ExpressionParser {
    pub assignments: Vec<Assignment>,
}

impl ExpressionParser {
    pub fn parse(input: &str) -> Result<Self, String> {
        let pairs = UpdateParser::parse(Rule::update_stmt, input.trim())
            .map_err(|e| format!("Parse error: {}", e))?;
        let mut assignments = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::update_stmt => {
                    for inner_pair in pair.into_inner() {
                        match inner_pair.as_rule() {
                            Rule::assignments => {
                                for assignment_pair in inner_pair.into_inner() {
                                    if let Rule::assignment = assignment_pair.as_rule() {
                                        let mut inner = assignment_pair.into_inner();
                                        let variable = inner.next().unwrap().as_str().to_string();
                                        let expression = inner.next().unwrap().as_str().to_string();
                                        assignments.push(Assignment { variable, expression });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self { assignments })
    }
}