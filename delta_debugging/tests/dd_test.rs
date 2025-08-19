use std::collections::HashMap;

use delta_debugging::{ddmin, Data, TestResult};

#[test]
fn test_ddmin() {
    let tcs = [
        (
            String::from("12345678"),
            String::from("178"),
            String::from("178"),
        ),
        (
            String::from("12345678"),
            String::from("1<78"),
            String::from("12345678"),
        ),
        (
            String::from("int a = 1; int b = 2; assert(a == b);"),
            String::from("assert(a == b);"),
            String::from("assert(a == b);"),
        ),
        (
            String::from("This is a test string with a problematic $ character inside."),
            String::from("$"),
            String::from("$"),
        ),
        (
            String::from("function calculate_sum(a, b) { return a + b; print(\"calc\" }"),
            String::from("print(\"calc"),
            String::from("print(\"calc"),
        ),
        (
            String::from(
                "{'data': [1, 2, 3, 'value', {'key': 'nested'}, 'extra', {'bug': 'missing_brace']}",
            ),
            String::from("missing_brace'"),
            String::from("'missing_brace"),
        ),
    ];
    for tc in tcs {
        let (input, fail_inducing_input, expected) = (tc.0, tc.1, tc.2);
        test(input, fail_inducing_input, expected.into_bytes());
    }
}

fn test(input: String, fail_inducing_input: String, expected: Data) {
    let oracle = make_oracle(fail_inducing_input.into_bytes());
    let minimized = ddmin(&input.into_bytes(), oracle);
    println!(
        " {:?} == {:?}",
        byte_to_str(&minimized),
        byte_to_str(&expected)
    );
    assert_eq!(minimized, expected);
}

fn byte_to_str(data: &Data) -> String {
    String::from_utf8(data.to_vec()).unwrap()
}

fn make_oracle(fail_inducing_input: Data) -> Box<dyn Fn(&Data) -> TestResult> {
    Box::new(move |data: &Data| {
        let mut fail_inducing_cnt = HashMap::new();
        for v in &fail_inducing_input {
            *fail_inducing_cnt.entry(v).or_insert(0) += 1;
        }
        let mut input_cnt = HashMap::new();
        for v in data {
            *input_cnt.entry(v).or_insert(0) += 1;
        }
        for (key, &needed) in &fail_inducing_cnt {
            let available = input_cnt.get(key).copied().unwrap_or(0);
            if available < needed {
                println!("pass: {:?}", byte_to_str(data));
                return TestResult::Pass;
            }
        }
        println!("failured: {:?}", byte_to_str(data));
        TestResult::Fail
    })
}
