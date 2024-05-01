# learning_synthesis

Try it out at <https://mkhan45.github.io/learning_synthesis/>

check out <https://mkhan45.github.io/interactive_vsa/>

```
// generates: (X[X.find('\d', 0)..(1 + X.find(('\d' <> '\b'), 0))]), size = 13
test_str!(
    test_duet_numbers,
    "I have 17 cookies" => "17",
    "Give me at least 3 cookies" => "3",
    "This number is 489" => "489";

    "A string with the number 54234564 in the middle" => "54234564",
    "36" => "36",
    "Number at the end 74" => "74"
);

More examples in tests.rs
```

Roughly following these to write a string transformation synthesizer
- <https://github.com/nadia-polikarpova/cse291-program-synthesis> and <https://people.csail.mit.edu/asolar/SynthesisCourse/index.htm>
    - To learn the general concepts
- <https://www.microsoft.com/en-us/research/wp-content/uploads/2016/12/oopsla15-pbe.pdf>
    - To implement a VSA
- <https://dl.acm.org/doi/10.1145/3434335>
    - For the algorithm

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
- [ ] Skolemization?
    - https://www.microsoft.com/en-us/research/wp-content/uploads/2016/12/oopsla15-pbe.pdf
    - parallelizing somehow
    - might not because wasm threads are hard
- [ ] Conditionals?
    - and other extensions to the language
- [ ] Middle Out?
  - https://dl.acm.org/doi/pdf/10.1145/3571226
