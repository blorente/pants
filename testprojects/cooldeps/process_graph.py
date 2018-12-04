import json
import subprocess
import sys
from pprint import pprint

json_file = sys.argv[1]

path_to_buildozer = "~/workspace/source/pants-support/buildifier/bin/buildozer"

def build_command(action, dependency):
  return "'" + action + " dependencies " + dependency + "'"

def add_dependency(toAdd, target):
  print("ADD: toAdd = ", toAdd, "to target = ", target, "\n")
  cmd = build_command("add", toAdd)
  subprocess.run([path_to_buildozer, cmd, target])


def remove_dependency(toRemove, target):
  print("DEL: toRemove = ", toRemove, " from target = ", target, "\n")
  cmd = build_command("remove", toRemove)
  subprocess.run([path_to_buildozer, cmd, target])

with open(json_file) as f:
  data = json.load(f)
  #print(data)

  targets_and_dependencies = {}
  for target, payload in data.items():
    targets_and_dependencies[target] = payload["dependencies"]
    dependencies = payload["dependencies"]
    for dep in dependencies:
      if dep["dependency_type"] == "undeclared":
        addDependency(dep["target"], target)
      if dep["dependency_type"] == 'unused':
        removeDependency(dep["target"], target)

  #print(targets_and_dependencies)

#pprint(data)
