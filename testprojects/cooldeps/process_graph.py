import json
import subprocess
import sys
from pprint import pprint
from graph_traversal import Graph

path_to_buildozer = "pants-support/buildifier/bin/buildozer"
add_action = "add dependencies "
remove_action = "remove dependencies "
strict_deps_action = "print strict deps"

# read the dep-usage graph
universe = sys.argv[1]

# read the direct dependees
direct_dependees = sys.argv[2]

# target to clean
target = sys.argv[3]

def build_command(action, dependency):
  # buildozer doesn't recognize a pattern of type "a/target/T:T"
  # in which case the last part needs to be removed
  parts = dependency.split(":")
  if parts[0].endswith(parts[1]):
    dependency = parts[0]
  return ' "' + action + dependency + '" '

def call_buildozer(action, dependency, target):
  cmd = build_command(action, dependency)
  full_cmd = " ".join([path_to_buildozer, cmd, target])
  subprocess.run(full_cmd, shell=True)

def add_dependency(dependency, target):
  print("ADD: toAdd = ", toAdd, "to target = ", target, "\n")
  call_buildozer(add_action, dependency, target)


def remove_dependency(dependency, target):
  print("DEL: toRemove = ", toRemove, " from target = ", target, "\n")
  call_buildozer(remove_action, dependency, target)

def process_node(start):
  # go through its dependencies and
  # 1. add undeclared dependencies
  # 2. remove unused dependencies
  # 3. store unused dependencies 
  # 4. return them
  print("Processed node: " + start)
  return set()

def extract_info(dependencies):
  outgoing_edges = {}
  for target, payload in dependencies.items():
    outgoing_edges[target] = {}
    for dependency in payload["dependencies"]:
      outgoing_edges[target].update({dependency["target"]: dependency["dependency_type"]})
  return outgoing_edges

with open(universe) as f, open(direct_dependees) as g:
  dependencies = json.load(f)
  dependees = json.load(g)
  processed_dependencies = extract_info(dependencies)

  graph = Graph(processed_dependencies, dependees)

  graph.dfs(target, process_node, set())

  """for target, payload in data.items():
    targets_and_dependencies[target] = payload["dependencies"]
    dependencies = payload["dependencies"]
    for dep in dependencies:
      if dep["dependency_type"] == "undeclared":
        add_dependency(dep["target"], target)
      if dep["dependency_type"] == 'unused':
        remove_dependency(dep["target"], target)"""

  #print(targets_and_dependencies)

#pprint(data)
