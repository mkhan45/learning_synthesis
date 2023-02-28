# learning_synthesis

```
    let res = top_down_vsa(&vec![
        (
            Lit::StringConst("I have 17 cookies".to_string()),
            Lit::StringConst("17".to_string()),
        ),
        (
            Lit::StringConst("Give me at least 3 cookies".to_string()),
            Lit::StringConst("3".to_string()),
        ),
        (
            Lit::StringConst("This number is 489".to_string()),
            Lit::StringConst("489".to_string()),
        ),
    ]);
    println!("{}, size = {}", res, res.size()); // X[X.find('\d')..(' ' <> X).find(('\d' <> '\b'))]
    assert_eq!(
        res.eval(&Lit::StringConst(
            "A string with the number 54234564 in the middle".to_string()
        )),
        Lit::StringConst("54234564".to_string())
    );
 ```

Roughly following these to learn synthesis
- <https://github.com/nadia-polikarpova/cse291-program-synthesis>
- <https://people.csail.mit.edu/asolar/SynthesisCourse/index.htm>
- <https://www.microsoft.com/en-us/research/wp-content/uploads/2016/12/oopsla15-pbe.pdf>

Goals:
- [X] Bottom Up
- [X] Top Down
- [X] Figure out VSAs?
    - kind of but I dont have a good algorithm to use them
    - [X] Top down with inverse semantics and a VSA but it's not very good
- [X] Duet approach
    - https://dl.acm.org/doi/10.1145/3434335
    - probably lacking some things but the overall approach is the same
    - src/main/top_down_vsa.rs
- Skolemization
    - https://www.microsoft.com/en-us/research/wp-content/uploads/2016/12/oopsla15-pbe.pdf
    - parallelizing somehow
- [ ] Middle Out?
  - https://dl.acm.org/doi/pdf/10.1145/3571226
