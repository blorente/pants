import subprocess

path_to_buildozer = "pants-support/buildifier/bin/buildozer"
add_action = "add dependencies "
remove_action = "remove dependencies "
strict_deps_action = "print strict_deps"

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
  if result.returncode > 0:
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
  # print("STRICT?: are strict deps enabled for {}".format(target))
  return bool(call_buildozer(strict_deps_action, "", target))

def enable_strict_deps(target):
  # print("STRICT: Enabling strict deps for {}".format(target))
  call_buildozer("set strict_deps True", "", target)
