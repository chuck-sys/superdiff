# superdiff

Are you working to eliminate similar/duplicate code from your files? Do you have a suspicion that chunks
of code are copy-pasted, but are slightly different s.t. normal `diff` methods don't work? Are you tired
of visually going through and inspecting your code for repeating chunks?

If so, this might be the tool for you!

## Features

- Finds duplicate code slices
- Finds similar-enough code slices
- `JSON` reporting for `jq` integeration
- Fast enough (00:03:39 for a 17k LOC with block size 10 and Levenshtein threshold 10)
- Can check for duplicate code across multiple files
- Vim integration!

## Limitations

- Not instantaneous for large files
- Single-threaded

## Short examples

```console
$ superdiff -l 1 -b 7 examples/really-bad-code.py
=== MATCH ===
File: examples/really-bad-code.py
Lines: [5, 11]
Size: 5
$ find src vim-superdiff -type f | superdiff -l 1 -b 7
=== MATCH ===
File: src/types.rs
Lines: [87, 185]
Size: 7

=== MATCH ===
File: vim-superdiff/autoload/superdiff.vim
Lines: [138, 154]
Size: 7

=== MATCH ===
File: vim-superdiff/autoload/superdiff.vim
Lines: [43, 86]
Size: 7
```

## Usage

Say you have some file `examples/really-bad-code.py` that you want to inspect.

<details>
    <summary><code>examples/really-bad-code.py</code></summary>

```python
#!/usr/bin/env python

class SomeClass:
    def __init__(self):
        self.alpha = 12
        self.beta = 14
        self.gamma = 16
        self.is_bad = True

    def reset(self):
        self.alpha = 12
        self.beta = 14
        self.gamma = 16
        self.is_bad = True

    def do_something(self):
        d = {}

        import random
        for i in range(20):
            if i % 3 == 0: continue
            d[i] = random.randrange(1, 1001)
            d[i ** 2] = d[i] ** 2
            d[d[i]] = i

    def do_something_else(self):
        d = {}

        import random
        for i in range(21):
            if i % 3 == 1: continue
            d[i] = random.randrange(1, 1001)
            d[i ** 2] = d[i]
            d[d[i]] = i

inst = SomeClass()
inst.reset()
```
</details>

You have a feeling that it might be bad, so you use the tool.

```console
$ superdiff -b 4 examples/really-bad-code.py
1 file(s) ["examples/really-bad-code.py"]
Verbosity (-v): true
Comparison threshold (-t): 0 (Strict equality)
Minimum length of first line before block consideration (-l): 1
Minimum length of block before consideration (-b): 4
Now comparing "examples/really-bad-code.py" (   37/   38)...done 1 out of 1
=== MATCH ===
File: "examples/really-bad-code.py"
Lines: [4, 10]
Size: 5

A total of 1 unique match(es) were found in the 1 file(s).
```

Wow! That's pretty nice that you found that! But maybe there are places in the file that aren't exact
copies, but are similar enough.

```console
$ superdiff -b 4 -t 5 examples/really-bad-code.py
1 file(s) ["examples/really-bad-code.py"]
Verbosity (-v): true
Comparison threshold (-t): 5 (Levenshtein distance)
Minimum length of first line before block consideration (-l): 1
Minimum length of block before consideration (-b): 4
Now comparing "examples/really-bad-code.py" (   37/   38)...done 1 out of 1
=== MATCH ===
File: "examples/really-bad-code.py"
Lines: [15, 25]
Size: 10

=== MATCH ===
File: "examples/really-bad-code.py"
Lines: [4, 10]
Size: 5

A total of 2 unique match(es) were found in the 1 file(s).
```

Huh, apparently there is a duplicate function that are pretty similar! And now (assuming that the output
of the function is pretty long and not laughably short), you want to know if line 30 is involved in
duplicate code, so you do the following:

```console
$ superdiff --reporting-mode json -b 5 -t 5 examples/really-bad-code.py > output.json
$ cat output.json | jq
{
  "version": "2.0.2",
  "files": {
    "examples/really-bad-code.py": {
      "count_blocks": 4
    }
  },
  "matches": [
    {
      "files": {
        "examples/really-bad-code.py": {
          "count_blocks": 2
        }
      },
      "blocks": {
        "examples/really-bad-code.py": [
          {
            "starting_line": 5,
            "block_length": 5
          },
          {
            "starting_line": 11,
            "block_length": 5
          }
        ]
      }
    },
    {
      "files": {
        "examples/really-bad-code.py": {
          "count_blocks": 2
        }
      },
      "blocks": {
        "examples/really-bad-code.py": [
          {
            "starting_line": 16,
            "block_length": 10
          },
          {
            "starting_line": 26,
            "block_length": 10
          }
        ]
      }
    }
  ]
}
$ cat output.json | jq '.matches | map(select((.blocks."examples/really-bad-code.py" | any(.starting_line <= 30 and .starting_line + .block_length >= 30))))'
[
  {
    "files": {
      "examples/really-bad-code.py": {
        "count_blocks": 2
      }
    },
    "blocks": {
      "examples/really-bad-code.py": [
        {
          "starting_line": 16,
          "block_length": 10
        },
        {
          "starting_line": 26,
          "block_length": 10
        }
      ]
    }
  }
]
```

**Note:** If anyone finds a better way of making the `jq` query, please make a pull request and/or let me
know.

## Vim integration

It's kind of work-in-progress at the moment, but here's what we have:

- Load JSON with `:SDLoad` (make sure you are in the same directory you ran `superdiff`)
- Open a file to edit
- Run `:SDLocal` to highlight matching blocks of code
- Run `:SDQuery` on a matching block to find other blocks of similar code

Check the vimdocs for more options and commands.

[![asciicast](https://asciinema.org/a/548069.svg)](https://asciinema.org/a/548069)

## Benchmark

These numbers are here to give a ballpark estimate of how long something would take given some files. It
is not scientific. I simply appended `time` to the beginning and copied the `real` time.

The data used for this benchmark can be found in `scripts/populate-data.sh`.

Version | Test name | Params | Time
---|---|---|---
2.0.3 | TerrariaClone | `-b 5 -t 5 json` | 40.284s
2.0.3 | TerrariaClone | `-b 5 json` | 0.489s

## In the works

None at the moment.
