import json
import sys
from pprint import pprint

json_file = sys.argv[1]

with open(json_file) as f:
  data = json.load(f)
  print(data)

  targets_and_dependencies = {}
  for target, payload in data.items():
    targets_and_dependencies[target] = payload["dependencies"]

  print(targets_and_dependencies)




pprint(data)
