<script lang="ts">
  import * as _ from "lodash"

  type IO = {
    in: string,
    out: string | null,
  };

  let examples: Array<IO> = [{in: "", out: null}];
  let program: string = "None";

  function removeExample() {
    examples = examples.slice(0, -1);
  }

  function pushExample() {
    examples = [...examples, {in: "", out: null}];
  }

  function setExamples(exs: Array<IO>) {
    examples = exs;
  }

  const builtin_examples: Array<{name: string, io: Array<IO>}> = [
      {
        name: "URLs",
        io: [
            {in: "http://www.example.com", out: "example"},
            {in: "https://www.apple.com/uk/mac", out: "apple"},
            {in: "https://www.google.com", out: null},
            {in: "www.mikail-khan.com", out: null},
        ]
      },
      {
        name: "Abbreviations",
        io: [
            {in: "First Last", out: "F.L."},
            {in: "Hi Aref", out: null},
            {in: "Bed Time", out: null},
            {in: "Another Name", out: null},
            {in: "Bhavesh Pareek", out: null},
            {in: "Saad Sharief", out: null},
        ]
      },
      {
        name: "Numbers",
        io: [
            {in: "I have 17 cookies", out: "17"},
            {in: "Give me at least 3 cookies", out: "3"},
            {in: "This number is 489", out: "489"},
            {in: "A string with the number 54234564 in the middle", out: null},
            {in: "36", out: null},
            {in: "Another 456432 string", out: ""},
        ]
      },
  ];

  async function run() {
    program = "Synthesizing...";
    let inps = examples.filter(e => e.out != null && e.out != "").map(e => e.in);
    let outs = examples.filter(e => e.out != null && e.out != "").map(e => e.out);
    let tests = examples.filter(e => e.in != null && e.in != "").map(e => e.in);

    console.log(inps, outs);

    const worker = new Worker("./pkg/synthesizer.js");
    worker.postMessage({inps, outs, tests});
    let finished = false;
    const result = await Promise.race([
      new Promise((resolve) =>
        worker.addEventListener(
          "message",
          ({data}) => {
            finished = true;
            resolve(data);
          },
          {
            once: true,
          }
        )
      ),
      new Promise((resolve) => {
        setTimeout(() => {
          if (finished) return;
          console.log("timeout");
          resolve({error: true});
        }, 10_000);
      }),
    ]);
    worker.terminate();

    if (!result.error) {
      program = result.get("program");
      let results = result.get("test_results");
      console.log(results);
      examples = examples.map((e, i) => {
        return {
          in: e.in,
          out: results[i],
        };
      });

      /*
      document.querySelector("#program").innerHTML = program;
      for (let i = 0; i < results.length; i += 1) {
        let row = tab.rows[i + 1];
        row.cells[1].childNodes[0].value = results[i];
      }
      */
    } else {
      console.log("error :(");
      program = "error :(";
      // document.querySelector("#program").innerHTML = "An error occured :(";
    }
  }
</script>

<main>
    <h1 style="margin-bottom: 0.1em">FlashFill--</h1>
    <div style="margin-bottom: 1em"><a href="https://github.com/mkhan45/learning_synthesis">GitHub</a></div>

    <table>
        <tr>
            <th>Input</th>
            <th>Output</th>
        </tr>
        {#each examples as {in: inp, out}}
            <tr>
                <td><input type="text" bind:value={inp} /></td>
                <td><input type="text" bind:value={out} /></td>
                <tr>
        {/each}
    </table>
    <button on:click={removeExample}>-</button>
    <button on:click={pushExample}>+</button>
    <pre class="program-box">Program: {program}</pre>
    <button on:click={run}>Run</button>

    <h2>Examples</h2>
    {#each builtin_examples as {name, io}}
        <button on:click={() => setExamples(io)}>{name}</button>
    {/each}
</main>

<style>
    table {
        width: 100%;
    }

    input {
        width: 100%;
        text-align: center;
    }

    button {
        width: 100%;
    }

    .program-box {
        margin-top: 1em;
        margin-bottom: 0.15em;
    }

    pre {
        font-size: 1.5em;
        min-width: 80ch;
    }
</style>
