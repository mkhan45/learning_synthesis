<script lang="ts">
  /* import * as _ from "lodash" */
  import type { IO } from "./util.ts";
  import { builtin_examples } from "./util.ts";

  let examples: Array<IO> = [{in: "", out: null}];
  let program: string = "None";

  function removeExample() {
    examples = examples.slice(0, -1);
  }

  function pushExample() {
    examples = [...examples, {in: "", out: null}];
  }

  function setExamples(exs: Array<IO>) {
    program = "None";
    examples = exs;
  }

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
        }, 15_000);
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

  setExamples(builtin_examples[1].io);
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

    <h2>Info</h2>
    <p>
        The synthesizer is written in Rust and compiled to WebAssembly, and it
        runs in a Web Worker with a timeout of 15 seconds. Because of that,
        some of the examples might work on your laptop but not on your phone.
        All of these examples work on my laptop, but numbers and remove between
        take a few seconds.
    </p>
    <p>
        For more info, check <a href="https://github.com/mkhan45/learning_synthesis">the GitHub repo</a>.
        This site uses the algorithm in <code>src/enumerative/top_down.rs</code>.
    </p>
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

    p {
        background: #1a1a1a;
        padding: 1em;
        margin: auto;
        text-align: left;
        max-width: 60ch;
        font-size: 1.1em;
    }

    code {
        font-size: 1.175em;
    }
</style>
