# platina
[![Build Status](https://travis-ci.com/Luminarys/platina.svg?branch=master)](https://travis-ci.com/Luminarys/platina)

Simple parameterized golden testing.

Golden files are a powerful way of testing: they make it convenient to separate tests from test data. However since most golden test libraries treat an individual test file as a single case it can lead to some inconveniences. This library provides a way to parameterize golden testing with a simple string based API.

For example, suppose we have a library for calculating levenshtein distances between strings. Normally test would be written like this:
```rust
#[test]
fn test1() {
  assert_eq!(lev_dist("a", "b"), 1);
}

#[test]
fn test2() {
  assert_eq!(lev_dist("c", "d"), 1);
}
```

While this isn't so bad, most real life test scenarios require much more than one line of setup. It can sometimes feel very frustrating to repeat the same lines of code again and again in tests. Even worse, what happens when you discover a bug in your code and have to now update all your test answers by hand (when it feels like this could all be automated). This is where parameterized golden tests come in.

We could represent our tests using platina with a data file like this:
```
[case1]
[input1]
a
----------
[input2]
b
----------
[output]
1
----------
==========

[case1]
[input1]
c
----------
[input2]
d
----------
[output]
1
----------
==========
```
Then we can write our tests like this:
```rust
struct LevTester;

impl platina::Testable for LevTester {
    fn run_testcase(&mut self, case: &mut TestCase) {
       let input1 = case.get_param("input1").unwrap();
       let input2 = case.get_param("input2").unwrap();
       case.compare_and_update_param("output",
                                     format!("{}", lev_dist(input1, input2)));
    }
}

#[test]
fn test_diff() {
  let mut t = LevTester;
  let mut f = platina::TestFile("test.txt").run_tests(&mut t).unwrap();
}


#[test]
#[ignore]
fn test_update() {
  let mut t = LevTester;
  let mut f = platina::TestFile("test.txt").run_tests_and_update(&mut t).unwrap();
}

```
Now to update your tests all you have to do is run ```cargo test -- --ignored```

This scenario might feel like a lot of overhead for doing something simple, but when you start needing complex inputs, flags, etc. to be passed into your test setup and all of a sudden need to tweak your output, this type of testing is invaluable.
