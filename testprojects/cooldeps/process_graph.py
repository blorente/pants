import json
import subprocess
import sys
from pprint import pprint
from graph_traversal import Graph

path_to_buildozer = "pants-support/buildifier/bin/buildozer"
add_action = "add dependencies "
remove_action = "remove dependencies "
strict_deps_action = "print strict_deps"

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
  if dependency and parts[0].endswith(parts[1]):
    dependency = parts[0]
  return ' "' + action + dependency + '" '

def call_buildozer(action, dependency, target):
  cmd = build_command(action, dependency)
  full_cmd = " ".join([path_to_buildozer, cmd, target])
  result = subprocess.run(full_cmd, shell=True, stdout=subprocess.PIPE)
  if result.returncode > 1:
    return None
  else:
    return result.stdout.decode("utf-8")

def add_dependency(dependency, target):
  print("ADD: toAdd = ", dependency, "to target = ", target)
  call_buildozer(add_action, dependency, target)

def remove_dependency(dependency, target):
  print("DEL: toRemove = ", dependency, " from target = ", target)
  call_buildozer(remove_action, dependency, target)

def has_strict_deps_enabled(target):
  print("STRICT?: are strict deps enabled for {}".format(target))
  return bool(call_buildozer(strict_deps_action, "", target))

def process_node(node, universe):
  print("Processed node: " + node.name)
  removed_deps = set()

  # go through its dependencies and
  for dep_name, dep_type in node.outgoing_edges.items():
    # 1. add undeclared dependencies
    if dep_type == "undeclared":
      add_dependency(dep_name, target)
    if dep_type == 'unused':
      # 2. remove unused dependencies
      remove_dependency(dep_name, target)
      # 3. store unused dependencies
      removed_deps.add(dep_name)

  # 4. Add removed dependencies to direct dependees outside of the universe
  for dependee in node.incoming_edges - universe:
    # 4.1. If strict_deps is not enabled
    if not has_strict_deps_enabled(dependee):
      for removed_dep in removed_deps:
        add_dependency(removed_dep, dependee)

  # 5. return them
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
