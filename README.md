# learning_synthesis

```
    let examples = vec![
        (
            StringExpr::Lit("First Last".to_owned()),
            StringExpr::Lit("F L".to_owned()),
        ),
        (
            StringExpr::Lit("Abc Def".to_owned()),
            StringExpr::Lit("A D".to_owned()),
        ),
    ];
    let prog = enumerative::search::bottom_up(&examples);
    dbg!(prog);
```

```
[src/main.rs:18] prog = Some(
    (((X[0..1]) <> (X[(X.find(' '))..-1]))[0..3]),
)
```

Roughly following these to learn synthesis
- <https://github.com/nadia-polikarpova/cse291-program-synthesis>
- <https://people.csail.mit.edu/asolar/SynthesisCourse/index.htm>

Goals:
- [X] Bottom Up
- [X] Top Down
    - [ ] Better pruning
- Figure out VSAs?
- [ ] Duet approach
    - https://dl.acm.org/doi/10.1145/3434335
- [ ] Middle Out?
  - https://dl.acm.org/doi/pdf/10.1145/3571226
