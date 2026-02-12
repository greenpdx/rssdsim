/// Expression parser and evaluator

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Expression {
    Constant(f64),
    Variable(String),
    /// Variable with subscripts for array indexing
    /// Example: Population[Region] or Sales[Region, Product]
    SubscriptedVariable {
        name: String,
        subscripts: Vec<crate::model::SubscriptRef>,
    },
    BinaryOp {
        op: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    // Comparison operators (return 1.0 for true, 0.0 for false)
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum UnaryOperator {
    Negate,
}

impl Expression {
    /// Parse a simple expression from string
    /// For now, supports: constants, variables, and basic operations
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();

        // Try to parse as number
        if let Ok(num) = s.parse::<f64>() {
            return Ok(Expression::Constant(num));
        }

        // Check for IF THEN ELSE conditional (highest priority)
        if let Some(conditional) = Self::try_parse_conditional(s) {
            return conditional;
        }

        // Check for comparison operators
        // Priority: comparisons (lowest), + - , * / , ^ (highest)
        if let Some(expr) = Self::try_parse_comparison(s) {
            return Ok(expr);
        }

        // Look for + or - (lowest precedence)
        if let Some(expr) = Self::try_parse_binary(s, &['+', '-']) {
            return Ok(expr);
        }

        // Look for * or /
        if let Some(expr) = Self::try_parse_binary(s, &['*', '/']) {
            return Ok(expr);
        }

        // Look for ^
        if let Some(expr) = Self::try_parse_binary(s, &['^']) {
            return Ok(expr);
        }

        // Check for parentheses first (before function calls)
        if s.starts_with('(') && s.ends_with(')') {
            // Could be either parentheses or a function call
            // If there's no function name before the '(', it's just parentheses
            return Self::parse(&s[1..s.len() - 1]);
        }

        // Check for function call
        if let Some(paren_idx) = s.find('(') {
            if s.ends_with(')') && paren_idx > 0 {
                let func_name = s[..paren_idx].trim();

                // Skip if function name is empty (means it's just parentheses)
                if !func_name.is_empty() {
                    let args_str = &s[paren_idx + 1..s.len() - 1];

                    // Split arguments by commas, respecting nested parentheses
                    let arg_strings = Self::split_function_args(args_str);

                    // Parse each argument
                    let args: Result<Vec<_>, _> = arg_strings
                        .iter()
                        .map(|arg| Self::parse(arg.trim()))
                        .collect();

                    return Ok(Expression::FunctionCall {
                        name: func_name.to_string(),
                        args: args?,
                    });
                }
            }
        }

        // Check for unary minus
        if s.starts_with('-') {
            let inner = Self::parse(&s[1..])?;
            return Ok(Expression::UnaryOp {
                op: UnaryOperator::Negate,
                expr: Box::new(inner),
            });
        }

        // Check for subscripted variable: VarName[Sub1, Sub2, ...]
        if let Some(bracket_idx) = s.find('[') {
            if s.ends_with(']') {
                let var_name = s[..bracket_idx].trim();
                let subscripts_str = &s[bracket_idx + 1..s.len() - 1];

                // Parse subscripts (comma-separated)
                let subscript_strings = Self::split_function_args(subscripts_str);
                let subscripts: Vec<crate::model::SubscriptRef> = subscript_strings
                    .iter()
                    .map(|sub| {
                        let sub_trimmed = sub.trim();
                        // For now, treat all subscripts as elements
                        // Could be enhanced to detect wildcards (*) or dimensions
                        if sub_trimmed == "*" {
                            crate::model::SubscriptRef::Wildcard
                        } else {
                            crate::model::SubscriptRef::Element(sub_trimmed.to_string())
                        }
                    })
                    .collect();

                return Ok(Expression::SubscriptedVariable {
                    name: var_name.to_string(),
                    subscripts,
                });
            }
        }

        // Otherwise treat as variable name
        Ok(Expression::Variable(s.to_string()))
    }

    fn try_parse_binary(s: &str, ops: &[char]) -> Option<Expression> {
        // Find rightmost operator (for left-to-right evaluation)
        let mut depth = 0;
        let mut op_pos = None;
        let mut op_char = None;

        for (i, ch) in s.chars().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                c if depth == 0 && ops.contains(&c) => {
                    op_pos = Some(i);
                    op_char = Some(c);
                }
                _ => {}
            }
        }

        if let (Some(pos), Some(op)) = (op_pos, op_char) {
            let left = &s[..pos].trim();
            let right = &s[pos + 1..].trim();

            if left.is_empty() || right.is_empty() {
                return None;
            }

            let left_expr = Self::parse(left).ok()?;
            let right_expr = Self::parse(right).ok()?;

            let operator = match op {
                '+' => Operator::Add,
                '-' => Operator::Subtract,
                '*' => Operator::Multiply,
                '/' => Operator::Divide,
                '^' => Operator::Power,
                _ => return None,
            };

            return Some(Expression::BinaryOp {
                op: operator,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            });
        }

        None
    }

    fn split_function_args(s: &str) -> Vec<String> {
        // Split on commas, but only when not inside parentheses
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut depth = 0;

        for ch in s.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    current_arg.push(ch);
                }
                ')' => {
                    depth -= 1;
                    current_arg.push(ch);
                }
                ',' if depth == 0 => {
                    args.push(current_arg.trim().to_string());
                    current_arg.clear();
                }
                _ => {
                    current_arg.push(ch);
                }
            }
        }

        // Don't forget the last argument
        if !current_arg.trim().is_empty() {
            args.push(current_arg.trim().to_string());
        }

        args
    }

    fn try_parse_conditional(s: &str) -> Option<Result<Expression, String>> {
        // Look for IF ... THEN ... ELSE ... pattern
        let s_upper = s.to_uppercase();

        if !s_upper.starts_with("IF ") {
            return None;
        }

        // Find THEN and ELSE at the correct nesting level
        // Use a simpler approach: scan for keywords and track depth
        let (then_pos, else_pos) = Self::find_if_then_else_positions(&s_upper)?;

        // Extract parts (use original case from s, not s_upper)
        // IF is at 0-1, space at 2, condition starts at 3
        // THEN is at then_pos..(then_pos+4), true_expr starts at then_pos+4
        // ELSE is at else_pos..(else_pos+4), false_expr starts at else_pos+4
        let condition_str = s[3..then_pos].trim();
        let true_str = s[then_pos + 4..else_pos].trim();  // Skip "THEN"
        let false_str = s[else_pos + 4..].trim();  // Skip "ELSE"

        // Parse each part recursively
        let condition = match Self::parse(condition_str) {
            Ok(e) => e,
            Err(e) => return Some(Err(e)),
        };
        let true_expr = match Self::parse(true_str) {
            Ok(e) => e,
            Err(e) => return Some(Err(e)),
        };
        let false_expr = match Self::parse(false_str) {
            Ok(e) => e,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(Expression::Conditional {
            condition: Box::new(condition),
            true_expr: Box::new(true_expr),
            false_expr: Box::new(false_expr),
        }))
    }

    fn find_if_then_else_positions(s_upper: &str) -> Option<(usize, usize)> {
        // Find matching THEN and ELSE for the IF at the beginning
        // Track depth: IF increases, THEN/ELSE decreases when inside nested IFs
        let mut depth = 0;
        let mut then_pos = None;
        let mut i = 0;

        while i < s_upper.len() {
            if Self::match_keyword_at(s_upper, i, "IF") {
                depth += 1;
                i += 2;
            } else if Self::match_keyword_at(s_upper, i, "THEN") {
                if depth == 1 && then_pos.is_none() {
                    // This is our THEN
                    then_pos = Some(i);
                } else if depth > 1 {
                    depth -= 1;
                }
                i += 4;
            } else if Self::match_keyword_at(s_upper, i, "ELSE") {
                if depth == 1 && then_pos.is_some() {
                    // This is our ELSE
                    return Some((then_pos.unwrap(), i));
                } else if depth > 1 {
                    depth -= 1;
                }
                i += 4;
            } else {
                i += 1;
            }
        }

        None
    }

    fn find_keyword_at_depth(s_upper: &str, keyword: &str, start_pos: usize) -> Option<usize> {
        // Find a keyword at nesting depth 0 (not inside a nested IF)
        // Track IF nesting depth: each "IF " increases depth, each "THEN" or "ELSE" at same depth closes it
        let mut depth = 0;
        let mut i = start_pos;
        let keyword_trimmed = keyword.trim();

        while i < s_upper.len() {
            // Check for nested IF (increases depth)
            if Self::match_keyword_at(s_upper, i, "IF") {
                depth += 1;
                i += 2;  // Skip "IF"
                continue;
            }

            // Check for the keyword we're looking for
            if Self::match_keyword_at(s_upper, i, keyword_trimmed) {
                if depth == 1 {
                    // Found it at the right depth (depth 1 because we're inside the outermost IF)
                    // Return position of the space before the keyword (for backward compatibility)
                    return if i > 0 && &s_upper[i-1..i] == " " {
                        Some(i - 1)
                    } else {
                        Some(i)
                    };
                } else if depth > 1 {
                    // This keyword closes a nested IF
                    depth -= 1;
                }
                i += keyword_trimmed.len();
                continue;
            }

            // Check for other keywords that affect depth
            if keyword_trimmed == "ELSE" {
                // When looking for ELSE, THEN keywords close nested IFs
                if Self::match_keyword_at(s_upper, i, "THEN") && depth > 1 {
                    depth -= 1;
                    i += 4;
                    continue;
                }
            }

            i += 1;
        }

        None
    }

    fn match_keyword_at(s: &str, pos: usize, keyword: &str) -> bool {
        // Check if keyword appears at position, with word boundaries
        if pos + keyword.len() > s.len() {
            return false;
        }

        // Check if the keyword matches
        if &s[pos..pos + keyword.len()] != keyword {
            return false;
        }

        // Check word boundary before (must be start, space, or non-letter)
        if pos > 0 {
            let prev_char = s.chars().nth(pos - 1).unwrap();
            if prev_char.is_alphabetic() {
                return false;
            }
        }

        // Check word boundary after (must be end, space, or non-letter)
        if pos + keyword.len() < s.len() {
            let next_char = s.chars().nth(pos + keyword.len()).unwrap();
            if next_char.is_alphabetic() {
                return false;
            }
        }

        true
    }

    fn try_parse_comparison(s: &str) -> Option<Expression> {
        // Look for comparison operators: >=, <=, >, <, ==, !=
        // Check two-character operators first
        let two_char_ops = [(">=", Operator::GreaterEqual), ("<=", Operator::LessEqual),
                           ("==", Operator::Equal), ("!=", Operator::NotEqual)];

        for (op_str, op) in &two_char_ops {
            if let Some(pos) = s.find(op_str) {
                let left = s[..pos].trim();
                let right = s[pos + 2..].trim();

                if !left.is_empty() && !right.is_empty() {
                    if let (Ok(left_expr), Ok(right_expr)) = (Self::parse(left), Self::parse(right)) {
                        return Some(Expression::BinaryOp {
                            op: *op,
                            left: Box::new(left_expr),
                            right: Box::new(right_expr),
                        });
                    }
                }
            }
        }

        // Check single-character operators (>, <)
        let mut depth = 0;
        let mut op_pos = None;
        let mut op_char = None;

        for (i, ch) in s.chars().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                '>' | '<' if depth == 0 => {
                    op_pos = Some(i);
                    op_char = Some(ch);
                }
                _ => {}
            }
        }

        if let (Some(pos), Some(op)) = (op_pos, op_char) {
            let left = s[..pos].trim();
            let right = s[pos + 1..].trim();

            if !left.is_empty() && !right.is_empty() {
                if let (Ok(left_expr), Ok(right_expr)) = (Self::parse(left), Self::parse(right)) {
                    let operator = match op {
                        '>' => Operator::GreaterThan,
                        '<' => Operator::LessThan,
                        _ => return None,
                    };

                    return Some(Expression::BinaryOp {
                        op: operator,
                        left: Box::new(left_expr),
                        right: Box::new(right_expr),
                    });
                }
            }
        }

        None
    }

    /// Evaluate expression given a context
    pub fn evaluate(&self, context: &mut EvaluationContext) -> Result<f64, String> {
        match self {
            Expression::Constant(val) => Ok(*val),

            Expression::Variable(name) => {
                context.get_variable(name)
            }

            Expression::SubscriptedVariable { name, subscripts } => {
                context.get_subscripted_variable(name, subscripts)
            }

            Expression::BinaryOp { op, left, right } => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;

                Ok(match op {
                    Operator::Add => left_val + right_val,
                    Operator::Subtract => left_val - right_val,
                    Operator::Multiply => left_val * right_val,
                    Operator::Divide => {
                        if right_val == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        left_val / right_val
                    }
                    Operator::Power => left_val.powf(right_val),
                    // Comparison operators return 1.0 for true, 0.0 for false
                    Operator::GreaterThan => if left_val > right_val { 1.0 } else { 0.0 },
                    Operator::LessThan => if left_val < right_val { 1.0 } else { 0.0 },
                    Operator::GreaterEqual => if left_val >= right_val { 1.0 } else { 0.0 },
                    Operator::LessEqual => if left_val <= right_val { 1.0 } else { 0.0 },
                    Operator::Equal => if (left_val - right_val).abs() < 1e-10 { 1.0 } else { 0.0 },
                    Operator::NotEqual => if (left_val - right_val).abs() >= 1e-10 { 1.0 } else { 0.0 },
                })
            }

            Expression::UnaryOp { op, expr } => {
                let val = expr.evaluate(context)?;
                Ok(match op {
                    UnaryOperator::Negate => -val,
                })
            }

            Expression::FunctionCall { name, args } => {
                Self::evaluate_function(name, args, context)
            }

            Expression::Conditional { condition, true_expr, false_expr } => {
                // Lazy evaluation: only evaluate the branch that's taken
                let cond_val = condition.evaluate(context)?;
                if cond_val > 0.5 {  // Treat > 0.5 as true
                    true_expr.evaluate(context)
                } else {
                    false_expr.evaluate(context)
                }
            }
        }
    }

    fn evaluate_function(name: &str, args: &[Expression], context: &mut EvaluationContext) -> Result<f64, String> {
        let arg_values: Result<Vec<f64>, String> = args
            .iter()
            .map(|arg| arg.evaluate(context))
            .collect();
        let arg_values = arg_values?;

        match name.to_uppercase().as_str() {
            // Variadic functions
            "MIN" => {
                if arg_values.is_empty() {
                    return Err("MIN requires at least 1 argument".to_string());
                }
                Ok(arg_values.iter().copied().fold(f64::INFINITY, f64::min))
            }
            "MAX" => {
                if arg_values.is_empty() {
                    return Err("MAX requires at least 1 argument".to_string());
                }
                Ok(arg_values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            }

            // Single-argument math functions
            "ABS" => {
                if arg_values.len() != 1 {
                    return Err(format!("ABS expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].abs())
            }
            "SQRT" => {
                if arg_values.len() != 1 {
                    return Err(format!("SQRT expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].sqrt())
            }
            "EXP" => {
                if arg_values.len() != 1 {
                    return Err(format!("EXP expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].exp())
            }
            "LN" => {
                if arg_values.len() != 1 {
                    return Err(format!("LN expects 1 argument, got {}", arg_values.len()));
                }
                if arg_values[0] <= 0.0 {
                    return Err("LN requires positive argument".to_string());
                }
                Ok(arg_values[0].ln())
            }
            "LOG" => {
                // LOG is same as LN in most SD tools
                if arg_values.len() != 1 {
                    return Err(format!("LOG expects 1 argument, got {}", arg_values.len()));
                }
                if arg_values[0] <= 0.0 {
                    return Err("LOG requires positive argument".to_string());
                }
                Ok(arg_values[0].ln())
            }
            "LOG10" => {
                if arg_values.len() != 1 {
                    return Err(format!("LOG10 expects 1 argument, got {}", arg_values.len()));
                }
                if arg_values[0] <= 0.0 {
                    return Err("LOG10 requires positive argument".to_string());
                }
                Ok(arg_values[0].log10())
            }

            // Trigonometric functions
            "SIN" => {
                if arg_values.len() != 1 {
                    return Err(format!("SIN expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].sin())
            }
            "COS" => {
                if arg_values.len() != 1 {
                    return Err(format!("COS expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].cos())
            }
            "TAN" => {
                if arg_values.len() != 1 {
                    return Err(format!("TAN expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].tan())
            }
            "ASIN" => {
                if arg_values.len() != 1 {
                    return Err(format!("ASIN expects 1 argument, got {}", arg_values.len()));
                }
                if arg_values[0] < -1.0 || arg_values[0] > 1.0 {
                    return Err("ASIN requires argument in [-1, 1]".to_string());
                }
                Ok(arg_values[0].asin())
            }
            "ACOS" => {
                if arg_values.len() != 1 {
                    return Err(format!("ACOS expects 1 argument, got {}", arg_values.len()));
                }
                if arg_values[0] < -1.0 || arg_values[0] > 1.0 {
                    return Err("ACOS requires argument in [-1, 1]".to_string());
                }
                Ok(arg_values[0].acos())
            }
            "ATAN" => {
                if arg_values.len() != 1 {
                    return Err(format!("ATAN expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].atan())
            }

            // Rounding functions
            "FLOOR" => {
                if arg_values.len() != 1 {
                    return Err(format!("FLOOR expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].floor())
            }
            "CEIL" => {
                if arg_values.len() != 1 {
                    return Err(format!("CEIL expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].ceil())
            }
            "ROUND" => {
                if arg_values.len() != 1 {
                    return Err(format!("ROUND expects 1 argument, got {}", arg_values.len()));
                }
                Ok(arg_values[0].round())
            }

            // Two-argument functions
            "POW" => {
                if arg_values.len() != 2 {
                    return Err(format!("POW expects 2 arguments, got {}", arg_values.len()));
                }
                Ok(arg_values[0].powf(arg_values[1]))
            }
            "MODULO" | "MOD" => {
                if arg_values.len() != 2 {
                    return Err(format!("MODULO expects 2 arguments, got {}", arg_values.len()));
                }
                if arg_values[1] == 0.0 {
                    return Err("MODULO by zero".to_string());
                }
                Ok(arg_values[0] % arg_values[1])
            }

            // System Dynamics specific functions
            "PULSE" => {
                // PULSE(start, width) or PULSE(start, width, repeat_interval)
                if arg_values.len() < 2 || arg_values.len() > 3 {
                    return Err(format!("PULSE expects 2 or 3 arguments, got {}", arg_values.len()));
                }
                let start = arg_values[0];
                let width = arg_values[1];
                let time = context.time;

                if arg_values.len() == 3 {
                    // Repeating pulse
                    let interval = arg_values[2];
                    if interval <= 0.0 {
                        return Err("PULSE interval must be positive".to_string());
                    }
                    if time < start {
                        return Ok(0.0);
                    }
                    let time_since_start = time - start;
                    let phase = time_since_start % interval;
                    if phase < width {
                        Ok(1.0)
                    } else {
                        Ok(0.0)
                    }
                } else {
                    // Single pulse
                    if time >= start && time < start + width {
                        Ok(1.0)
                    } else {
                        Ok(0.0)
                    }
                }
            }
            "STEP" => {
                // STEP(height, step_time)
                if arg_values.len() != 2 {
                    return Err(format!("STEP expects 2 arguments, got {}", arg_values.len()));
                }
                let height = arg_values[0];
                let step_time = arg_values[1];
                let time = context.time;

                if time >= step_time {
                    Ok(height)
                } else {
                    Ok(0.0)
                }
            }
            "RAMP" => {
                // RAMP(slope, start_time) or RAMP(slope, start_time, end_time)
                if arg_values.len() < 2 || arg_values.len() > 3 {
                    return Err(format!("RAMP expects 2 or 3 arguments, got {}", arg_values.len()));
                }
                let slope = arg_values[0];
                let start_time = arg_values[1];
                let time = context.time;

                if time < start_time {
                    return Ok(0.0);
                }

                if arg_values.len() == 3 {
                    // Ramp with end time
                    let end_time = arg_values[2];
                    if time >= end_time {
                        Ok(slope * (end_time - start_time))
                    } else {
                        Ok(slope * (time - start_time))
                    }
                } else {
                    // Unlimited ramp
                    Ok(slope * (time - start_time))
                }
            }

            // Special functions
            "TIME" => {
                if !arg_values.is_empty() {
                    return Err(format!("TIME expects 0 arguments, got {}", arg_values.len()));
                }
                Ok(context.time)
            }

            // Delay functions
            "DELAY1" | "SMOOTH" => {
                // DELAY1(input, delay_time) or DELAY1(input, delay_time, initial)
                if arg_values.len() < 2 || arg_values.len() > 3 {
                    return Err(format!("{} expects 2 or 3 arguments, got {}", name, arg_values.len()));
                }
                let input = arg_values[0];
                let delay_time = arg_values[1];
                let initial = if arg_values.len() == 3 {
                    arg_values[2]
                } else {
                    input
                };

                // Create unique key for this delay
                let key = format!("{}_{}", name, args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join("_"));

                let delay = context.state.delays.get_or_create_exponential(&key, initial, delay_time, 1);
                Ok(delay.get_value())
            }

            "DELAY3" => {
                // DELAY3(input, delay_time) or DELAY3(input, delay_time, initial)
                if arg_values.len() < 2 || arg_values.len() > 3 {
                    return Err(format!("DELAY3 expects 2 or 3 arguments, got {}", arg_values.len()));
                }
                let input = arg_values[0];
                let delay_time = arg_values[1];
                let initial = if arg_values.len() == 3 {
                    arg_values[2]
                } else {
                    input
                };

                let key = format!("DELAY3_{}", args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join("_"));

                let delay = context.state.delays.get_or_create_exponential(&key, initial, delay_time, 3);
                Ok(delay.get_value())
            }

            "DELAYP" => {
                // DELAYP(input, delay_time, initial)
                if arg_values.len() != 3 {
                    return Err(format!("DELAYP expects 3 arguments, got {}", arg_values.len()));
                }
                let input = arg_values[0];
                let delay_time = arg_values[1];
                let initial = arg_values[2];

                let key = format!("DELAYP_{}", args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join("_"));

                let delay = context.state.delays.get_or_create_pipeline(&key, initial, delay_time);
                Ok(delay.get_delayed_value(context.time))
            }

            // Lookup functions
            "LOOKUP" => {
                // LOOKUP(lookup_table_name, x)
                if arg_values.len() != 2 {
                    return Err(format!("LOOKUP expects 2 arguments, got {}", arg_values.len()));
                }

                // First argument should be a constant that represents the lookup table name
                // For now, we'll use a workaround - the table name is embedded in the function call
                // Better: LOOKUP("TableName", x) where first arg is parsed as string
                // For implementation, we expect: LOOKUP(table_index, x) where table_index refers to a parameter

                // This is a simplified implementation - in practice, you'd want to support string arguments
                Err("LOOKUP function requires direct lookup table reference - use WITH_LOOKUP instead".to_string())
            }

            "WITH_LOOKUP" => {
                // WITH_LOOKUP(x, (x1,y1), (x2,y2), ...)
                // This is a simplified inline lookup - user provides data points directly
                // For example: WITH_LOOKUP(TIME, 0,0, 10,100, 20,50)
                // Pairs of values represent (x,y) points

                if arg_values.len() < 3 || arg_values.len() % 2 != 1 {
                    return Err("WITH_LOOKUP expects odd number of arguments: x, x1, y1, x2, y2, ...".to_string());
                }

                let x = arg_values[0];
                let mut points: Vec<(f64, f64)> = Vec::new();

                for i in (1..arg_values.len()).step_by(2) {
                    points.push((arg_values[i], arg_values[i + 1]));
                }

                // Create temporary lookup table
                let table = crate::simulation::LookupTable::new("inline".to_string(), points)?;
                Ok(table.lookup(x))
            }

            // Stochastic functions
            "RANDOM" => {
                // RANDOM() - uniform [0, 1)
                if !arg_values.is_empty() {
                    return Err(format!("RANDOM expects 0 arguments, got {}", arg_values.len()));
                }
                Ok(context.state.stochastic.random())
            }

            "UNIFORM" => {
                // UNIFORM(min, max)
                if arg_values.len() != 2 {
                    return Err(format!("UNIFORM expects 2 arguments, got {}", arg_values.len()));
                }
                Ok(context.state.stochastic.uniform(arg_values[0], arg_values[1]))
            }

            "NORMAL" => {
                // NORMAL(mean, std_dev)
                if arg_values.len() != 2 {
                    return Err(format!("NORMAL expects 2 arguments, got {}", arg_values.len()));
                }
                context.state.stochastic.normal(arg_values[0], arg_values[1])
            }

            "LOGNORMAL" => {
                // LOGNORMAL(mean, std_dev)
                if arg_values.len() != 2 {
                    return Err(format!("LOGNORMAL expects 2 arguments, got {}", arg_values.len()));
                }
                context.state.stochastic.lognormal(arg_values[0], arg_values[1])
            }

            "POISSON" => {
                // POISSON(lambda)
                if arg_values.len() != 1 {
                    return Err(format!("POISSON expects 1 argument, got {}", arg_values.len()));
                }
                context.state.stochastic.poisson(arg_values[0])
            }

            // Agent-Based Modeling functions
            // Note: These are simplified implementations. In practice, you'd want to
            // support string arguments for agent type names. For now, we use parameter references.
            "AGENT_COUNT" => {
                // AGENT_COUNT() - total count of all agents
                // or AGENT_COUNT(type_name) - count of specific type (not yet implemented)
                if arg_values.is_empty() {
                    Ok(context.state.agents.total_agent_count() as f64)
                } else {
                    Err("AGENT_COUNT with type parameter not yet implemented - use AGENT_COUNT()".to_string())
                }
            }

            "AGENT_SUM" => {
                // AGENT_SUM(type_name, attribute_name)
                // Simplified: expects numeric parameters
                Err("AGENT_SUM requires string arguments for type and attribute names".to_string())
            }

            "AGENT_MEAN" => {
                // AGENT_MEAN(type_name, attribute_name)
                Err("AGENT_MEAN requires string arguments for type and attribute names".to_string())
            }

            "AGENT_MAX" => {
                // AGENT_MAX(type_name, attribute_name)
                Err("AGENT_MAX requires string arguments for type and attribute names".to_string())
            }

            "AGENT_MIN" => {
                // AGENT_MIN(type_name, attribute_name)
                Err("AGENT_MIN requires string arguments for type and attribute names".to_string())
            }

            _ => Err(format!("Unknown function: '{}' (length: {})", name, name.len()))
        }
    }
}

/// Context for evaluating expressions
pub struct EvaluationContext<'a> {
    pub model: &'a crate::model::Model,
    pub state: &'a mut crate::simulation::SimulationState,
    pub time: f64,
}

impl<'a> EvaluationContext<'a> {
    pub fn new(model: &'a crate::model::Model, state: &'a mut crate::simulation::SimulationState, time: f64) -> Self {
        Self { model, state, time }
    }

    pub fn get_variable(&self, name: &str) -> Result<f64, String> {
        // Handle special built-in variables
        if name.to_uppercase() == "TIME" {
            return Ok(self.time);
        }

        self.model.get_variable(name, self.state)
    }

    pub fn get_subscripted_variable(
        &self,
        name: &str,
        subscripts: &[crate::model::SubscriptRef],
    ) -> Result<f64, String> {
        // For now, subscripted variables are not fully supported in the basic SimulationState
        // This is a placeholder that will need to be implemented when ArraySimulationState is integrated
        // For basic implementation, we can try to construct the full variable name
        // e.g., "Population[North]" becomes "Population_North"

        if subscripts.is_empty() {
            return self.get_variable(name);
        }

        // Try to construct a flattened name
        let mut full_name = name.to_string();
        for sub in subscripts {
            match sub {
                crate::model::SubscriptRef::Element(elem) => {
                    full_name.push('_');
                    full_name.push_str(elem);
                }
                crate::model::SubscriptRef::Dimension(dim) => {
                    // Sum over all elements in dimension (not implemented yet)
                    return Err(format!(
                        "Dimension subscript '{}' not yet supported - use specific elements",
                        dim
                    ));
                }
                crate::model::SubscriptRef::Wildcard => {
                    return Err("Wildcard subscript not yet supported".to_string());
                }
            }
        }

        // Try to find the flattened variable
        self.get_variable(&full_name)
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Constant(val) => write!(f, "{}", val),
            Expression::Variable(name) => write!(f, "{}", name),
            Expression::SubscriptedVariable { name, subscripts } => {
                write!(f, "{}[", name)?;
                for (i, sub) in subscripts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match sub {
                        crate::model::SubscriptRef::Element(e) => write!(f, "{}", e)?,
                        crate::model::SubscriptRef::Dimension(d) => write!(f, "{}", d)?,
                        crate::model::SubscriptRef::Wildcard => write!(f, "*")?,
                    }
                }
                write!(f, "]")
            }
            Expression::BinaryOp { op, left, right } => {
                let op_str = match op {
                    Operator::Add => "+",
                    Operator::Subtract => "-",
                    Operator::Multiply => "*",
                    Operator::Divide => "/",
                    Operator::Power => "^",
                    Operator::GreaterThan => ">",
                    Operator::LessThan => "<",
                    Operator::GreaterEqual => ">=",
                    Operator::LessEqual => "<=",
                    Operator::Equal => "==",
                    Operator::NotEqual => "!=",
                };
                write!(f, "({} {} {})", left, op_str, right)
            }
            Expression::UnaryOp { op, expr } => {
                match op {
                    UnaryOperator::Negate => write!(f, "(-{})", expr),
                }
            }
            Expression::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expression::Conditional { condition, true_expr, false_expr } => {
                write!(f, "IF {} THEN {} ELSE {}", condition, true_expr, false_expr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constant() {
        let expr = Expression::parse("42").unwrap();
        assert!(matches!(expr, Expression::Constant(42.0)));
    }

    #[test]
    fn test_parse_variable() {
        let expr = Expression::parse("Population").unwrap();
        assert!(matches!(expr, Expression::Variable(ref name) if name == "Population"));
    }

    #[test]
    fn test_parse_simple_conditional() {
        let expr = Expression::parse("IF x > 0 THEN 1 ELSE 0").unwrap();
        assert!(matches!(expr, Expression::Conditional { .. }));
    }

    #[test]
    fn test_parse_nested_conditional() {
        let expr = Expression::parse("IF x > 2 THEN 1.0 ELSE IF x > 1 THEN 0.5 ELSE 0.1").unwrap();
        assert!(matches!(expr, Expression::Conditional { .. }));
    }

    #[test]
    fn test_parse_addition() {
        let expr = Expression::parse("1 + 2").unwrap();
        assert!(matches!(expr, Expression::BinaryOp { op: Operator::Add, .. }));
    }

    #[test]
    fn test_parse_multiplication() {
        let expr = Expression::parse("3 * 4").unwrap();
        assert!(matches!(expr, Expression::BinaryOp { op: Operator::Multiply, .. }));
    }
}
