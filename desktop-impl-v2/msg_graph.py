import re
from typing import Dict, Tuple, Set
import sys

args = sys.argv
if len(args) != 3:
    print("invalid argument")
    exit(1)

input_file, output_file = args[1], args[2]

first_pattern = r"from node: (\w+), to node: (\w+), msg: (.+)"
second_pattern = r"handle message result: (\w+)"

graph: Dict[Tuple[str, str], Set[str]] = {}
first_node = 'Scheduler'

with open(input_file) as f:
    for line in f:
        if 'handle message from' not in line:
            continue
        matches = re.search(first_pattern, line)
        from_node, to_node, msg = matches.group(1), matches.group(2), matches.group(3)
        
        matches = re.search(second_pattern, next(f))
        result = matches.group(1)
        if result == 'Discard':
            continue
        
        k = (from_node, to_node)
        if k not in graph.keys():
            graph[k] = set()
        graph[k].add(msg)

with open(output_file, mode='w') as f:
    f.write(f'@startuml 消息图\n[*] --> {first_node}\n')
    for (edge, msgs) in graph.items():
        for msg in msgs:
            f.write(f'{edge[0]} --> {edge[1]}: {msg}\n')
    f.write('@enduml')