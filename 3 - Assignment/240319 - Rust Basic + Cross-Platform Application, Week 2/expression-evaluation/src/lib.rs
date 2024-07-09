// An operation to perform on two subexpressions.
#[derive(Debug)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

// An expression, in tree form.
#[derive(Debug)]
enum Expression {
    // An operation on two subexpressions.
    Op {
        op: Operation,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // A literal value
    Value(i64),
}

fn eval(e: Expression) -> Result<i64, String> {
    // if Expression has no operation(Op), just return Expression::Value. (match 문 사용)
    // if Expression::Value: ~
    match e {
        Expression::Value(value) => return Ok(value), // base case 
        Expression::Op {op, left, right}    => { 
            let left_value = eval(*left)?;    // in case of embedded Expression, evaluate recursively 
            let right_value = eval(*right)?;
            match op {
                // if Expression::Op : 
                // do Operation on de-referenced left and right value
                Operation::Add => return Ok(left_value + right_value),    
                Operation::Sub => return Ok(left_value - right_value),
                Operation::Mul => return Ok(left_value * right_value),
                Operation::Div => {
                    if right_value == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(left_value / right_value) 
                    }
                }
            }
        },
    } 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        assert_eq!(eval(Expression::Value(19)), Ok(19));
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            eval(Expression::Op {
                op: Operation::Add,
                left: Box::new(Expression::Value(10)),
                right: Box::new(Expression::Value(20)),
            }),
            Ok(30)
        );
    }

    #[test]
    fn test_recursion() {
        let term1 = Expression::Op {
            op: Operation::Mul,
            left: Box::new(Expression::Value(10)),
            right: Box::new(Expression::Value(9)),
        };
        let term2 = Expression::Op {
            op: Operation::Mul,
            left: Box::new(Expression::Op {
                op: Operation::Sub,
                left: Box::new(Expression::Value(3)),
                right: Box::new(Expression::Value(4)),
            }),
            right: Box::new(Expression::Value(5)),
        };
        assert_eq!(
            eval(Expression::Op {
                op: Operation::Add,
                left: Box::new(term1),
                right: Box::new(term2),
            }),
            Ok(85)
        );
    }

    #[test]
    fn test_error() {
        assert_eq!(
            eval(Expression::Op {
                op: Operation::Div,
                left: Box::new(Expression::Value(99)),
                right: Box::new(Expression::Value(0)),
            }),
            Err(String::from("Division by zero"))
        );
    }
}
