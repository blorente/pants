#!/usr/local/bin/python3
import json

from argument_parsing import parse_cli
from buildozer_lib import add_dependency, enable_strict_deps, remove_dependency
from graph_traversal import Graph

processed_nodes = 0
def process_node(node, universe, deps_removed_by_dependencies):
  global processed_nodes
  processed_nodes += 1
  print("Processed node = {}, which is {}%".format(node.name, 100.0 * float(processed_nodes)/float(len(universe))))
  if node.name.startswith("3rdparty") or node.name.startswith("//:"):
    print("Processing 3rdparty. Stopping early.")
    return deps_removed_by_dependencies
  removed_deps = set()

  # go through its dependencies and
  for dep_name, dep_type in node.outgoing_edges.items():
    # 1. add undeclared dependencies
    if dep_type == "undeclared":
      add_dependency(dep_name, node.name)
    if dep_type == 'unused' and not dep_name.split(":")[1].startswith("thrift-"):
      # 2. remove unused dependencies
      remove_dependency(dep_name, node.name)
      # 3. store unused dependencies
      removed_deps.add(dep_name)

  # 4. Add removed dependencies to direct dependees outside of the universe
  dependencies_to_remove = removed_deps | deps_removed_by_dependencies
  for dependee in node.incoming_edges - universe:
    # 4.1. If strict_deps is not enabled
    # if not has_strict_deps_enabled(dependee):
    for removed_dep in dependencies_to_remove:
      add_dependency(removed_dep, dependee)

  # 5. return them
  return dependencies_to_remove

def extract_info(dependencies):
  outgoing_edges = {}
  for target, payload in dependencies.items():
    outgoing_edges[target] = {}
    for dependency in payload["dependencies"]:
      outgoing_edges[target].update({dependency["target"]: dependency["dependency_type"]})
  return outgoing_edges

def process_graph(args):
  with open(args.dependencies_file) as f, open(args.direct_dependees_file) as g:
    dependencies = json.load(f)
    dependees = json.load(g)
    processed_dependencies = extract_info(dependencies)

    graph = Graph(processed_dependencies, dependees)
    target_node = graph.graph[args.target]
    process_node(target_node, set(graph.graph.keys()), set())
    if args.enable_strict_deps:
      enable_strict_deps(args.target)


if __name__ == "__main__":
  args = parse_cli()
  process_graph(args)
