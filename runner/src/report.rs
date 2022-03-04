use std::collections::HashMap;
use std::io::Write;
use junit_report::{Duration, Report, ReportError, TestCase, TestSuite};
use crate::TestResult;

pub fn xml_dump_result(results: Vec<TestResult>, mut write: impl Write) -> Result<(), ReportError> {
    let mut sorted = HashMap::new();
    for result in &results {
        let entry = sorted.entry(&result.case.suite_name).or_insert_with(|| vec![]);
        entry.push(result.clone());
    }
    let mut report = Report::new();
    for (suite_name, tests) in sorted {
        let mut suite = TestSuite::new(suite_name);
        for test in tests {
            let test_case = match test.error {
                None => {
                    TestCase::success(&test.case.test_name, Duration::microseconds(0))
                }
                Some(err) => {
                    let msg = format!("Assert failed at {}:{}", err.file_name, err.lineno);
                    TestCase::error(&test.case.test_name, Duration::microseconds(0), "Assert Failed", &msg)
                }
            };
            suite.add_testcase(test_case)
        }
        report.add_testsuite(suite);
    }
    report.write_xml(&mut write)
}
