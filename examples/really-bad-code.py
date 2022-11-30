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
