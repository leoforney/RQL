#[cfg(test)]
pub mod tests {
    use crate::rqle::rqle_parser::ExpressionParser;
    use pest::Parser;

    #[test]
    fn test_assign_name() {
        let input = "columnname3 = columnname2 - sin(columnname1)";

        match ExpressionParser::parse(input) {
            Ok(parsed) => {
                assert_eq!(parsed.assignments.len(), 1);
                assert_eq!(parsed.assignments.iter().next().unwrap().variable, "columnname3");
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }
}