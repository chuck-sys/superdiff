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
- A progress bar (Amazing!)

## Limitations

- Can't check for duplicate code across multiple files (only works on 1 file at a time)
- Not instantaneous

## Usage

Say you have some file `examples/really-bad-code.py` that you want to inspect. You have a feeling that it
might be bad, so you use the tool.

```console
$ superdiff -vv -b 4 examples/really-bad-code.py
File: "examples/really-bad-code.py" (38 lines)
Verbosity (-v): 2
Comparison threshold (-t): 0 (Strict equality)
Minimum length of first line before block consideration (-l): 1
Minimum length of block before consideration (-b): 4
Line 5 length 5: [11]
        self.alpha = 12
        self.beta = 14
        self.gamma = 16
        self.is_bad = True

1 unique blocks with duplicates found, 2 total duplicates
```

Wow! That's pretty nice that you found that! But maybe there are places in the file that aren't exact
copies, but are similar enough.

```console
$ superdiff -vv -b 4 -t 5 examples/really-bad-code.py
File: "examples/really-bad-code.py" (38 lines)
Verbosity (-v): 2
Comparison threshold (-t): 5 (Levenshtein distance)
Minimum length of first line before block consideration (-l): 1
Minimum length of block before consideration (-b): 4
Line 5 length 5: [11]
        self.alpha = 12
        self.beta = 14
        self.gamma = 16
        self.is_bad = True

Line 16 length 10: [26]
    def do_something(self):
        d = {}

        import random
        for i in range(20):
            if i % 3 == 0: continue
            d[i] = random.randrange(1, 1001)
            d[i ** 2] = d[i] ** 2
            d[d[i]] = i

2 unique blocks with duplicates found, 4 total duplicates
```

Huh, apparently there is a duplicate function that are pretty similar!
