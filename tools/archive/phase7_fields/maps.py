# The typed pieces' maps, from `$S/maps.list` (one per line: map file, static, path), in the
# order the converters try them.
import json


def load(S):
    rows = [l.split('\t') for l in open(S + '/maps.list').read().splitlines() if l.strip()]
    return [(json.load(open(S + '/' + f)), st, use) for f, st, use in rows]
