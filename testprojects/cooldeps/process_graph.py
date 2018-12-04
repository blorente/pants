import json
import subprocess
import sys
from pprint import pprint

json_file = sys.argv[1]

path_to_buildozer = "pants-support/buildifier/bin/buildozer"

def build_command(action, dependency):
  parts = dependency.split(":")
  if parts[0].endswith(parts[1]):
    dependency = parts[0]
  return ' "' + action + ' dependencies ' + dependency + '" '

def add_dependency(toAdd, target):
  print("ADD: toAdd = ", toAdd, "to target = ", target, "\n")
  cmd = build_command("add", toAdd)
  full_cmd = " ".join([path_to_buildozer, cmd, target])
  print ("full_cmd = ", full_cmd, "\n")
  subprocess.run(full_cmd, shell=True)


def remove_dependency(toRemove, target):
  print("DEL: toRemove = ", toRemove, " from target = ", target, "\n")
  cmd = build_command("remove", toRemove)
  full_cmd = " ".join([path_to_buildozer, cmd, target])
  print ("full_cmd = ", full_cmd, "\n")
  subprocess.run(full_cmd, shell=True)

with open(json_file) as f:
  data = json.load(f)
  #print(data)

  targets_and_dependencies = {}
  for target, payload in data.items():
    targets_and_dependencies[target] = payload["dependencies"]
    dependencies = payload["dependencies"]
    for dep in dependencies:
      if dep["dependency_type"] == "undeclared":
        add_dependency(dep["target"], target)
      if dep["dependency_type"] == 'unused':
        remove_dependency(dep["target"], target)

  #print(targets_and_dependencies)

#pprint(data)
