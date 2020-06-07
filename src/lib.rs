//! platina is a simple parameterized golden file testing library.
//!
//! # Usage
//!
//! First, create a test by implementing [`Testable`] for a struct.
//! Then, create a new [`TestFile`], with the path to a platina test file.
//! Finally, run your tests.
//!
//! [`Testable`]: ../platina/trait.Testable.html
//! [`TestFile`]: ../platina/struct.TestFile.html
//!
//! ## Examples
//!
//! An example can found in the in the `README.md` [on
//! GitHub].
//!
//! [on GitHub]: https://github.com/luminarys/platina/blob/master/README.md
//!
//! ## Guide
//!
//! A getting started guide is available in the

use std::io::{self, BufReader, BufRead, BufWriter, Write};
use std::fs::File;
use std::collections::HashMap;

/// Testable describes something which can be tested via platina.
pub trait Testable {
    fn run_testcase(&mut self, case: &mut TestCase);
}

/// TestFile represents a plaintext file in platina's expected format
#[derive(Clone, Debug)]
pub struct TestFile {
    file: String,
}

/// TestCase represents one logical case for a test file in platina.
#[derive(Clone, Debug)]
pub struct TestCase {
    name: String,
    params: HashMap<String, String>,
    order: Vec<String>,
    diffs: Vec<Diff>,
}

#[derive(Clone, Debug)]
struct Diff {
    param: String,
    expected: String,
    actual: String,
}

const CASE_SEP: &'static str =  "===========";
const PARAM_SEP: &'static str = "-----------";

impl TestFile {
    /// Construct a new TestFile from a path to a platina text file.
    pub fn new(path: &str) -> TestFile {
        TestFile {
            file: path.to_owned()
        }
    }

    /// Runs tests in this file using the provided tester.
    pub fn run_tests<T: Testable>(&mut self, tester: &mut T) -> io::Result<()> {
        self.run_test_(tester, false)
    }

    /// Runs tests in this file using the provided tester, updating the test file
    /// with the expected results.
    pub fn run_tests_and_update<T: Testable>(&mut self, tester: &mut T) -> io::Result<()> {
        self.run_test_(tester, true)
    }

    fn run_test_<T: Testable>(&mut self, tester: &mut T, update: bool) -> io::Result<()> {
        let mut reader = BufReader::new(File::open(&self.file)?);
        let mut cases = Vec::new();
        while let Some(case) = TestCase::new(&mut reader)? {
            cases.push(case);
        }
        drop(reader);
        for case in &mut cases {
            tester.run_testcase(case);
        }
        let mut failures = String::new();
        for case in &cases {
            if !case.diffs.is_empty() {
                failures.push_str(format!("CASE FAILED: {}\n", case.name).as_str());
            }
            for diff in &case.diffs {
                failures.push_str(format!("PARAM MISMATCH: {}\nexpected: {}\nactual: {}\n", diff.param, diff.actual, diff.expected).as_str());
            }
        }
        if update {
            let mut writer = BufWriter::new(File::create(&self.file)?);
            for case in &cases {
                case.write(&mut writer)?;
            }
        }
        assert!(failures == "", "\nFAILURES:\n{}", failures);
        Ok(())
    }
}

impl TestCase {
    fn new(reader: &mut BufReader<File>) -> io::Result<Option<TestCase>> {
        let mut case = TestCase {
            name: String::new(),
            params: HashMap::new(),
            order: Vec::new(),
            diffs: Vec::new(),
        };
        let mut line = String::new();
        while reader.read_line(&mut line)? != 0 {
            let trimmed = line.trim();
            if line.trim() != "" {
                assert!(trimmed.starts_with("[") &&
                    trimmed.ends_with("]"),
                        "Case must be in form [case], found {}", trimmed);
                assert!(trimmed.len() > 2, "Case must have name");
                case.name = trimmed[1..trimmed.len()-1].to_owned();
                line.clear();
                break;
            }
            line.clear();
        }
        if case.name == "" {
            return Ok(None)
        }

        let mut cur_param = None;
        while reader.read_line(&mut line)? != 0 {
            let trimmed = line.trim();
            if let Some(param) = cur_param.as_ref().clone() {
                if trimmed.starts_with(PARAM_SEP) {
                    // Remove the final newline
                    case.params.get_mut(param)
                        .unwrap().pop();
                    cur_param = None;
                } else {
                    case.params.get_mut(param)
                        .unwrap().push_str(line.as_str());
                }
            }  else if trimmed.starts_with(CASE_SEP) {
                return Ok(Some(case));
            }  else if trimmed != "" {
                assert!(trimmed.starts_with("[") &&
                    trimmed.ends_with("]"),
                        "Parameter must be in form [param], found {}", trimmed);
                let name = &trimmed[1..trimmed.len()-1];
                cur_param = Some(name.to_owned());
                case.params.insert(name.to_owned(), String::new());
                case.order.push(name.to_owned());
            }
            line.clear();
        }
        panic!("EOF before case could be parsed!");
    }

    fn write(&self, writer: &mut BufWriter<File>) -> io::Result<()> {
        writer.write(format!("[{}]\n", self.name).as_bytes())?;
        for param in &self.order {
            let val = self.params.get(param).unwrap();
            writer.write(format!("[{}]\n", param).as_bytes())?;
            writer.write(format!("{}\n", val).as_bytes())?;
            writer.write(format!("{}\n", PARAM_SEP).as_bytes())?;
        }
        writer.write(format!("{}\n\n", CASE_SEP).as_bytes())?;
        Ok(())
    }

    /// Returns a param's value if it exists
    pub fn get_param(&self, param: &str) -> Option<String> {
        self.params.get(param).map(String::from)
    }

    /// Updates a param with the expected value. If there is a mismatch then the
    /// TestFile will produce a failure at the end of the test.
    pub fn compare_and_update_param(&mut self, param: &str, expected: &str) {
        let actual = self.params.insert(param.to_owned(), expected.to_owned()).unwrap_or(
            "".to_owned()
            );
        if actual != expected {
            self.diffs.push(Diff {
                param: param.to_owned(),
                expected: expected.to_owned(),
                actual,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleTester;

    impl Testable for SimpleTester {
        fn run_testcase(&mut self, case: &mut TestCase) {
            let input = case.get_param("input").unwrap();
            case.compare_and_update_param("expected_output",
                                          &input.replace("test case", "result")
                                          );
        }
    }

    #[test]
    fn test_diff() {
        let mut t = SimpleTester {};
        let mut f = TestFile::new("test.txt");
        let res = f.run_tests(&mut t);
        assert_eq!(res.as_ref().ok(), Some(&()), "Could not run tests: {:?}", res);
    }

    #[ignore]
    #[test]
    fn test_update() {
        let mut t = SimpleTester {};
        let mut f = TestFile::new("test.txt");
        let res = f.run_tests_and_update(&mut t);
        assert_eq!(res.as_ref().ok(), Some(&()), "Could not run tests: {:?}", res);
    }
}
